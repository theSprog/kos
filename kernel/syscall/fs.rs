use crate::*;
use crate::{memory::page_table, process::processor, sbi::*};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// 由于内核和应用地址空间的隔离， sys_write 不再能够直接访问位于应用空间中的数据，
/// 而需要手动查页表才能知道那些数据被放置在哪些物理页帧上并进行访问
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = page_table::api::translated_byte_buffer(
                processor::api::current_user_token(),
                buf,
                len,
            );
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd(={fd}) in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        // stdin 每次读入一个字符
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in FD_STDIN!");
            let c = console_getchar(); // 阻塞式 IO;
            assert_ne!(c, 0);
            let ch = c as u8;
            let mut buffers = page_table::api::translated_byte_buffer(
                processor::api::current_user_token(),
                buf,
                len,
            );
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }

            len as isize
        }
        _ => {
            panic!("Unsupported fd(={fd}) in sys_read!");
        }
    }
}
