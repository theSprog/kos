#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{err_msg, list_dir, syserr, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() > 2 {
        println!("Usage: ls <path>");
        return 1;
    }

    let path = if args.len() == 1 { "." } else { &args[1] };
    let res = list_dir(path);
    if res != 0 {
        println!("{:?}: {}", path, err_msg(res));
    }

    syserr::errno(res) as i32
}
