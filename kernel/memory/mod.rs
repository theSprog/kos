pub mod heap_alloc;

pub fn init() {
    heap_alloc::init_heap();
    heap_alloc::heap_test();
}
