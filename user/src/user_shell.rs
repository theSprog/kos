#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

use core::ops::Deref;

use alloc::borrow::ToOwned;
use alloc::string::String;
use user_lib::console::getchar;
use user_lib::{chdir, constant::*, shutdown};
use user_lib::{exec, fork, waitpid};

struct Cmd {
    prompt: String,
    cmd: String,
    idx: usize,
}

impl Cmd {
    fn new() -> Self {
        Self {
            prompt: ">> ".to_owned(),
            cmd: "".to_owned(),
            idx: 0,
        }
    }
    fn fresh(&self) {
        print!("\x1B[2K\r"); // 删除当前行并将光标回到行首
        print!("{}{}", self.prompt, self.cmd);
        print!("\x1B[{}G", self.prompt.len() + self.idx + 1); // 将光标设定在指定的列上
    }
    fn clear(&mut self) {
        self.cmd.clear();
        self.idx = 0;
    }

    fn add(&mut self, c: char) {
        self.cmd.insert(self.idx, c);
        self.idx += 1;
    }
    fn backspace(&mut self) {
        if self.idx > 0 {
            self.cmd.remove(self.idx - 1);
            self.idx -= 1;
        }
    }
    fn cursor_back(&mut self) {
        if self.idx > 0 {
            self.idx -= 1;
        }
    }
    fn cursor_forward(&mut self) {
        if self.idx < self.cmd.len() {
            self.idx += 1;
        }
    }
}

impl Deref for Cmd {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.cmd
    }
}

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    run_shell()
}

fn run_shell() -> i32 {
    let mut cmd = Cmd::new();
    cmd.fresh();

    loop {
        let c = getchar();
        match c {
            LF | CR => {
                // 敲下回车
                println!("");
                let ret = shell_cmd(&cmd);
                if ret < 0 {
                    return ret;
                }
                cmd.clear();
            }

            // 退格
            BS | DL => {
                cmd.backspace();
            }

            WS..=WAVES => {
                cmd.add(c as char);
            }

            ESC => {
                let c1 = getchar();
                let c2 = getchar();

                match (c1, c2) {
                    (b'[', b'C') => {
                        // →
                        cmd.cursor_forward();
                    }
                    (b'[', b'D') => {
                        // ←
                        cmd.cursor_back();
                    }
                    (b'[', b'A') => {
                        // ↑
                    }
                    (b'[', b'B') => {
                        // ↓
                    }
                    // (b'[', b'K') => {
                    //     // 清屏
                    //     print!(" [K ")
                    // }
                    // (b'[', b'J') => {
                    //     // 清行
                    //     print!(" [J ")
                    // }
                    // (b'[', b'H') => {
                    //     // 回车
                    //     print!(" [H ")
                    // }
                    // (b'[', b'M') => {
                    //     // 换行
                    //     print!(" [M ")
                    // }
                    _ => {
                        print!("{}", ESC as char);
                        print!("{}", c1 as char);
                        print!("{}", c2 as char);
                    }
                }
            }

            _ => {
                cmd.add(c as char);
            }
        }
        cmd.fresh();
    }
}

fn shell_cmd(line: &String) -> i32 {
    let line = line.trim();

    match &line[..] {
        "" => 0,
        "quit" => shutdown(),
        // cd 命令针对当前 shell
        cmd if cmd.starts_with("cd ") => cd(cmd.split_at(3).1.trim()),
        _ => normal_cmd(line),
    }
}

fn cd(path: &str) -> i32 {
    let path = if path.is_empty() { "." } else { path };
    let res = chdir(path);
    if res < 0 {
        println!("Cannot change dir \"{}\": {}", path, res);
    }
    // 始终要返回 0, shell 不能退出
    0
}

fn normal_cmd(line: &str) -> i32 {
    let pid = fork();
    if pid == 0 {
        // child process
        if exec(line, None) == -1 {
            println!("\x1B[31mError when executing \"{}\"!\x1B[0m", line);
            return -4;
        }
        unreachable!();
    } else {
        let mut exit_code: i32 = 0;
        let exit_pid = waitpid(pid as usize, &mut exit_code);
        assert_eq!(pid, exit_pid);
        if exit_code != 0 {
            println!(
                "\x1B[31mShell: Process {} exited with code {}\x1B[0m",
                pid, exit_code
            );
        } else {
            println!(
                "\x1B[32mShell: Process {} exited with code {}\x1B[0m",
                pid, exit_code
            );
        }
    }
    0
}
