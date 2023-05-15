use crate::{memory::page_table::translated_byte_buffer, print, task::current_user_token};

const FD_STDOUT: usize = 1;

/// 由于内核和应用地址空间的隔离， sys_write 不再能够直接访问位于应用空间中的数据，
/// 而需要手动查页表才能知道那些数据被放置在哪些物理页帧上并进行访问
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
