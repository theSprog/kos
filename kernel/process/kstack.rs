use crate::{
    memory::{address::*, address_space::KERNEL_SPACE, kernel_view, segment::MapPermission},
    process::pid::RecycleAllocator,
    sync::unicore::UPSafeCell,
};

use super::pid::Pid;

lazy_static! {
    static ref KSTACK_ID_ALLOCATOR: UPSafeCell<RecycleAllocator> =
        unsafe { UPSafeCell::new(RecycleAllocator::new()) };
}

#[derive(Debug)]
pub struct KernelStack {
    kstack_id: usize,
}

// RAII 进程退出时回收资源
impl Drop for KernelStack {
    fn drop(&mut self) {
        let kernel_view = kernel_view::get_kernel_view();
        let (kstack_bottom, _) = kernel_view.kernel_stack_range(self.kstack_id);
        let kstack_bottom_vaddr: VirtAddr = kstack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .free_kernel_stack_segment(kstack_bottom_vaddr.into());
    }
}

impl KernelStack {
    pub fn alloc() -> KernelStack {
        let kstack_id = KSTACK_ID_ALLOCATOR.exclusive_access().alloc();

        let kernel_view = kernel_view::get_kernel_view();
        let (bottom, top) = kernel_view.kernel_stack_range(kstack_id);
        // 内核栈立即分配
        KERNEL_SPACE.exclusive_access().insert_framed_segment(
            bottom.into(),
            top.into(),
            MapPermission::R | MapPermission::W,
        );
        KernelStack { kstack_id }
    }

    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) =
            kernel_view::get_kernel_view().kernel_stack_range(self.kstack_id);
        kernel_stack_top
    }
}
