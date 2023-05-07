#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::yield_;

const WIDTH: usize = 20;
const HEIGHT: usize = 6;

#[no_mangle]
fn main() -> i32 {
    for i in 0..HEIGHT {
        for _ in 0..WIDTH {
            print!("A");
        }
        println!(" [{}/{}]", i + 1, HEIGHT);
        yield_();
    }
    println!("App write_a OK and exit!");
    0
}
