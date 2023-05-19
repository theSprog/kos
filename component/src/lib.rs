#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(fmt_internals)]

const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

extern crate alloc;

/// 本库用于放置各种可配置组件,
/// 包括内存分配算法, 进程调度算法, 文件系统
pub mod fs;
pub mod memory;
pub mod process;
pub mod util;
