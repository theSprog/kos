#![no_main]
#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(panic_info_message)]

mod init;
mod lang_items;
mod sbi;

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
