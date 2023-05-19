#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    for _ in 0..1000 {
        print!("A");
    }
    0
}
