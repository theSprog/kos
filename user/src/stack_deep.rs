#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let start = 0; // 栈上放一个元素
    test_stack(&start, 0, &start);
    0
}

#[allow(unconditional_recursion)]
fn test_stack(current: *const u8, deep: usize, start: *const u8) {
    if deep % 128 == 0 {
        println!("{} KiB", (start as usize - current as usize) / 1024);
    }
    let new_current = 1;
    test_stack(&new_current, deep + 1, start);
}
