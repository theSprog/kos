use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

use super::context::TaskContext;

// 以函数的形式导出
extern "C" {
    // 由于声明为一个函数，所以在调用前后 Rust 编译器会自动帮助我们插入保存/恢复调用者保存寄存器的汇编代码
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
