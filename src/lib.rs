#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

pub mod config;
pub mod console;
pub mod init;
pub mod lang_items;
pub mod loader;
pub mod sbi;
pub mod syscall;
pub mod task;
pub mod trap;
pub mod unicore;
pub mod util;

use crate::console::LogLevel;
use console::*;
const LOG_LEVEL: LogLevel = LogLevel::TRACE;
