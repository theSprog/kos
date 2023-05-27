use crate::util::*;
use crate::MB;
use core::{
    alloc::{GlobalAlloc, Layout},
    assert_eq, assert_ne,
    cmp::max,
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr::{null_mut, NonNull},
};
use logger::{debug, info};
use spin::Mutex;

use super::IAllocator;

macro_rules! look {
    ($num:expr, $pos:expr) => {
        ($num >> $pos) & 1
    };
}

// 分配内存的基本单位是一个 usize 长度
const BLOCK_UNIT: usize = size_of::<usize>();

// 最大管理堆大小, 设为 64 MB
const MAX_HEAP_SIZE: usize = 64 * MB;

// 需要多少个 bits (最多 1MB = 64MB / (8*8))
const BITMAP_SIZE: usize = MAX_HEAP_SIZE / (BLOCK_UNIT * 8);

pub struct Heap {
    // 用 1bit 代表一个 usize 区域的内存
    bitmap: [u8; BITMAP_SIZE],
    // 位图的有效位末端, 位图有效区域 [0, endpoint)
    endpoint: usize,

    // 堆的起始地址。堆的结束地址: heap_start_ptr + endpoint * BLOCK_UNIT
    heap_start_ptr: usize,

    // 附带信息
    // 用户请求使用的内存量
    user: usize,
    // 分配给用户的内存量, 可能与 user 不相等。比如用户请求一个 u8 但是分配一个 usize
    allocated: usize,

    // 总量
    total: usize,
}

