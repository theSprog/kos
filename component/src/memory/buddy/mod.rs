#![cfg_attr(feature = "const_fn", feature(const_mut_refs, const_fn_fn_ptr_basics))]

extern crate alloc;

// 内存分配注册器
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::fmt;
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::NonNull;

// 使用的是 spin 自旋方式, 来自外部库 spin, 而非 std
// 因为只有 mutex 变量才能够线程间安全共享
extern crate spin;
use logger::info;
use spin::Mutex;

mod linked_list;

/// A heap that uses buddy system with configurable order.
///
/// # Usage
///
/// Create a heap and add a memory region to it:
/// ```
/// use buddy_system_allocator::*;
/// # use core::mem::size_of;
/// let mut heap = Heap::<32>::empty();
/// # let space: [usize; 100] = [0; 100];
/// # let begin: usize = space.as_ptr() as usize;
/// # let end: usize = begin + 100 * size_of::<usize>();
/// # let size: usize = 100 * size_of::<usize>();
/// unsafe {
///     heap.init(begin, size);
///     // or
///     heap.add_to_heap(begin, end);
/// }
/// ```
pub struct Heap<const ORDER: usize> {
    // buddy system with max order of `ORDER`
    free_list: [linked_list::LinkedList; ORDER],

    // statistics
    user: usize,
    allocated: usize,
    total: usize,
}

impl<const ORDER: usize> Heap<ORDER> {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            free_list: [linked_list::LinkedList::new(); ORDER],
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

    /// Create an empty heap
    pub const fn empty() -> Self {
        Self::new()
    }

    /// 向堆中添加一个内存范围 [start, end)
    pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        // avoid unaligned access on some platforms
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end &= !size_of::<usize>() + 1;
        assert!(start <= end);

        let mut total = 0;
        let mut current_start = start;

        while current_start + size_of::<usize>() <= end {
            // 保留最低一位, 其余都设定为 0。
            // 例如 start = 11010000 -> 00010000
            let lowbit = current_start & (!current_start + 1);
            let size = min(lowbit, prev_power_of_two(end - current_start));
            total += size;

            self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
            current_start += size;
        }

        self.total += total;
    }

    /// Add a range of memory [start, start+size) to the heap
    pub fn init(&mut self, start: usize, size: usize) {
        unsafe {
            self.add_to_heap(start, start + size);
        }
    }

    /// Alloc a range of memory from the heap satifying `layout` requirements
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        // 为了对齐, 取最大值
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            // Find the first non-empty size class
            if !self.free_list[i].is_empty() {
                // Split buffers
                for j in (class + 1..i + 1).rev() {
                    if let Some(block) = self.free_list[j].pop() {
                        unsafe {
                            self.free_list[j - 1]
                                .push((block as usize + (1 << (j - 1))) as *mut usize);
                            self.free_list[j - 1].push(block);
                        }
                    } else {
                        return Err(());
                    }
                }

                // NonNull 如果传入空, 则返回 None
                let result = NonNull::new(
                    self.free_list[class]
                        .pop()
                        .expect("current block should have free space now")
                        as *mut u8,
                );
                if let Some(result) = result {
                    self.user += layout.size();
                    self.allocated += size;
                    return Ok(result);
                } else {
                    return Err(());
                }
            }
        }
        Err(())
    }

    /// Dealloc a range of memory from the heap
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;

        unsafe {
            // Put back into free list
            self.free_list[class].push(ptr.as_ptr() as *mut usize);

            // Merge free buddy lists
            let mut current_ptr = ptr.as_ptr() as usize;
            let mut current_class = class;
            while current_class < self.free_list.len() {
                let buddy = current_ptr ^ (1 << current_class);
                let mut flag = false;
                for block in self.free_list[current_class].iter_mut() {
                    if block.value() as usize == buddy {
                        block.pop();
                        flag = true;
                        break;
                    }
                }

                // Free buddy found
                if flag {
                    self.free_list[current_class].pop();
                    current_ptr = min(current_ptr, buddy);
                    current_class += 1;
                    self.free_list[current_class].push(current_ptr as *mut usize);
                } else {
                    break;
                }
            }
        }

        self.user -= layout.size();
        self.allocated -= size;
    }

    /// Return the number of bytes that user requests
    pub fn stats_alloc_user(&self) -> usize {
        self.user
    }

    /// Return the number of bytes that are actually allocated
    pub fn stats_alloc_actual(&self) -> usize {
        self.allocated
    }

    /// Return the total number of bytes in the heap
    pub fn stats_total_bytes(&self) -> usize {
        self.total
    }
}

