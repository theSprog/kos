#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fs::OpenOptions, get_time_ms, io::Write};

#[no_mangle]
pub fn main() -> i32 {
    // test write speed
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .trunc(true)
        .create(true, 0o644)
        .open("/home/new_file.c")
        .unwrap();

    const BUFFER_LEN: usize = 4096; // 4KiB
    let mut buffer = [0u8; BUFFER_LEN];
    for ch in buffer.iter_mut() {
        *ch = '0' as u8 + (get_time_ms() as u8 % 10);
    }

    let start = get_time_ms();
    let size_mb = 1;
    let count = (size_mb * 1024 * 1024) / BUFFER_LEN;
    for _ in 0..count {
        let size = file.write(&buffer).unwrap();
        assert!(size as usize == BUFFER_LEN);
    }
    let time_ms = (get_time_ms() - start) as usize;

    let speed_kbs = size_mb * 1024 * 1000 / time_ms;
    println!(
        "{}MiB written, time cost = {}ms, write speed = {}KiB/s",
        size_mb, time_ms, speed_kbs
    );
    0
}
