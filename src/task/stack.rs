use crate::{config::*, trap::context::TrapContext};

#[repr(align(4096))]
#[derive(Copy, Clone, Debug)]
pub struct KernelStack {
    pub(crate) data: [u8; KERNEL_STACK_SIZE],
}
impl KernelStack {
    pub fn new() -> KernelStack {
        KernelStack {
            data: [0; KERNEL_STACK_SIZE],
        }
    }

    pub fn push_context(&self, cx: TrapContext) -> usize {
        // 预留栈空间
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            // 将内容放进预留的空间中
            *cx_ptr = cx;
        }
        cx_ptr as usize
    }

    // 获取栈顶地址, 即数组结尾
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

#[repr(align(4096))]
#[derive(Copy, Clone, Debug)]
pub struct UserStack {
    pub(crate) data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    pub fn new() -> UserStack {
        UserStack {
            data: [0; USER_STACK_SIZE],
        }
    }
    // 获取栈顶地址, 即数组结尾
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub static KERNEL_STACKS: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

pub static USER_STACKS: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];
