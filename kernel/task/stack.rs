use logger::debug;

use crate::{trap::context::TrapContext, *};

#[repr(align(4096))]
#[derive(Copy, Clone, Debug)]
pub struct KernelStack {
    pub(crate) data: [u8; KERNEL_STACK_SIZE],
}
impl KernelStack {
    pub const fn new() -> KernelStack {
        let mut ret = KernelStack {
            data: [0; KERNEL_STACK_SIZE],
        };
        // 埋伏金丝雀
        ret.data[0] = CANARY_MAGIC_NUMBER;
        ret
    }

    // 准备好返回的 context, 返回指向 context 的指针
    pub fn push_context(&self, ctx: TrapContext) -> usize {
        let trap_ctx_size = core::mem::size_of::<TrapContext>();
        assert!(
            trap_ctx_size <= KERNEL_STACK_SIZE,
            "trap_ctx_size(size: {}) too large for KERNEL_STACK_SIZE(size: {})",
            trap_ctx_size,
            KERNEL_STACK_SIZE
        );
        // 预留栈空间
        let ctx_ptr = (self.get_sp() - trap_ctx_size) as *mut TrapContext;

        debug!(
            "original_kernel_sp: 0x{:x}, trap_ctx_size: 0x{:x}, now_kernel_sp: 0x{:x}",
            self.get_sp(),
            trap_ctx_size,
            self.get_sp() - trap_ctx_size
        );

        unsafe {
            // 将内容放进预留的空间中
            *ctx_ptr = ctx;
        }
        ctx_ptr as usize
    }

    // 获取栈顶地址, 即数组结尾
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub(crate) fn check_canary(&self) {
        assert_eq!(
            self.data[0], CANARY_MAGIC_NUMBER,
            "oops! kernel stack overflow"
        );
    }
}

#[repr(align(4096))]
#[derive(Copy, Clone, Debug)]
pub struct UserStack {
    pub(crate) data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    pub const fn new() -> UserStack {
        let mut ret = UserStack {
            data: [0; USER_STACK_SIZE],
        };
        // 埋伏金丝雀
        ret.data[0] = CANARY_MAGIC_NUMBER;
        ret
    }

    // 获取栈顶地址, 即数组结尾
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }

    pub(crate) fn check_canary(&self) {
        assert_eq!(
            self.data[0], CANARY_MAGIC_NUMBER,
            "oops! user stack overflow"
        );
    }
}

pub static KERNEL_STACKS: [KernelStack; MAX_APP_NUM] = [KernelStack::new(); MAX_APP_NUM];

pub static USER_STACKS: [UserStack; MAX_APP_NUM] = [UserStack::new(); MAX_APP_NUM];
