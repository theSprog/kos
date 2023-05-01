#![no_main]
#![no_std]

// 将 kos 以一个库的形式打包
#[macro_use]
extern crate kos;

use core::arch::{asm, global_asm};
use kos::init::*;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn main() -> ! {
    init();

    println!("Hello, world!");
    panic!("goodbye world!");
}