impl<const ORDER: usize> fmt::Debug for Heap<ORDER> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Heap")
            .field("user", &self.user)
            .field("allocated", &self.allocated)
            .field("total", &self.total)
            .finish()
    }
}

/// A locked version of `Heap`
///
/// # Usage
///
/// Create a locked heap and add a memory region to it:
/// ```
/// use buddy_system_allocator::*;
/// # use core::mem::size_of;
/// let mut heap = LockedHeap::<32>::new();
/// # let space: [usize; 100] = [0; 100];
/// # let begin: usize = space.as_ptr() as usize;
/// # let end: usize = begin + 100 * size_of::<usize>();
/// # let size: usize = 100 * size_of::<usize>();
/// unsafe {
///     heap.lock().init(begin, size);
///     // or
///     heap.lock().add_to_heap(begin, end);
/// }
/// ```

/// 伙伴系统以块为分配单位，每个块包含若干个物理页，物理页的数量必须是 2 的幂次
/// ORDER 决定了能 "连续分配" 的物理页, 相当于是空闲链表数组的长度。
/// 第 n 个数组项有 2^n 个物理页面。总和可以大于堆大小，但不能小于
const ORDER: usize = 32;
pub struct LockedHeap(Mutex<Heap<ORDER>>);

impl LockedHeap {
    /// Creates an empty heap
    pub const fn new() -> Self {
        LockedHeap(Mutex::new(Heap::<ORDER>::new()))
    }

    /// Creates an empty heap
    pub const fn empty() -> Self {
        LockedHeap(Mutex::new(Heap::<ORDER>::new()))
    }
}

// 实现 deref 可以直接使用内部的 Heap, LockedHeap 就是对 Heap 的一层封装
impl Deref for LockedHeap {
    type Target = Mutex<Heap<ORDER>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 调用内部的 Heap 的 alloc 实现
        // 其实也可以直接 self.lock() 但为了语义明显我们还是保留 self.0
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut::<u8>(), |allocation| {
                allocation.as_ptr()
            })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // 调用内部的 Heap 的 dealloc 实现
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

/// A locked version of `Heap` with rescue before oom
///
/// # Usage
///
/// Create a locked heap:
/// ```
/// use buddy_system_allocator::*;
/// let heap = LockedHeapWithRescue::new(|heap: &mut Heap<32>, layout: &core::alloc::Layout| {});
/// ```
///
/// Before oom, the allocator will try to call rescue function and try for one more time.
pub struct LockedHeapWithRescue<const ORDER: usize> {
    inner: Mutex<Heap<ORDER>>,
    rescue: fn(&mut Heap<ORDER>, &Layout),
}

impl<const ORDER: usize> LockedHeapWithRescue<ORDER> {
    /// Creates an empty heap
    #[cfg(feature = "const_fn")]
    pub const fn new(rescue: fn(&mut Heap<ORDER>, &Layout)) -> Self {
        LockedHeapWithRescue {
            inner: Mutex::new(Heap::<ORDER>::new()),
            rescue,
        }
    }

    /// Creates an empty heap
    #[cfg(not(feature = "const_fn"))]
    pub fn new(rescue: fn(&mut Heap<ORDER>, &Layout)) -> Self {
        LockedHeapWithRescue {
            inner: Mutex::new(Heap::<ORDER>::new()),
            rescue,
        }
    }
}

impl<const ORDER: usize> Deref for LockedHeapWithRescue<ORDER> {
    type Target = Mutex<Heap<ORDER>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

unsafe impl<const ORDER: usize> GlobalAlloc for LockedHeapWithRescue<ORDER> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.inner.lock();
        match inner.alloc(layout) {
            Ok(allocation) => allocation.as_ptr(),
            Err(_) => {
                (self.rescue)(&mut inner, &layout);
                inner
                    .alloc(layout)
                    .ok()
                    .map_or(core::ptr::null_mut::<u8>(), |allocation| {
                        allocation.as_ptr()
                    })
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner
            .lock()
            .dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

/// 找到小于等于 num 的最大的幂次
/// 例如 14 -> 8, 32 -> 32
/// 与其相对应的还有 next_power_of_two, 是找到大于等于 num 的最小的幂次
/// 例如 14 -> 16, 32 -> 32
pub(crate) fn prev_power_of_two(num: usize) -> usize {
    1 << (usize::BITS as usize - num.leading_zeros() as usize - 1)
}
