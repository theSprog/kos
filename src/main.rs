#![no_main]
#![no_std]

use core::arch::global_asm;
use kos::init::*;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn main() -> ! {
    // 创建 init 进程
    init();

    panic!("process init should not be exit!");
}
