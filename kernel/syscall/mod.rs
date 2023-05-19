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
        // SYSCALL_BRK => sys_sbrk(args[0]), // 暂且用 sbrk 替代

        // 严格来说这里不应该直接 panic,
        // 否则的话应用程序只需要一个非法系统调用就可以把 kernel 打挂
        _ => panic!(
            "Unsupported SYSCALL_ID: {}, SYSCALL_NAME: {}",
            syscall_id, SYSCALL_CALL_NAME[syscall_id]
        ),
    }
}
