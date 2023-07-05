#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, err_msg, link, open, unlink, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 3 {
        println!("Usage: mv <src> <dst>");
        return 1;
    }

    let src = args[1].as_str();
    let dst = args[2].as_str();
    let dst_fd = open(dst, OpenFlags::RDONLY, 0);
    if dst_fd >= 0 {
        println!("mv: {:?} {}", dst, err_msg(dst_fd));
        return 1;
    }
    let src_fd = open(src, OpenFlags::RDONLY, 0);
    if src_fd < 0 {
        println!("mv: {:?} {}", src, err_msg(src_fd));
        return -1;
    }

    let dst_fd = link(src, dst);
    if dst_fd < 0 {
        println!("mv: create hardlink {:?} <-> {:?} failed", src, dst);
        return 1;
    }
    let dst_fd = dst_fd as usize;

    let err = unlink(src);
    if err < 0 {
        println!("mv: {:?} {}", src, err_msg(err));
        close(dst_fd);
        return 1;
    }

    0
}
