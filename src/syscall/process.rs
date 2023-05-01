use crate::{batch::run_apps, info, println};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}\n", exit_code);
    run_apps() // 批处理系统上一个任务处理完后又要继续处理下一个任务
}
