#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, open, read, Env, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        println!("Usage: cat <path>");
        return 1;
    }
    let path = args.get(1).unwrap();
    let fd = open(path, OpenFlags::RDONLY);
    if fd < 0 {
        panic!("Error occured when opening file \"{}\"", path);
    }

    const BUFFER_LEN: usize = 4096; // 4KiB
    let mut buffer = [0u8; BUFFER_LEN];
    let fd = fd as usize;
    loop {
        let read_size = read(fd, &mut buffer);
        if read_size <= 0 {
            break;
        }
        let read_size = read_size as usize;
        print!("{}", core::str::from_utf8(&buffer[..read_size]).unwrap());
    }
    close(fd);

    0
}
