#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

use crate::cmd::Cmd;
use alloc::vec::Vec;
use user_lib::{console::getchar, *};

mod cmd;
mod utils;

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust User Shell");
    run_shell()
}

fn run_shell() -> i32 {
    let mut cmd = Cmd::new();

    loop {
        cmd.fresh();
        let c = getchar();
        let line = cmd.process_char(c);
        match line {
            None => continue,
            Some(line) => {
                let ret = shell_cmd(line);
                if ret < 0 {
                    // shell 永远返回 0, 负值只能是 app 的返回值
                    return ret;
                }
                cmd.clear();
            }
        };
    }
}

fn shell_cmd(line: &str) -> i32 {
    let line = line.trim();
    if line.is_empty() {
        return 0;
    }

    let words = line.split_whitespace().collect::<Vec<_>>();
    match words.as_slice() {
        ["shutdown"] => shutdown(),
        // exit 只退出当前 shell, 而不会关机
        ["exit"] => exit(0) as i32,
        // cd 命令针对当前 shell
        ["cd", paths @ ..] => cd(paths),
        _ => normal_cmd(line),
    }
}

fn cd(paths: &[&str]) -> i32 {
    if paths.len() <= 1 {
        let path = if paths.is_empty() { "." } else { paths[0] };
        let res = chdir(path);
        if res < 0 {
            println!("Cannot change dir to \"{}\": {}", path, res);
        }
    }

    // 始终要返回 0, shell 不能退出
    0
}

fn normal_cmd(line: &str) -> i32 {
    let pid = fork();
    if pid == 0 {
        // 子进程部分
        if exec(line, None) == -1 {
            red!("Error when executing \"{}\"", line);
            // 子进程返回 -4
            return -4;
        }
        unreachable!();
    } else {
        let mut exit_code: i32 = 0;
        let exit_pid = waitpid(pid as usize, &mut exit_code);
        assert_eq!(pid, exit_pid);

        let msg = alloc::format!("Shell: Process {} exited with code {}", pid, exit_code);
        match exit_code == 0 {
            true => green!("{}", msg),
            false => red!("{}", msg),
        };
        0
    }
}
