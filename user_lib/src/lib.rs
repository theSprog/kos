#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

extern crate alloc;
extern crate logger;
#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)]
use logger::*;

// 定义 logger 层级
pub const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

#[macro_use]
pub mod console;

use alloc::{format, string::String};
// 向外提供 kernel 配置，例如页大小
pub use sys_interface::config::*;
pub use sys_interface::syserr;
pub use sys_interface::sysfs::OpenFlags;
pub use sys_interface::syssig::*;

pub mod constant;

mod env;
pub use env::Env;
pub mod fs;
pub mod io;

mod lang_items;
mod start;
mod syscall;
use syscall::*;

// 沟通 OS 系统调用, 发起请求后陷入 kernel
pub fn open(path: &str, flags: OpenFlags, mode: u16) -> isize {
    // TODO 应该把相对路径转为绝对路径
    let path = format!("{}\0", path);
    sys_open(path.as_str().as_ptr(), flags.bits(), mode)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn ftruncate(fd: usize, size: usize) -> isize {
    sys_ftruncate(fd, size)
}

pub fn lseek(fd: usize, offset: isize, whence: usize) -> isize {
    sys_lseek(fd, offset, whence)
}

pub fn list_dir(path: &str) -> isize {
    let path = format!("{}\0", path);
    sys_list_dir(path.as_str().as_ptr())
}

pub fn list_apps() -> isize {
    sys_list_apps()
}

pub fn chdir(path: &str) -> isize {
    let path = format!("{}\0", path);
    sys_chdir(path.as_str().as_ptr())
}

pub fn getcwd(buffer: &mut [u8]) -> isize {
    sys_getcwd(buffer.as_mut_ptr(), buffer.len())
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn mkdir(path: &str, mode: usize) -> isize {
    let path = format!("{}\0", path);
    sys_mkdir(path.as_str().as_ptr(), mode)
}

pub fn unlink(path: &str) -> isize {
    let path = format!("{}\0", path);
    sys_unlink(path.as_str().as_ptr())
}

pub fn link(to: &str, from: &str) -> isize {
    let to = format!("{}\0", to);
    let from = format!("{}\0", from);
    sys_link(to.as_str().as_ptr(), from.as_str().as_ptr())
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
pub fn yield_cpu() -> isize {
    sys_yield()
}
pub fn get_time_ms() -> isize {
    sys_get_time_ms()
}

pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}

pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}

pub fn exec(line: &str, new_env: Option<Env>) -> isize {
    // 由于要和内核交互需要极其小心生命周期管理
    // 准备新的 env
    let new_env = Env::build_env(line, new_env);
    // argv
    let (args_vec, args_ptrs) = new_env.build_c_args();
    // envs
    let (envs_vec, envs_ptr) = new_env.build_c_envs();

    // 可执行文件路径
    let exec_app = format!("{}\0", args_vec[0]);
    sys_execve(exec_app.as_ptr(), args_ptrs.as_ptr(), envs_ptr.as_ptr())
}

/// wait 任意子进程结束
/// 如果要等待的子进程不存在则返回 -1；
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        // 参数 -1 表示等待任何一个子进程
        match sys_waitpid(-1, exit_code as *mut _) {
            syserr::EAGAIN => {
                // 等待一段时间再重试
                yield_cpu();
            }
            // 返回值为 -1 表示不存在
            syserr::ECHILD => return -1,

            exit_pid => {
                assert!(exit_pid > 0, "exit_pid must be positive: {}", exit_pid);
                return exit_pid;
            }
        }
    }
}

/// waitpid 等待特定子进程结束
/// 用户可观察到的要么是 -1, 要么是一个正数 pid
pub fn waitpid(pid: isize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid, exit_code as *mut i32) {
            syserr::EAGAIN => {
                // 子进程尚未退出, 等待一段时间再重试
                yield_cpu();
            }
            syserr::ECHILD => return syserr::ECHILD,
            exit_pid => {
                assert!(exit_pid > 0, "exit_pid must be positive: {}", exit_pid);
                return exit_pid;
            }
        }
    }
}
pub fn sleep(period_ms: usize) {
    // 以毫秒的形式返回值
    let start = sys_get_time_ms();
    while sys_get_time_ms() < start + period_ms as isize {
        sys_yield();
    }
}

pub fn brk(_addr: usize) -> i32 {
    // On success, brk() returns zero.  On error, -1 is returned
    // sys_brk(addr) as i32;
    todo!();
}

pub fn sbrk(incrment: usize) -> usize {
    // 调用 brk 进行实现
    sys_sbrk(incrment) as usize
}

/// 当前进程向另一个进程（可以是自身）发送一个信号。
pub fn kill(pid: usize, signal: i32) -> isize {
    sys_kill(pid, signal)
}

/// 为 signal 注册某种处理函数
pub fn sigaction(
    signal: i32,
    action: Option<&SignalAction>,
    old_action: Option<&mut SignalAction>,
) -> isize {
    let action = action.map_or(core::ptr::null(), |a| a) as usize;
    let old_action = old_action.map_or(core::ptr::null_mut(), |a| a) as usize;
    sys_sigaction(signal, action, old_action)
}

// 设置进程的信号屏蔽掩码
pub fn sigprocmask(mask: u32) -> isize {
    sys_sigprocmask(mask)
}

pub fn sigreturn() -> isize {
    sys_sigreturn()
}

pub fn shutdown() -> ! {
    sys_shutdown();
}

pub fn err_msg(syscall_err: isize) -> String {
    format!(
        "{} (os errno {})",
        syserr::msg(syscall_err),
        syserr::errno(syscall_err)
    )
}
