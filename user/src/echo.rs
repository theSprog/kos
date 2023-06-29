#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{close, open, println, write, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 && args.len() != 4 {
        panic!("Usage: echo <msg> [>> <file>]");
    }
    if args.len() == 4 && args.get(2).unwrap() != ">>" {
        panic!("Usage: echo <msg> [>> <file>]");
    }

    if args.len() == 2 {
        let msg = args.get(1).unwrap();
        println!("{}", msg);
        0
    } else {
        let path = args.get(3).unwrap();
        let fd = open(path, OpenFlags::WRONLY | OpenFlags::CREATE);
        if fd < 0 {
            panic!("Could not create \"{}\" file", path);
        }
        let fd = fd as usize;
        let mut msg = args.get(1).unwrap().clone();
        msg.push('\n');
        write(fd, msg.as_bytes());
        close(fd);
        0
    }
}
