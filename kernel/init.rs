use component::util::human_size::*;
use logger::{debug, info};

use crate::memory::kernel_view::get_kernel_view;
use crate::{clock, memory, fs};
use crate::{loader, process, task, trap};

pub fn kernel_start() -> bool {
    print_banner();
    clear_bss();

    memory::init();

    trap::init();
    clock::init(); // 开启分时机制
    loader::init();
    fs::init();
    task::api::init(); // 加载 init 进程, 它是第一个进程
    process::processor::api::run_app();
    // 初始化成功
    true
}

pub fn init() {
    // init process and never exit
}

fn clear_bss() {
    let kernel_view = get_kernel_view();
    let bss = kernel_view.bss_range();
    debug!(
        "bss_range: [{:#x}..{:#x}), BSS size: {}",
        bss.start,
        bss.end,
        debug_size(bss.len())
    );

    // 将 bss 清零
    info!("Clearing BSS, it may take some time if BSS is large enough");
    unsafe {
        // 优化后的版本, 更快
        core::ptr::write_bytes(bss.start as *mut u8, 0, bss.len());
    }
}

fn print_banner() {
    crate::println!("{}", include_str!("banner"));
    info!("KOS: A Simple RISC-V64 Operating System Written In Rust");
    let kernel_view = get_kernel_view();
    let kernel_range = kernel_view.kernel_range();
    debug!(
        "kernel_range: [{:#x}..{:#x}), kernel size: {}",
        kernel_range.start,
        kernel_range.end,
        debug_size(kernel_range.len())
    );
    info!("Now I am initalizing something neccessary");
}
