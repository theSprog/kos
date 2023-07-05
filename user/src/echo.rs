#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{close, open, println, red, write, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 && args.len() != 4 {
        red!("Usage: echo <msg> [>> <file>]");
        return 1;
    }
    if args.len() == 4 && args.get(2).unwrap() != ">>" {
        red!("Usage: echo <msg> [>> <file>]");
        return 1;
    }

    if args.len() == 2 {
        let msg = args.get(1).unwrap();
        println!("{}", msg);
    } else {
        let path = args.get(3).unwrap();
        let fd = open(
            path,
            OpenFlags::WRONLY | OpenFlags::CREATE | OpenFlags::APPEND,
            0o644,
        );
        if fd < 0 {
            red!("Could not create \"{}\" file", path);
            return 1;
        }
        let fd = fd as usize;
        let mut msg = args.get(1).unwrap().clone();
        msg.push('\n');
        write(fd, msg.as_bytes());
        close(fd);
    }

    0
}
