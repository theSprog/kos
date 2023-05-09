#![no_main]
#![no_std]

use core::arch::global_asm;
use kos::init::*;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn main() -> ! {
    // 内核初始化
    if !kernel_start() {
        panic!("kernel init failed!");
    }
    panic!("process init should not be exit!");
}

global_asm!(include_str!("link_app.S"));
