#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(fmt_internals)]
#![allow(unused_variables)]

// #![feature(error_in_core)]

use sys_interface::config::*;
const LOG_LEVEL: logger::LogLevel = logger::LogLevel::INFO;

extern crate alloc;
extern crate lazy_static;

pub mod crt0;
/// 本库用于放置各种可配置组件,
/// 包括内存分配算法, 进程调度算法, 文件系统 等等
pub mod device_tree;
pub mod fs;
pub mod memory;
pub mod process;
pub mod util;
