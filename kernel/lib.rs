#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(fn_align)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

const LOG_LEVEL: logger::LogLevel = logger::LogLevel::TRACE;

#[macro_use]
extern crate lazy_static;
extern crate alloc;
extern crate bitflags;

pub mod console;
pub mod init;

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
pub const MB: usize = 1024 * KB;
pub const PAGE_SIZE: usize = 4 * KB;
// 单页页宽
pub const PAGE_SIZE_BITS: usize = 12;

// 用户栈大小, 8MB, 由于有了虚拟内存, 可以开大一点
pub const USER_STACK_SIZE: usize = 8 * MB;
// 内核栈大小, 512K, 应该开大一点，因为内核栈有时候会爆栈
// 比如下面的栈经过测试 3KB 会提示内核栈溢出 (canary 机制, 以及分页后的 guard page 机制)
pub const KERNEL_STACK_SIZE: usize = 512 * KB;

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

/// 内核堆大小 32M
pub const KERNEL_HEAP_SIZE: usize = 0x2_000_000;

// 外部组件
// ----------------------------------------------------------------
// 使用 bitmap 分配内存
use component::memory::bitmap::LockedHeap;
type KernelHeapAllocator = LockedHeap;

// 使用 buddy 伙伴系统分配内存
// use component::memory::buddy::LockedHeap;
// type KernelHeapAllocator = LockedHeap;
