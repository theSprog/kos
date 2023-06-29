#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::{close, get_time_ms, open, read, write, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    random_str_test(4096 * 15);
    0
}

fn random_str_test(len: usize) {
    let fd = open("/new_file.c", OpenFlags::RDWR | OpenFlags::TRUNC) as usize;
    assert!(fd == 3);
    println!("test len: {}", len);
    let mut str = String::new();
    // random digit
    for _ in 0..len {
        str.push(char::from('0' as u8 + (get_time_ms() as u8 % 10)));
    }
    let write = write(fd, &str.as_bytes());
    println!("write size: {}", write);
    close(fd);

    let fd = open("/new_file.c", OpenFlags::RDWR) as usize;
    assert!(fd == 3);
    let mut read_buffer = [0u8; 8192];
    let mut read_str = String::new();
    println!("start read");

    loop {
        let len = read(fd, &mut read_buffer);
        if len <= 0 {
            break;
        }
        let len = len as usize;
        read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
    }
    close(fd);
    println!("str_len: {} read_str_len:{}", str.len(), read_str.len());
    // assert 失败
    assert_eq!(str, read_str);
}
