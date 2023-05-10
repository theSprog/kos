use core::ops::Range;

use crate::{debug, info, memory, println, task, timer};
use crate::{loader, trap};

pub fn kernel_start() -> bool {
    print_banner();
    clear_bss();

    trap::init();
    loader::init();
    timer::init();
    // task::start();
    memory::init();

    // 初始化成功
    true
}

pub fn init() -> ! {
    // something else to do and never exit

    // if init exit we have nothing to do so we must panic
    panic!("Process 'init' should not be exited!");
}

pub fn get_kernel_bss_range() -> Range<usize> {
    extern "C" {
        // bss 起始处
        fn sbss();
        // bss 结束处
        fn ebss();
    }

    sbss as usize..ebss as usize
}

fn clear_bss() {
    let bss = get_kernel_bss_range();
    debug!(
        "bss_start: {:p}, bss_end: {:p}, BSS size: 0x{:x} Bytes",
        bss.start as *const u8,
        bss.end as *const u8,
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
    println!("{}", include_str!("banner"));
    info!("KOS: A Simple Riscv Operating System Written In Rust");
    info!("Now I am initalizing something neccessary")
}
