#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

extern crate alloc;
extern crate logger;
// 定义 logger 层级
pub const LOG_LEVEL: logger::LogLevel = logger::LogLevel::WARN;

#[macro_use]
pub mod console;

use alloc::format;
// 向外提供 kernel 配置，例如页大小
pub use sys_interface::config::*;
pub mod constant;

mod lang_items;
mod start;
mod syscall;

use core::todo;
use syscall::*;

// 沟通 OS 系统调用, 发起请求后陷入 kernel
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
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
pub fn exec(path: &str) -> isize {
    // 手动在末尾加上 \0
    sys_exec(&format!("{}\0", path))
}

/// wait 任意子进程结束
/// 如果要等待的子进程不存在则返回 -1；
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        // 参数 -1 表示等待任何一个子进程
        match sys_waitpid(-1, exit_code as *mut _) {
            // 返回值 -2 表示进程未结束
            -2 => {
                // -2 不应该返回给用户
                yield_cpu();
            }
            // 返回值为 -1 表示不存在
            exit_pid => return exit_pid,
        }
    }
}

/// waitpid 等待特定子进程结束
/// 用户可观察到的要么是 -1, 要么是一个正数 pid
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_cpu();
            }
            // 返回值为 -1 表示不存在该进程
            exit_pid => return exit_pid,
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
