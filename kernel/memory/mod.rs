pub mod address;
pub mod address_space;
pub mod frame;
pub mod heap_alloc;
pub mod kernel_view;
pub mod page_table;
pub mod segment;

pub fn init() {
    heap_alloc::init_allocator();
    frame::init_frame_allocator();

    // 初始化 KERNEL_SPACE 并且激活
    address_space::KERNEL_SPACE
        .exclusive_access()
        .enable_paging();

    address_space::remap_test();
}
