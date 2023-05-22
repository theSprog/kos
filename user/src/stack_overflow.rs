#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    println!("It should trigger segmentation fault!");
    let ori = 0;
    f(0, &ori, 0);
    0
}

#[allow(unconditional_recursion)]
fn f(now: usize, ori: *const usize, depth: usize) {
    let offset = (ori as usize - &now as *const usize as usize) / 1024;
    if depth % 128 == 0 {
        println!("offset = {} KiB", offset);
    }
    f(now, ori, depth + 1);
}
