#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![allow(dead_code)]

#[macro_use]
pub mod console;
mod lang_items;
mod logger;
mod syscall;

use syscall::*;
const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

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
    // 将 bss 清零
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

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
