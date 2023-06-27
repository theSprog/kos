#![allow(unused)]

use core::arch::asm;

use logger::*;

use crate::{fs::inode::VFS, println};

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;

// system reset extension
const SYSTEM_RESET_EXTENSION: usize = 0x53525354;
const SBI_SHUTDOWN: usize = 0;
const SBI_COLD_REBOOT: usize = 1;

#[derive(Clone, Copy, Debug)]
pub struct SBIRet {
    error: isize,
    value: usize,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum SBIErrType {
    SBI_SUCCESS = 0,                // Completed successfully
    SBI_ERR_FAILED = -1,            // Failed
    SBI_ERR_NOT_SUPPORTED = -2,     // Not supported
    SBI_ERR_INVALID_PARAM = -3,     // Invalid parameter(s)
    SBI_ERR_DENIED = -4,            // Denied or not allowed
    SBI_ERR_INVALID_ADDRESS = -5,   // Invalid address(s)
    SBI_ERR_ALREADY_AVAILABLE = -6, // Already available
    SBI_ERR_ALREADY_STARTED = -7,   // Already started
    SBI_ERR_ALREADY_STOPPED = -8,   // Already stopped
    SBI_ERR_NO_SHMEM = -9,          // Shared memory not available
}

impl From<isize> for SBIErrType {
    fn from(value: isize) -> Self {
        match value {
            0 => SBIErrType::SBI_SUCCESS,
            -1 => SBIErrType::SBI_ERR_FAILED,
            -2 => SBIErrType::SBI_ERR_NOT_SUPPORTED,
            -3 => SBIErrType::SBI_ERR_INVALID_PARAM,
            -4 => SBIErrType::SBI_ERR_DENIED,
            -5 => SBIErrType::SBI_ERR_INVALID_ADDRESS,
            -6 => SBIErrType::SBI_ERR_ALREADY_AVAILABLE,
            -7 => SBIErrType::SBI_ERR_ALREADY_STARTED,
            -8 => SBIErrType::SBI_ERR_ALREADY_STOPPED,
            -9 => SBIErrType::SBI_ERR_NO_SHMEM,
            _ => panic!("Invalid value for conversion to SBIErrType"),
        }
    }
}

// 向下层 SBI 发起调用
// 新标准有一个 fid
#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SBIRet {
    let ret1;
    let ret2;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret1,
            inlateout("x11") arg1 => ret2,
            in("x12") arg2,
            in("x16") fid,
            in("x17") eid,
        );
    }
    SBIRet {
        error: ret1,
        value: ret2,
    }
}

// 设置下一次中断的发生时间
pub fn set_timer(timer: usize) {
    sbi_call(SBI_SET_TIMER, 0, timer, 0, 0);
}

pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, 0, c, 0, 0);
}

pub fn console_getchar() -> usize {
    let ret = sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0, 0);
    // 由于是用 legacy 代码因此 error 反而才是 返回值
    // 这是 sbi 不兼容导致的
    ret.error as usize
}

pub fn console_getchar_nio(base_addr: *const u8) -> usize {
    panic!("unsupported getchar function");
    // let low = ((base_addr as usize) << 32) >> 32;
    // let high = base_addr as usize >> 32;
    // debug!(
    //     "low = {:#x}, high = {:#x}, base_addr = {:?}",
    //     low, high, base_addr
    // );
    // let ret = sbi_call(0x4442434E, 1, 1, low, high);
    // assert_eq!(SBIErrType::SBI_SUCCESS, ret.error.into());
    // ret.value
}

#[allow(clippy::empty_loop)]
pub fn shutdown() -> ! {
    // Shutdown should flush filesystem
    {
        VFS.flush()
    };
    println!("goodbye!");
    let ret = sbi_call(SYSTEM_RESET_EXTENSION, SBI_SHUTDOWN, 0, 0, 0);
    error!("cannot be here");
    // 此时 assert 已经不管用了, 因为 assert 失败会 shutdown, 又回到这个函数
    // 同样 unreachable 也不可能行，因此只打印一句 error 就 loop
    // 最后的防线如果不关机，就自旋
    loop {}
}

#[allow(clippy::empty_loop)]
pub fn reboot() -> ! {
    println!("rebooting...");
    let ret = sbi_call(SYSTEM_RESET_EXTENSION, SBI_COLD_REBOOT, 0, 0, 0);
    assert_eq!(SBIErrType::SBI_SUCCESS, ret.error.into());
    loop {}
}
