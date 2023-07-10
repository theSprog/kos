use crate::{memory::kernel_view, trap::trap_return};

// 按照 C 方式解释，编译器不得重排它们
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct TaskContext {
    // 返回地址，__switch 最后一句代码需要用到它
    ra: usize,
    /// 内核栈指针
    sp: usize,
    // 被调用者保存的 12 个 s 寄存器
    s: [usize; 12],
}

impl TaskContext {
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }

    pub fn idle() -> TaskContext {
        // 生成一个 idle 进程的 TaskContext
        const IDLE_PID: usize = 0;
        let (_, top) = kernel_view::get_kernel_view().kernel_stack_range(IDLE_PID);

        Self {
            ra: 0usize,
            sp: top,
            s: [0; 12],
        }
    }
}
