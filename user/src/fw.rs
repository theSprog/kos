#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, get_time_ms, open, write, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    // test write speed
    let fd = open("/new_file.c", OpenFlags::RDWR);
    const BUFFER_LEN: usize = 4096; // 4KiB
    let mut buffer = [0u8; BUFFER_LEN];
    for ch in buffer.iter_mut() {
        *ch = '0' as u8 + (get_time_ms() as u8 % 10);
    }

    let fd = fd as usize;
    let start = get_time_ms();
    let size_mb = 1;
    let count = (size_mb * 1024 * 1024) / BUFFER_LEN;
    for _ in 0..count {
        let size = write(fd, &buffer);
        assert!(size >= 0);
        assert!(size as usize == BUFFER_LEN);
    }
    close(fd);

    let time_ms = (get_time_ms() - start) as usize;
    let speed_kbs = size_mb * 1024 * 1000 / time_ms;
    println!(
        "{}MiB written, time cost = {}ms, write speed = {}KiB/s",
        size_mb, time_ms, speed_kbs
    );
    0
}
