#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec;
use user_lib::{thread_create, waittid};

pub fn thread_a(args: usize) -> i32 {
    print!("a");

    unsafe {
        let ptr: *const i32 = args as _; // 创建一个空指针
        let value: i32 = *ptr; // 强行解引用空指针, 模拟异常退出
        println!("Value: {}", value);
    }

    0
}

pub fn thread_b(_args: usize) -> i32 {
    print!("b");
    0
}

pub fn thread_c(_args: usize) -> i32 {
    print!("c");
    0
}

#[no_mangle]
pub fn main() -> i32 {
    let v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];

    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("main thread exited.");
    0
}