impl Heap {
    const fn new() -> Self {
        Heap {
            bitmap: [0; BITMAP_SIZE],
            heap_start_ptr: 0,
            endpoint: 0,
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

    // 管理 size bytes 的堆大小
    pub fn init(&mut self, start: usize, size: usize) {
        assert!(
            size <= (8 * self.bitmap.len()) * BLOCK_UNIT,
            "Heap size 0x{:x} overflow for upper bound of bitmap(upper bound: 0x{:x})",
            size,
            MAX_HEAP_SIZE
        );

        // 避免某些平台地址不对齐
        assert_eq!(
            0,
            start % size_of::<usize>(),
            "start must be aligned with {}",
            size_of::<usize>()
        );
        assert_eq!(
            0,
            (start + size) % size_of::<usize>(),
            "end must be aligned with {}",
            size_of::<usize>()
        );

        assert_eq!(0, size % MB, "size({}) must be a multiple of MB", size);

        unsafe {
            self.add_to_heap(start, start + size);
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        // 至少分配一个 BLOCK_UNIT 空间, 可能会按 align 来分配, 例如请求 9 分配 16
        // alloc_size 表示需要分配的 bytes 数目
        let alloc_size = self.align(max(layout.size(), max(layout.align(), size_of::<usize>())));
        assert_eq!(0, alloc_size % BLOCK_UNIT);

        // 需要分配的单元数目
        let units = alloc_size / BLOCK_UNIT;
        let result = NonNull::new(self.find_free(units));

        if let Some(result) = result {
            self.user += layout.size();
            self.allocated += alloc_size;
            Ok(result)
        } else {
            Err(())
        }
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let dealloc_size = self.align(max(layout.size(), max(layout.align(), size_of::<usize>())));
        assert_ne!(dealloc_size, 0);
        let offset = ptr.as_ptr() as usize - self.heap_start_ptr;
        assert_eq!(offset % BLOCK_UNIT, 0);

        let units = offset / BLOCK_UNIT;
        let start = (units / 8, units % 8);

        assert_eq!(dealloc_size % BLOCK_UNIT, 0);
        self.fill(start, dealloc_size / BLOCK_UNIT, 0);

        self.user -= layout.size();
        self.allocated -= dealloc_size;
    }

    unsafe fn add_to_heap(&mut self, start: usize, end: usize) {
        assert!(start <= end);
        self.heap_start_ptr = start;

        // 包含多少个单元
        self.total = (end - start) / BLOCK_UNIT;

        // endpoint 就是 bitmap 的最后一位(不包含), 换句话说 bitmap 范围 [0, endpoint)
        // 这样做会浪费一些空间 (最多 (BLOCK_UNIT * 8) - 1)
        self.endpoint = self.total / 8;
    }

    fn align(&self, size: usize) -> usize {
        if size / BLOCK_UNIT == 0 {
            size
        } else {
            (size + BLOCK_UNIT - 1) / BLOCK_UNIT * BLOCK_UNIT
        }
    }

    fn find_free(&mut self, units: usize) -> *mut u8 {
        assert_ne!(units, 0);
        let mut byte_index = 0usize;
        let mut found_bits = 0usize;

        // bitmap 的空闲区域 start 索引
        let mut start = (0usize, 0usize);

        while byte_index < self.endpoint {
            let cur_byte = self.bitmap[byte_index];
            // 已分配, 跳过
            if cur_byte == 0xff {
                found_bits = 0;
                byte_index += 1;
                continue;
            }

            // 遍历每一个 bit
            for i in 0..8usize {
                if look!(cur_byte, i) == 0 {
                    // 如果刚找到, 需要更新 start
                    if found_bits == 0 {
                        start = (byte_index, i)
                    }
                    found_bits += 1;
                    if found_bits == units {
                        self.fill(start, found_bits, 1);
                        return self.conv_to_ptr(start);
                    }
                } else {
                    found_bits = 0;
                }
            }

            byte_index += 1;
        }

        // required number of bits not found
        null_mut()
    }

    fn fill(&mut self, start: (usize, usize), mut units: usize, fill: u8) {
        assert_ne!(units, 0);
        assert!(start.0 < self.endpoint && start.1 < 8);
        assert!(fill == 0 || fill == 1);

        let (mut byte_idx, mut bit_idx) = start;
        for i in bit_idx..8usize {
            if fill == 0 {
                // 如果填充 0 那么之前一定是 1
                assert_eq!(look!(self.bitmap[byte_idx], i), 1);
                self.bitmap[byte_idx] &= !(1 << i);
            } else {
                // 如果填充 1 那么之前一定是 0
                assert_eq!(look!(self.bitmap[byte_idx], i), 0);
                self.bitmap[byte_idx] |= 1 << i;
            }
            units -= 1;
            if units == 0 {
                break;
            }
        }

        if units != 0 {
            // 开始从下一字节开始查找
            byte_idx += 1;
            bit_idx = 0;

            let packed = units / 8;
            let rest = units % 8;
            for _ in 0..packed {
                if fill == 0 {
                    assert_eq!(self.bitmap[byte_idx], 0xff);
                    self.bitmap[byte_idx] = 0;
                } else {
                    assert_eq!(self.bitmap[byte_idx], 0);
                    self.bitmap[byte_idx] = 0xff;
                }
                byte_idx += 1;
            }

            if rest != 0 {
                self.fill((byte_idx, bit_idx), rest, fill);
            }
        }
    }

    // 将 bitmap 中找到的空闲区域转换为对应的指针起始
    fn conv_to_ptr(&self, start: (usize, usize)) -> *mut u8 {
        assert!(start.0 < self.endpoint && start.1 < 8);
        let (byte_idx, bit_idx) = start;
        let offset = ((byte_idx * 8) + bit_idx) * BLOCK_UNIT;
        (self.heap_start_ptr + offset) as *mut u8
    }

    fn display(&self) {
        // 此处必须使用不分配内存版本的 human_size_n
        // 因为我们已经把全局堆锁住了, human_size 会无法分配内存而一直阻塞, 形成死锁
        info!(
            "Mem Display: kernel-allocator = 'bitmap': [{:#x}..{:#x}), user = {}, allocated = {}, total = {}",
            self.heap_start_ptr,
            self.heap_start_ptr + self.endpoint * BLOCK_UNIT,
            human_size_n(self.user),
            human_size_n(self.allocated),
            human_size_n(self.total * BLOCK_UNIT)
        );
    }
}

pub struct LockedHeap(Mutex<Heap>);

// 转发到内部实现
impl LockedHeap {
    pub const fn empty() -> Self {
        LockedHeap(Mutex::new(Heap::new()))
    }

    pub fn init(&mut self, start: usize, size: usize) {
        self.0.lock().init(start, size);
    }

    pub fn display(&self) {
        self.0.lock().display();
    }
}

impl Deref for LockedHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 实现接口
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
