use sys_interface::syscall::*;

use self::{fs::sys_write, process::*};

mod fs;
mod process;

/// 统一处理系统调用入口
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // 分发给不同的系统调用
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_SCHED_YIELD => sys_sched_yield(),
        SYSCALL_GETTIMEOFDAY => sys_get_time_of_day(),
        _ => panic!("Unsupported SYSCALL_ID: {}", syscall_id),
    }
}
