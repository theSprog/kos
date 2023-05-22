#![no_std]
#![no_main]

use user_lib::{console::getchar, constant::*, *};

#[no_mangle]
fn main() -> i32 {
    println!("please enter your text");
    let mut buf = [0; 1024];
    let mut len = 0;
    loop {
        let c = getchar();
        if c == LF || c == CR {
            // 都处理成换行
            println!("");
            break;
        }

        buf[len] = c;
        len += 1;
        print!("{}", c as char);
    }
    println!("I got '{}'", core::str::from_utf8(&buf[0..len]).unwrap());
    0
}
