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
    println!("a: {}, prev: {}", 14, prev_power_of_two(14));
    println!("a: {}, prev: {}", 32, prev_power_of_two(32));
    println!("a: {}, next: {}", 14, (14 as usize).next_power_of_two());
    println!("a: {}, next: {}", 32, (32 as usize).next_power_of_two());
    todo!()
}

pub(crate) fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * (core::mem::size_of::<usize>()) - num.leading_zeros() as usize - 1)
}
