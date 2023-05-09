#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

pub mod init;
pub mod interface;

mod console;
mod lang_items;
mod loader;
mod logger;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;
mod unicore;
mod util;

use crate::logger::LogLevel;
const LOG_LEVEL: LogLevel = LogLevel::TRACE;

// 配置信息
// ---------------------------------------------------------------------

pub const KB: usize = 1024;
pub const PAGE: usize = 4 * KB;

// 用户栈大小, 64K
pub const USER_STACK_SIZE: usize = 64 * KB;
// 内核栈大小, 32K, 应该开大一点，因为内核栈有时候会爆栈
// 比如下面的栈经过测试 3KB 会提示内核栈溢出 (canary 机制)
pub const KERNEL_STACK_SIZE: usize = 32 * KB;

// 最多允许 8 个 app
pub const MAX_APP_NUM: usize = 8;

// 0x80000000 - 0x80200000 固件地址
// 0x80200000 - 0x82000000 内核空间
// 0x82000000 - 0x87000000 用户空间
// 0x87000000 - 0x870012be 设备树区域
// 用户程序起始基地址
pub const USER_BASE_ADDRESS: usize = 0x82000000;
// 每个 app 的 size 上限, 128K
pub const APP_SIZE_LIMIT: usize = 0x20000;

// 金丝雀魔数, 用于检测栈溢出
pub const CANARY_MAGIC_NUMBER: u8 = 0x55;

/// 时钟频率, 机器每秒执行 CLOCK_FREQ 这么多 cycle
/// 因此 CLOCK_FREQ 可以理解为一秒
pub const CLOCK_FREQ: usize = 10000000;
