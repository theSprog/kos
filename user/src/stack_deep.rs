#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let a = 0; // 栈上放一个元素
    test_stack(&a, 0, &a);
    0
}

#[allow(unconditional_recursion)]
fn test_stack(a: *const u8, deep: usize, ori: *const u8) {
    if deep % 128 == 0 {
        println!("{} KiB", (ori as usize - a as usize) / 1024);
    }
    let cur = 1;
    test_stack(&cur, deep + 1, ori);
}
