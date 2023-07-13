use logger::*;
use sys_interface::{syscall::*, syssig::SignalAction};

use crate::sbi::shutdown;
mod fs;
mod process;
mod signal;
mod thread;

use self::{fs::*, process::*, signal::*, thread::*};

/// 统一处理系统调用入口
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // 分发给不同的系统调用
    match syscall_id {
        // I/O 相关系统调用
        SYSCALL_OPENAT => sys_open(args[0] as *const u8, args[1] as u32, args[2] as u16),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_FTRUNCATE => sys_ftruncate(args[0], args[1]),
        SYSCALL_UNLINKAT => sys_unlinkat(args[0] as *const u8),
        SYSCALL_LINKAT => sys_linkat(args[0] as *const u8, args[1] as *const u8),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        SYSCALL_LSEEK => sys_lseek(args[0], args[1] as isize, args[2]),
        SYSCALL_MKDIRAT => sys_mkdirat(args[0] as *const u8, args[1]),
        SYSCALL_FSTAT => sys_fstat(args[0], args[1] as *mut u8),
        SYSCALL_PIPE2 => sys_pipe(args[0] as *mut usize),
        SYSCALL_DUP => sys_dup(args[0]),

        SYSCALL_IO_SETUP => sys_io_setup(args[0], args[1], args[2]),
        SYSCALL_IO_DESTROY => sys_io_destroy(args[0], args[1], args[2]),

        // 信号相关系统调用
        SYSCALL_KILL => sys_kill(args[0], args[1] as i32),
        SYSCALL_RT_SIGACTION => sys_sigaction(
            args[0] as i32,
            args[1] as *const SignalAction,
            args[2] as *mut SignalAction,
        ),
        SYSCALL_RT_SIGRETURN => sys_sigreturn(),
        SYSCALL_RT_SIGPROCMASK => sys_sigprocmask(args[0] as u32),

        // 进程相关系统调用
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

        SYSCALL_SHUTDOWN => shutdown(),

        // 自定义系统调用
        SYSCALL_CUSTOM_LISTDIR => sys_listdir(args[0] as *const u8),
        SYSCALL_CUSTOM_LISTAPPS => sys_listapps(),

        SYSCALL_CUSTOM_THREAD_CREATE => sys_thread_create(args[0], args[1]),

        // SYSCALL_IO_DESTROY => sys_io_destroy(args[0] as *const u8),
        // 严格来说这里不应该直接 panic,
        // 否则的话应用程序只需要一个非法系统调用就可以把 kernel 打挂
        _ => panic!(
            "Unsupported SYSCALL_ID: {}, SYSCALL_NAME: {}, args: {:?}",
            syscall_id, SYSCALL_CALL_NAME[syscall_id], args
        ),
    }
}
