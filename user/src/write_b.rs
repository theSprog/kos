#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    for _i in 0..10000 {
        print!("B");
    }
    0
}
