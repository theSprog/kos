// 按照 C 方式解释，编译器不得重排它们
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct TaskContext {
    // 返回地址，__switch 最后一句代码需要用到它
    ra: usize,
    // 用户栈指针
    sp: usize,
    // 被调用者保存的 12 个 s 寄存器
    s: [usize; 12],
}

impl TaskContext {
    // 返回时返回到 __restore 从而恢复寄存器现场
    // 从 __restore 返回到用户态
    pub fn goto_restore(kernel_stack_ptr: usize) -> Self {
        extern "C" {
            fn __restore(cx_addr: usize);
        }
        Self {
            // 记录返回地址
            ra: __restore as usize,
            // sp 此时指向内核栈
            sp: kernel_stack_ptr,
            s: [0; 12],
        }
    }
}
