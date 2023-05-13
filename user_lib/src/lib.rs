#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![allow(dead_code)]

extern crate logger;
// 定义 logger 层级
pub const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

use syscall::*;

// 应用程序入口点
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    let exit_code = main();
    // 进程退出后调用 exit
    // 发生 panic 的进程不应该到此处，而会进入 panic 处理
    exit(exit_code);

    // 应该不可达
    unreachable!()
}

// 定义弱符号 main, 如果用户没有定义 main 则会进入该函数
// 否则会进入用户定义的 main 中
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        // bss 起始处
        fn start_bss();
        // bss 结束处
        fn end_bss();
    }

    let start_bss = start_bss as usize;
    let end_bss = end_bss as usize;
    // 将 bss 清零
    unsafe {
        // 优化后的版本, 更快
        core::ptr::write_bytes(start_bss as *mut u8, 0, end_bss - start_bss);
    }
}

// 沟通 OS 系统调用, 发起请求后陷入 kernel
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
pub fn yield_cpu() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
