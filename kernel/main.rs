#![no_main]
#![no_std]

use core::arch::global_asm;
use kos::{init::*, println};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn main() -> ! {
    // test_rust();
    // 内核初始化
    if !kernel_start() {
        panic!("kernel init failed!");
    }

    init();

    // if init exit we have nothing to do so we must panic
    panic!("Process 'init' should not be exited!");
}

global_asm!(include_str!("link_app.S"));

fn test_rust() {
    todo!()
}
