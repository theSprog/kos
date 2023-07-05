#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{err_msg, unlink, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() == 1 {
        println!("Usage: rm <path>...");
        return 1;
    }

    let path_num = args.len() - 1;
    for i in 0..path_num {
        let path = args.get(i + 1).unwrap();
        let err = unlink(path);
        if err != 0 {
            println!("rm: {:?} {}", path, err_msg(err));
            return 1;
        }
    }

    0
}
