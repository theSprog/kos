use logger::{debug, info};

use crate::memory::kernel_view::get_kernel_view;
use crate::{loader, task, trap, KB};
use crate::{memory, timer};

pub fn kernel_start() -> bool {
    print_banner();
    clear_bss();
    memory::init();

    trap::init();
    timer::init(); // 开启分时机制
    task::start();
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
        "bss_start: {:#x}, bss_end: {:#x}, BSS size: 0x{:#x} Bytes",
        bss.start,
        bss.end,
        bss.len()
    );

    // 将 bss 清零
    info!("Clearing BSS, it may take some time if BSS is large enough");
    unsafe {
        // 优化后的版本, 更快
        core::ptr::write_bytes(bss.start as *mut u8, 0, bss.len());
    }

    // 不要用这个版本, 极慢
    // for bit_addr in bss {
    //     unsafe { (bit_addr as *mut u8).write_volatile(0u8) }
    // }
}

fn print_banner() {
    crate::println!("{}", include_str!("banner"));
    info!("KOS: A Simple Riscv Operating System Written In Rust");
    let kernel_view = get_kernel_view();
    let kernel_range = kernel_view.kernel_range();
    debug!(
        "kernel_start: {:#x}, kernel_end: {:#x}, kernel size: {:#x} KiB",
        kernel_range.start,
        kernel_range.end,
        kernel_range.len() / KB
    );
    info!("Now I am initalizing something neccessary");
}
