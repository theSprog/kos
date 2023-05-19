#![no_std]
#![no_main]

use user_lib::{sbrk, PAGE_SIZE};

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let cur = sbrk(0);
    println!("cur heap_end :{}", cur);

    // 申请 1B 内存，但是按页分配
    let now = sbrk(1);
    println!("now heap_end :{}", now);
    unsafe {
        // 现在应该可写
        let cur = cur as *mut i32;
        *cur = 1;
        assert_eq!(*cur, 1);
    }

    let now2 = sbrk(4096);
    assert_eq!(now + PAGE_SIZE, now2);
    unsafe {
        let cur = now as *mut i32;
        *cur = 2;
        assert_eq!(*cur, 2);
    }

    let now3 = sbrk(4097);
    unsafe {
        // 现在应该可写
        let cur = (now2 + 4099) as *mut i32;
        *cur = 3;
        assert_eq!(*cur, 3);
    }
    assert_eq!(now2 + 2 * PAGE_SIZE, now3);
    0
}
