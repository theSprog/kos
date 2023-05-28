#![no_std]
#![no_main]

use user_lib::console::getchar;

extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    let div = getchar();
    100 / (div - b'0') as i32
}
