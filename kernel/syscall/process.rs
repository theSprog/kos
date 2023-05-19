use logger::*;

use crate::task;
use crate::timer;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!(
        "App-{} exited with code {}",
        task::api::current_tid(),
        exit_code
    );
    task::api::exit_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_sched_yield() -> isize {
    // 处理方式就是挂起当前，并且运行下一个
    task::api::suspend_and_run_next();
    0
}

pub fn sys_get_time_of_day() -> isize {
    timer::get_time_ms() as isize
}

pub fn sys_sbrk(incrment: usize) -> isize {
    task::api::sbrk(incrment) as isize
}
