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

/// 向 fd 文件描述符写入 buffer 内的内容, 返回成功写入 u8 个数
/// # Arguments
///
/// * `fd` - 文件描述符
/// * `buffer` - 内存中缓冲区的起始地址
/// # Returns
///
/// 返回成功写入的 u8 长度
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_ptr() as usize, buffer.len()])
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

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}
pub fn sys_fork() -> isize {
    syscall(SYSCALL_CLONE, [0, 0, 0])
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

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPENAT, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}
