#![no_std]
#![no_main]

use user_lib::get_time_of_day;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("current time: {}", get_time_of_day());
    0
}
