use crate::{
    init::get_kernel_bss_range, println, util::human_size, GeneralAllocator, KERNEL_HEAP_ORDER,
    KERNEL_HEAP_SIZE, PAGE,
};

use alloc::format;
use core::{mem::size_of, ops::Range};
use logger::{debug, info};

// 任何对象只需要实现 alloc::alloc::GlobalAlloc 库中的分配函数
// pub unsafe fn alloc(&self, layout: Layout) -> *mut u8;
// pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
// 就能够使用动态内存分配

// 将 GeneralAllocator 作为全局堆分配器, GeneralAllocator 必须实现 GlobalAlloc 要求的抽象接口
#[global_allocator]
static HEAP_ALLOCATOR: GeneralAllocator = GeneralAllocator::empty();
// 内核堆空间, 位于内核的 .bss 段中
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

// 分配出错的 error handler
#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    if layout.size() >= KERNEL_HEAP_SIZE {
        panic!(
            "Heap allocation error: out of memory, allocated 0x{:x} bytes, layout = {:?}",
            layout.size(),
            layout,
        );
    }
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub fn init_heap() {
    info!("Memory allocator initalizing");
    let heap_range = get_kernel_heap_range();
    info!("Kernel heap initalizing");
    assert_eq!(
        0,
        heap_range.len() % PAGE,
        "Kernel heap size must be an integer multiple of the page size"
    );

    HEAP_ALLOCATOR
        .lock()
        .init(heap_range.start, KERNEL_HEAP_SIZE); // 以起点和长度作为参数

    info!(
        "kernel heap range: [0x{:x}..0x{:x}), size: {}",
        heap_range.start,
        heap_range.end,
        human_size(heap_range.len()) // 现在我们已经可以使用 format! 宏格式化字符串了
    );

    info!("Now String, Vec and other internal data-structures are available");
}

pub fn get_kernel_heap_range() -> Range<usize> {
    unsafe { (HEAP_SPACE.as_ptr() as usize)..(HEAP_SPACE.as_ptr() as usize + KERNEL_HEAP_SIZE) }
}

pub fn heap_test() {
    info!("Heap test start");
    let heap_range = get_kernel_heap_range();

    test_vec(&heap_range);
    test_box(&heap_range);
    test_string(&heap_range);

    info!("Heap test passed! good luck");
}

fn test_vec(heap_range: &Range<usize>) {
    use alloc::vec::Vec;

    let len = 500;
    debug!("alloc Vec of usize (len: {})", len);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..len {
        v.push(i);
    }
    for i in 0..len {
        assert_eq!(v[i], i);
    }
    debug!("size of Vec<usize> is {}", core::mem::size_of_val(&v));

    assert_eq!(
        core::mem::size_of_val(&v[0]) * v.len(),
        len * size_of::<usize>()
    );
    debug!(
        "total size of (content of Vec<usize>) is {}",
        core::mem::size_of_val(&v[0]) * v.len()
    );
    assert!(heap_range.contains(&(v.as_ptr() as usize)));
    debug!("dealloc Vec");
    drop(v);
}

fn test_string(heap_range: &Range<usize>) {
    use alloc::string::String;
    debug!("alloc String for random string");
    let mut string = String::new();
    string.push_str("random string");
    assert_eq!(string, "random string");

    debug!("size of String is {}", core::mem::size_of_val(&string));

    assert!(heap_range.contains(&(string.as_ptr() as usize)));
}

fn test_box(heap_range: &Range<usize>) {
    use alloc::boxed::Box;
    debug!("alloc Box ptr of '{}'", 5);
    let a = Box::new(5);
    assert_eq!(*a, 5);
    debug!("size of Box<i32> is {}", core::mem::size_of_val(&a));
    // a 是一个指针, 因此可以直接用 {:p} 打印
    debug!("assert bss contains '{:p}'", a);
    assert!(heap_range.contains(&(a.as_ref() as *const _ as usize)));
    debug!("dealloc Box");
    drop(a);
}
