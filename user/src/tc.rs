#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, ftruncate, open, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        panic!("Usage: tc <path>");
    }
    let path = args.get(1).unwrap();

    let fd = open(path, OpenFlags::RDWR);
    if fd < 0 {
        panic!("Could not open \"{}\" file", path);
    }
    let fd = fd as usize;
    ftruncate(fd, 0);
    close(fd);
    0
}
