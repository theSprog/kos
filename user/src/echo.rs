#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{eprintln, println, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        eprintln!("Usage: echo <msg>");
        return 1;
    }

    let msg = args.get(1).unwrap();
    println!("{}", msg);

    0
}
