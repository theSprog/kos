use crate::memory::{address::*, address_space::KERNEL_SPACE, kernel_view, segment::MapPermission};

use super::pid::Pid;

pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid_handle: &Pid) -> Self {
        let kernel_view = kernel_view::get_kernel_view();
        let pid = pid_handle.0;
        let (bottom, top) = kernel_view.kernel_stack_range(pid);
        KERNEL_SPACE.exclusive_access().insert_framed_segment(
            bottom.into(),
            top.into(),
            MapPermission::R | MapPermission::W,
        );
        KernelStack { pid: pid_handle.0 }
    }

    // // psuh 一个 Sized 结构体到栈顶
    // pub fn push_on_top<T: Sized>(&self, value: T) -> *mut T {
    //     let kernel_stack_top = self.get_top();
    //     let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
    //     unsafe {
    //         *ptr_mut = value;
    //     }
    //     ptr_mut
    // }

    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_view::get_kernel_view().kernel_stack_range(self.pid);
        kernel_stack_top
    }
}

// RAII 进程退出时回收资源
impl Drop for KernelStack {
    fn drop(&mut self) {
        let kernel_view = kernel_view::get_kernel_view();

        let (kernel_stack_bottom, _) = kernel_view.kernel_stack_range(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .release_kernel_stack_segment(kernel_stack_bottom_va.into());
    }
}
