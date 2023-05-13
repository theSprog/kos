#![no_std]
#![no_main]

use core::unreachable;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    loop {
        print!("A");
        unreachable!()
    }
}
