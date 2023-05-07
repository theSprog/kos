use crate::task::{exit_and_run_next, suspend_and_run_next};
use crate::{info, println};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    // 处理方式就是挂起当前，并且运行下一个
    suspend_and_run_next();
    0
}
