#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{getcwd, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() > 1 {
        println!("Usage: pwd");
        return 1;
    }

    let mut cwd_buffer = [0u8; 512];
    let res = getcwd(&mut cwd_buffer);
    if res < 0 {
        println!("getcwd failed");
        return 1;
    }

    let cwd_slice = core::str::from_utf8(&cwd_buffer).unwrap();
    // 删除尾部 0
    let cwd_slice = cwd_slice.trim_end_matches(char::from(0));
    println!("{}", cwd_slice);

    res as i32
}
