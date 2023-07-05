#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::io::*;
use user_lib::{lseek, open, read, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let fd = open("/home/new_file.c", OpenFlags::RDWR, 0) as usize;
    lseek(fd, 5, SEEK_SET);
    let mut buf = [0; 64];
    read(fd, &mut buf);
    println!("len: {}", buf.len());
    println!("content: {}", core::str::from_utf8(&buf).unwrap());

    lseek(fd, -3, SEEK_END);
    let mut buf = [0; 64];
    read(fd, &mut buf);
    println!("len: {}", buf.len());
    println!("content: {}", core::str::from_utf8(&buf).unwrap());

    lseek(fd, 9, SEEK_CURRENT);
    let mut buf = [0; 64];
    read(fd, &mut buf);
    println!("len: {}", buf.len());
    println!("content: {}", core::str::from_utf8(&buf).unwrap());

    lseek(fd, 0, SEEK_SET);
    let mut buf = [0; 64];
    read(fd, &mut buf);
    println!("len: {}", buf.len());
    println!("content: {}", core::str::from_utf8(&buf).unwrap());
    0
}
