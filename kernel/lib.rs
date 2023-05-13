#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

extern crate alloc;

extern crate logger;
const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

extern crate qemu_config;

pub mod console;
pub mod init;
pub mod interface;

mod lang_items;
mod loader;
mod memory;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;
mod unicore;
mod util;

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

/// 内核堆大小, 16M
pub const KERNEL_HEAP_SIZE: usize = 0x1_000_000;
/// 伙伴系统以块为分配单位，每个块包含若干个物理页，物理页的数量必须是 2 的幂次
/// ORDER 决定了能连续分配的物理页, 相当于是空闲链表数组的长度。
/// 第 n 个数组项有 2^n 个物理页面。总和可以大于堆大小，但不能小于
pub const KERNEL_HEAP_ORDER: usize = 32;

// 外部组件
// ----------------------------------------------------------------
// 使用 bitmap 分配内存
use component::memory::bitmap::LockedHeap;
type GeneralAllocator = LockedHeap;

// 使用 buddy 伙伴系统分配内存
// use component::memory::buddy::LockedHeap;
// type GeneralAllocator = LockedHeap<KERNEL_HEAP_ORDER>;
