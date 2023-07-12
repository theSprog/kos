#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

use crate::cmd::Cmd;
use alloc::vec::Vec;
use parser::parse_line;
use user_lib::{console::getchar, *};

mod cmd;
mod parser;
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
                if ret != 0 {
                    // shell 永远返回 0, 负值只能是 app 的返回值
                    println!("failed to execute command {:?}", line);
                    loop {}
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
        ["apps"] => list_apps() as i32,
        // exit 只退出当前 shell, 而不会关机
        // ["exit"] => exit(0) as i32,
        // cd 命令针对当前 shell
        ["cd", paths @ ..] => cd(paths),
        _ => parse_line(line),
    }
}

fn cd(paths: &[&str]) -> i32 {
    if paths.len() <= 1 {
        let path = if paths.is_empty() { "." } else { paths[0] };
        let err = chdir(path);
        if err != 0 {
            println!("cd: {:?}: {}", path, err_msg(err));
        }
    }

    // 始终要返回 0, shell 不能退出
    0
}
