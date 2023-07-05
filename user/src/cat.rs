#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fs::OpenOptions, io::Read, Env};

#[no_mangle]
pub fn main() -> i32 {
    let env = Env::new();
    let args = env.args();
    if args.len() != 2 {
        println!("Usage: cat <path>");
        return 1;
    }
    let path = args.get(1).unwrap();
    let mut file = OpenOptions::new().read(true).open(path).unwrap();

    const BUFFER_LEN: usize = 4096; // 4KiB
    let mut buffer = [0u8; BUFFER_LEN];
    loop {
        let read_size = file.read(&mut buffer).unwrap();
        if read_size == 0 {
            break;
        }
        print!("{}", core::str::from_utf8(&buffer[..read_size]).unwrap());
    }
    println!("");

    0
}
