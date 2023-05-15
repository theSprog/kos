#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    for _i in 0..100 {
        print!("A");
    }
    0
}
