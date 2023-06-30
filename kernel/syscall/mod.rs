use sys_interface::syscall::*;

use crate::sbi::shutdown;

use self::{fs::*, process::*};

mod fs;
mod process;

/// 统一处理系统调用入口
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // 分发给不同的系统调用
    match syscall_id {
        SYSCALL_OPENAT => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_FTRUNCATE => sys_ftruncate(args[0], args[1]),
        SYSCALL_UNLINKAT => sys_unlinkat(args[0] as *const u8),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        // SYSCALL_LSEEK => sys_lseek(args[0], args[1], args[2]),
        SYSCALL_MKDIRAT => sys_mkdirat(args[0] as *const u8, args[1]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_SCHED_YIELD => sys_sched_yield(),
        SYSCALL_GETTIMEOFDAY => sys_get_time_of_day(),
        SYSCALL_BRK => sys_sbrk(args[0]), // 暂且用 sbrk 替代
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_CLONE => sys_fork(),
        SYSCALL_EXECVE => sys_execve(
            args[0] as *const u8,
            args[1] as *const *const u8,
            args[2] as *const *const u8,
        ),
        SYSCALL_WAIT4 => sys_waitpid(args[0] as isize, args[1] as *mut i32),

        SYSCALL_IO_DESTROY => sys_io_destroy(args[0], args[1], args[2]),

        SYSCALL_SHUTDOWN => shutdown(),

        // 自定义系统调用
        SYSCALL_CUSTOM_LISTDIR => sys_listdir(args[0] as *const u8),

        // SYSCALL_IO_DESTROY => sys_io_destroy(args[0] as *const u8),
        // 严格来说这里不应该直接 panic,
        // 否则的话应用程序只需要一个非法系统调用就可以把 kernel 打挂
        _ => panic!(
            "Unsupported SYSCALL_ID: {}, SYSCALL_NAME: {}, args: {:?}",
            syscall_id, SYSCALL_CALL_NAME[syscall_id], args
        ),
    }
}
