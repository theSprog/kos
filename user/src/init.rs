#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    // 执行 shell
    if fork() == 0 {
        exec("shell", None);
    } else {
        loop {
            let mut exit_code: i32 = 0;

            // 将当前线程挂起直至任意子进程终结, exit_code 用于获取子进程退出码
            // pid 返回退出的子进程 pid 号
            let pid = wait(&mut exit_code);

            // 尚不存在死亡的子进程
            if pid == -1 {
                yield_cpu();
                continue;
            }
            println!(
                "[init] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }

    unreachable!()
}
