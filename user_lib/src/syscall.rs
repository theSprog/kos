//! 本 mod 将上层传入的高级参数翻译为底层可识别的形式
//! 例如 &[u8] 转为起始地址和长度
use core::{arch::asm, todo};

use sys_interface::syscall::*;

#[inline(always)]
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_mkdir(dirpath: *const u8, mode: usize) -> isize {
    syscall(SYSCALL_MKDIRAT, [dirpath as usize, mode, 0])
}

pub fn sys_unlink(filepath: *const u8) -> isize {
    syscall(SYSCALL_UNLINKAT, [filepath as usize, 0, 0])
}

pub fn sys_link(to: *const u8, from: *const u8) -> isize {
    syscall(SYSCALL_LINKAT, [to as usize, from as usize, 0])
}

// pub fn sys_rmdir(dirpath: *const u8) -> isize {
//     syscall(SYSCALL_RMDIRAT, [dirpath as usize, 0])
// }

pub fn sys_ftruncate(fd: usize, length: usize) -> isize {
    syscall(SYSCALL_FTRUNCATE, [fd, length, 0])
}

pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize {
    syscall(SYSCALL_LSEEK, [fd, offset as usize, whence])
}

/// 退出应用程序并将返回值告知系统
/// # Arguments
///
/// * `exit_code` - 表示应用程序的返回值, 用来告知系统应用程序的执行状况
///
/// # Returns
///
/// 该函数正常来说永不返回
pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_SCHED_YIELD, [0, 0, 0])
}

pub fn sys_get_time_ms() -> isize {
    // 其实这个系统调用号是返回自 1970年1月1日 到现在的时间的时间差
    // 但是我们现在假借这个系统调用号来获取自开机起到当前的时间
    syscall(SYSCALL_GETTIMEOFDAY, [0, 0, 0])
}

pub fn sys_brk(_addr: usize) -> isize {
    todo!();
}

pub fn sys_sbrk(incrment: usize) -> isize {
    // 目前 sbrk 暂时借用 brk 系统调用
    syscall(SYSCALL_BRK, [incrment, 0, 0])
}

pub fn sys_kill(pid: usize, signal: i32) -> isize {
    syscall(SYSCALL_KILL, [pid, signal as usize, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}
pub fn sys_fork() -> isize {
    syscall(SYSCALL_CLONE, [0, 0, 0])
}

pub fn sys_pipe(pipe_fd: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE2, [pipe_fd.as_mut_ptr() as usize, 0, 0])
}

pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0])
}

/// 只将字符串起始地址传入, 因此需要用户调用该系统调用时先将 \0 准备好
pub fn sys_execve(filename: *const u8, args: *const *const u8, envs: *const *const u8) -> isize {
    syscall(
        SYSCALL_EXECVE,
        [filename as usize, args as usize, envs as usize],
    )
}

/// 如果要等待的子进程不存在则返回 -1；
/// 否则如果要等待的子进程均未结束则返回 -2；
/// 否则返回结束的子进程的进程 ID。
pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAIT4, [pid as usize, exit_code as usize, 0])
}

pub fn sys_shutdown() -> ! {
    syscall(SYSCALL_SHUTDOWN, [0, 0, 0]);
    unreachable!();
}

pub fn sys_open(path: *const u8, flags: u32, mode: u16) -> isize {
    syscall(
        SYSCALL_OPENAT,
        [path as usize, flags as usize, mode as usize],
    )
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}

// 通过 open 打开 dir, 将返回的 dir fd 传入得到 DirEntry
// pub fn sys_getdents(fd: usize, entry: *mut DirEntry) -> isize {
//     syscall(SYSCALL_GETDENTS64, [fd, entry as usize, 0])
// }

pub fn sys_list_dir(path: *const u8) -> isize {
    syscall(SYSCALL_CUSTOM_LISTDIR, [path as usize, 0, 0])
}

pub fn sys_chdir(path: *const u8) -> isize {
    syscall(SYSCALL_CHDIR, [path as usize, 0, 0])
}

pub fn sys_getcwd(buf: *mut u8, size: usize) -> isize {
    syscall(SYSCALL_GETCWD, [buf as usize, size, 0])
}

pub fn sys_sigaction(signal: i32, action: usize, old_action: usize) -> isize {
    syscall(SYSCALL_RT_SIGACTION, [signal as usize, action, old_action])
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    syscall(SYSCALL_RT_SIGPROCMASK, [mask as usize, 0, 0])
}

pub fn sys_sigreturn() -> isize {
    syscall(SYSCALL_RT_SIGRETURN, [0, 0, 0])
}
