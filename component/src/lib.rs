#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]

const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

/// 本库用于放置各种可配置组件,
/// 包括内存分配算法, 进程调度算法, 文件系统
pub mod fs;
pub mod memory;
pub mod process;
