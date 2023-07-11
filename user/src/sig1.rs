#![no_std]
#![no_main]

use user_lib::*;

fn func(sig: usize) {
    println!("user_sig_test passed: {}", sig);
    // 信号处理函数不能普通 return 而要 sigreturn
    sigreturn();
}

#[no_mangle]
pub fn main() -> i32 {
    let mut new = SignalAction::default();
    let mut old = SignalAction::default();
    new.handler = func as usize;

    println!("signal_simple: sigaction");
    if sigaction(SIGUSR1, Some(&new), Some(&mut old)) < 0 {
        panic!("Sigaction failed!");
    }
    println!("signal_simple: kill");
    if kill(getpid() as usize, SIGUSR1) < 0 {
        println!("Kill failed!");
        exit(1);
    }
    println!("signal_simple: Done");
    0
}
