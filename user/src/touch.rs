#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{close, open, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        panic!("Usage: touch <path>");
    }
    let path = args.get(1).unwrap();

    let fd = open(path, OpenFlags::CREATE, 0o644);
    if fd < 0 {
        panic!("Could not create \"{}\" file", path);
    }
    let fd = fd as usize;
    close(fd);
    0
}
