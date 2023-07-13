#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec;
use user_lib::{thread_create, yield_cpu};

pub fn thread_a() {
    loop {
        print!("a");
        yield_cpu();
    }
}

pub fn thread_b() -> ! {
    loop {
        print!("b");
        yield_cpu();
    }
}

pub fn thread_c() -> ! {
    loop {
        print!("c");
        yield_cpu();
    }
}

#[no_mangle]
pub fn main() -> i32 {
    let _v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    loop {
        print!("X");
        yield_cpu();
    }

    // for tid in v.iter() {
    //     let exit_code = waittid(*tid as usize);
    //     println!("thread#{} exited with code {}", tid, exit_code);
    // }
    // println!("main thread exited.");
    0
}
