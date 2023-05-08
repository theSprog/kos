use core::arch::asm;

// 各种系统调用号
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

/// 向 fd 文件描述符写入 buffer 内的内容, 返回成功写入 u8 个数
/// # Arguments
///
/// * `fd` - 文件描述符
/// * `buffer` - 内存中缓冲区的起始地址
/// # Returns
///
/// 返回成功写入的 u8 长度
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 退出应用程序并将返回值告知批处理系统
/// # Arguments
///
/// * `exit_code` - 表示应用程序的返回值, 用来告知系统应用程序的执行状况
///
/// # Returns
///
/// 该函数正常来说永不返回
pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}
