#![no_std]
#![no_main]

extern crate user_lib;
use user_lib::*;

const MAGIC: i32 = -12345;

#[no_mangle]
pub fn main() -> i32 {
    println!("I am the parent. Forking the child...");
    let pid = fork();
    if pid == 0 {
        println!("I am the child.");
        for _ in 0..7 {
            yield_cpu();
        }
        exit(MAGIC);
    } else {
        println!("I am parent, fork a child pid {}", pid);
    }
    println!("I am the parent, waiting now..");
    let mut xstate: i32 = 0;
    assert!(waitpid(pid as usize, &mut xstate) == pid && xstate == MAGIC);
    assert!(waitpid(pid as usize, &mut xstate) < 0 && wait(&mut xstate) <= 0);
    println!("waitpid {} ok.", pid);
    println!("exit pass.");
    0
}
