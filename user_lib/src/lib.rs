#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

extern crate alloc;
extern crate logger;
#[macro_use]
extern crate lazy_static;

// 定义 logger 层级
pub const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

#[macro_use]
pub mod console;

use alloc::{format, string::String, vec::Vec};
use bitflags::bitflags;
// 向外提供 kernel 配置，例如页大小
pub use sys_interface::config::*;
pub mod constant;

mod env;
pub use env::Env;

mod fs;
mod io;
mod lang_items;
mod start;
mod syscall;

use core::{ptr, todo};
use syscall::*;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

// 沟通 OS 系统调用, 发起请求后陷入 kernel
pub fn open(path: &str, flags: OpenFlags) -> isize {
    // TODO 应该把相对路径转为绝对路径
    let path = format!("{}\0", path);
    sys_open(path.as_str().as_ptr(), flags.bits())
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

pub fn list_dir(path: &str) -> isize {
    let path = format!("{}\0", path);
    sys_list_dir(path.as_str().as_ptr())
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
pub fn exec(name: &str, new_env: Option<Env>) -> isize {
    let args: Vec<String> = name.split(" ").map(|s| String::from(s)).collect();

    // 准备新的 env
    let new_env = match new_env {
        // args 参数替换
        Some(mut new_env) => {
            new_env.args_mut().clear();
            new_env.args_mut().extend(args);
            new_env
        }
        None => {
            // 否则新建一个
            let mut new_env = Env::from(Env::new());
            new_env.args_mut().clear();
            new_env.args_mut().extend(args);
            new_env
        }
    };

    //  --------------------argv---------------------------------------
    // 之所以需要一个新 vec 是因为要保存这些 C 字符串变量的生命周期
    let args_vec: Vec<String> = new_env
        .args()
        .iter()
        .map(|arg| format!("{}\0", arg))
        .collect();
    // 收集各个字符串的指针
    let mut args_ptr_vec: Vec<_> = args_vec.iter().map(|arg| (*arg).as_ptr()).collect();
    // 最后一个指针设为 null 表示数组结束
    args_ptr_vec.push(ptr::null());
    let args_ptr = args_ptr_vec.as_ptr();

    // -------------------envs-------------------------------------
    let envs_vec: Vec<String> = new_env
        .envs()
        .iter()
        .map(|(k, v)| format!("{}={}\0", k, v))
        .collect();
    let mut envs_ptr_vec: Vec<_> = envs_vec.iter().map(|arg| (*arg).as_ptr()).collect();
    envs_ptr_vec.push(ptr::null());
    let envs_ptr = envs_ptr_vec.as_ptr();

    // -----------可执行文件路径--------------------------------
    let new_envs = new_env.envs();
    assert!(new_envs.contains_key("HOME"));
    let filename = format!("{}/{}\0", new_envs.get("HOME").unwrap(), new_env.args()[0]);
    sys_execve(filename.as_ptr() as *const u8, args_ptr, envs_ptr)
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

pub fn shutdown() -> ! {
    sys_shutdown();
}
