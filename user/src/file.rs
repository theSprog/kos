#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, open, read, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let fd = open("/new_file.c", OpenFlags::RDWR);
    if fd == -1 {
        panic!("Error occured when opening file");
    }
    println!("Got file descriptor fd: {}", fd as usize);
    let fd = fd as usize;
    // let mut buf = [0u8; 256];
    // loop {
    //     let size = read(fd, &mut buf) as usize;
    //     if size == 0 {
    //         break;
    //     }
    //     println!("{}", core::str::from_utf8(&buf[..size]).unwrap());
    // }
    let res = close(fd);
    if res == -1 {
        panic!("Error occured when closing file");
    }
    println!("close() returned {}", res);
    0
}
