use crate::{
    init::get_kernel_bss_range, println, timer::get_time_ms, util::human_size, GeneralAllocator,
    KERNEL_HEAP_ORDER, KERNEL_HEAP_SIZE, PAGE,
};

use alloc::{format, string::String};
use core::{assert_eq, mem::size_of, ops::Range};
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
    use core::time::Duration;

    let start = Duration::from_millis(get_time_ms() as u64);
    info!("Heap test start, Time: [{:8} ms]", start.as_millis());

    let heap_range = get_kernel_heap_range();
    test_vec(&heap_range);
    test_box(&heap_range);
    test_string(&heap_range);
    test_hashmap(&heap_range);

    let end = Duration::from_millis(get_time_ms() as u64);
    info!(
        "Heap test passed! good luck, Time: [{:8} ms]",
        end.as_millis()
    );
}

fn test_vec(heap_range: &Range<usize>) {
    use alloc::vec::Vec;

    let len = 5000;
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
    string.remove(3);
    assert_eq!(string, "ranom string");

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
    let x = Box::new([1, 2, 3]);
    debug!("x: {:?}, x[2]: {:?}", x, x[2]);
    debug!("dealloc Box");
    drop(a);
}

fn test_hashmap(heap_range: &Range<usize>) {
    use crate::alloc::string::ToString;
    use hashbrown::HashMap;

    debug!("alloc hashmap for random insert String");

    // Type inference lets us omit an explicit type signature (which
    // would be `HashMap<String, String>` in this example).
    let mut book_reviews = HashMap::new();

    // Review some books.
    book_reviews.insert(
        "Adventures of Huckleberry Finn".to_string(),
        "My favorite book.".to_string(),
    );
    book_reviews.insert(
        "Grimms' Fairy Tales".to_string(),
        "Masterpiece.".to_string(),
    );
    book_reviews.insert(
        "Pride and Prejudice".to_string(),
        "Very enjoyable.".to_string(),
    );
    book_reviews.insert(
        "The Adventures of Sherlock Holmes".to_string(),
        "Eye lyked it alot.".to_string(),
    );

    // Check for a specific one.
    // When collections store owned values (String), they can still be
    // queried using references (&str).
    if !book_reviews.contains_key("Les Miserables") {
        debug!(
            "We've got {} reviews, but Les Miserables ain't one.",
            book_reviews.len()
        );
    }

    // oops, this review has a lot of spelling mistakes, let's delete it.
    book_reviews.remove("The Adventures of Sherlock Holmes");

    // Look up the values associated with some keys.
    let to_find = ["Pride and Prejudice", "Alice's Adventure in Wonderland"];
    for &book in &to_find {
        match book_reviews.get(book) {
            Some(review) => {
                debug!("{}: {}", book, review);
                assert!(heap_range.contains(&(review as *const _ as usize)))
            }
            None => debug!("{} is unreviewed.", book),
        }
    }

    // Look up the value for a key (will panic if the key is not found).
    debug!("Review for Jane: {}", book_reviews["Pride and Prejudice"]);

    // Iterate over everything.
    for (book, review) in &book_reviews {
        debug!("{}: \"{}\"", book, review);
    }
}
