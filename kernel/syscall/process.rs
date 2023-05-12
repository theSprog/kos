use logger::info;

use crate::task::{exit_and_run_next, suspend_and_run_next};
use crate::timer::get_time_ms;

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

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}
