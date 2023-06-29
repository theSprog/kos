#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{unlink, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        println!("Usage: rm <path>");
        return 1;
    }

    let path = args.get(1).unwrap();
    let res = unlink(path);
    if res != 0 {
        println!("rm failed: {}", res);
        return 1;
    }

    0
}
