#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, err_msg, open, read, write, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 3 {
        println!("Usage: cp <src> <dst>");
        return 1;
    }

    let src = args[1].as_str();
    let dst = args[2].as_str();

    let src_fd = open(src, OpenFlags::RDONLY, 0);
    if src_fd < 0 {
        println!("mv: {:?} {}", src, err_msg(src_fd));
        return src_fd as i32;
    }
    let dst_fd = open(
        dst,
        OpenFlags::WRONLY | OpenFlags::TRUNC | OpenFlags::CREATE,
        0o644,
    );
    if dst_fd < 0 {
        println!("mv: {:?} {}", dst, err_msg(dst_fd));
        return dst_fd as i32;
    }

    let src_fd = src_fd as usize;
    let dst_fd = dst_fd as usize;
    const BUFFER_LEN: usize = 4096; // 4KiB
    let mut buffer = [0u8; BUFFER_LEN];
    loop {
        let read_count = read(src_fd, &mut buffer);
        if read_count == 0 {
            break;
        }
        let write_count = write(dst_fd, &buffer[..read_count as usize]);
        assert_eq!(read_count, write_count);
    }

    close(src_fd);
    close(dst_fd);
    0
}
