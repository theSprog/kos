#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::vec;
use user_lib::{console::getchar, constant::*, *};

const DEPTH: usize = 4;

fn fork_child(cur: &str, branch: char) {
    let mut next = [0u8; DEPTH + 1];
    let len = cur.len();
    if len >= DEPTH {
        return;
    }
    next[..len].copy_from_slice(cur.as_bytes());
    next[len] = branch as u8;
    if fork() == 0 {
        fork_tree(core::str::from_utf8(&next[..len + 1]).unwrap());
        yield_cpu();
        exit(0);
    }
}

fn fork_tree(cur: &str) {
    println!("pid{}: {}", getpid(), cur);
    fork_child(cur, '0');
    fork_child(cur, '1');
}
#[no_mangle]
fn main() -> i32 {
    // 执行 user_shell
    // if fork() == 0 {
    //     exec("user_shell\0");
    // } else {
    //     loop {
    //         let mut exit_code: i32 = 0;
    //         // 将当前线程挂起直至子进程终结, exit_code 用于获取子进程退出码
    //         // pid 返回退出的子进程 pid 号
    //         let pid = wait(&mut exit_code);

    //         // 子进程尚不存在
    //         if pid == -1 {
    //             yield_cpu();
    //             continue;
    //         }
    //         println!(
    //             "[initproc] Released a zombie process, pid={}, exit_code={}",
    //             pid, exit_code,
    //         );
    //     }
    // }

    println!("starting user");

    fork_tree("");
    loop {}
}
