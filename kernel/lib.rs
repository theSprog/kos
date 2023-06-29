#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(fn_align)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

const LOG_LEVEL: logger::LogLevel = logger::LogLevel::DEBUG;

extern crate alloc;
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

pub mod console;
pub mod init;

mod clock;
mod driver;
mod fs;
mod lang_items;
mod loader;
mod memory;
mod process;
mod sbi;
mod sync;
mod syscall;
mod task;
mod trap;

use component::fs::ext2::Ext2FileSystem;
use process::PCB;
// 配置信息
// ---------------------------------------------------------------------
use sys_interface::config::*;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

/// QEMU 配置总内存大小 256 M, 区间 0x80000000..0x90000000
/// 内存基本分区如下
/// 0x80000000 - 0x80200000 固件(Firmware)地址
/// 0x80200000 - 0x84000000 内核空间 (大约 64 M, 结束点不精确)
/// 0x84000000 - 0x8f000000 用户空间
/// 0x8f000000 - 0x8f0012be 设备树区域

/// 可用内存空间的结尾, 从 USER_BASE_ADDRESS 到 MEMORY_END 会被页表管理
pub const MEMORY_END: usize = 0x8f000000;

/// 内核堆大小 64M
pub const KERNEL_HEAP_SIZE: usize = 64 * MB;

// 外部组件
// ----------------------------------------------------------------
// 使用 bitmap 分配内存
use component::memory::bitmap::LockedHeap;
type KernelHeapAllocator = LockedHeap;

// 使用 buddy 伙伴系统分配内存
// use component::memory::buddy::LockedHeap;
// type KernelHeapAllocator = LockedHeap;

use component::process::FIFO;
type KernelScheduler = FIFO<PCB>;

use component::fs::vfs;
type KernelFileSystem = Ext2FileSystem;
