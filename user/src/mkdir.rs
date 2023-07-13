#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{err_msg, mkdir, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        println!("Usage: mkdir <path>");
        return 1;
    }
    let path = args.get(1).unwrap();
    let res = mkdir(path, 0o774);
    if res != 0 {
        println!("mkdir: {:?} {}", path, err_msg(res));
        return 1;
    }

    0
}
