#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod batch;
mod init;
mod lang_items;
mod sbi;
mod unicore;

#[macro_use]
mod console;
use console::*;

use crate::init::*;
use core::arch::{asm, global_asm};

global_asm!(include_str!("entry.asm"));

const LOG_LEVEL: LogLevel = LogLevel::TRACE;

#[no_mangle]
pub fn entry() -> ! {
    init();
    println!("Hello, world!");
    panic!("goodbye world!");
}
