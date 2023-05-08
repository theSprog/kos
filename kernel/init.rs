use crate::{debug, info, println, task, timer};
use crate::{loader, trap};

pub fn kernel_start() -> bool {
    print_banner();
    clear_bss();

    trap::init();
    loader::init();
    timer::init();
    task::start();

    // 初始化成功
    true
}

pub(crate) fn clear_bss() {
    extern "C" {
        // bss 起始处
        fn sbss();
        // bss 结束处
        fn ebss();
    }
    // 将 bss 清零
    let bss_start = sbss as usize;
    let bss_end = ebss as usize;
    info!("bss_start:0x{:x}, bss_end:0x{:x}", bss_start, bss_end);
    for bit_addr in bss_start..bss_end {
        unsafe { (bit_addr as *mut u8).write_volatile(0u8) }
    }
}

pub(crate) fn print_banner() {
    let banner = include_str!("banner");
    println!("{}", banner);
}
