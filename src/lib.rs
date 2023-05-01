#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod batch;
pub mod console;
pub mod init;
pub mod lang_items;
pub mod sbi;
pub mod syscall;
pub mod trap;
pub mod unicore;

use crate::console::LogLevel;
use console::*;
const LOG_LEVEL: LogLevel = LogLevel::TRACE;
