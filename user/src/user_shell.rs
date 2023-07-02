#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

use core::ops::Deref;

use alloc::string::String;
use alloc::vec::Vec;
use user_lib::console::getchar;
use user_lib::{chdir, constant::*, shutdown};
use user_lib::{exec, fork, waitpid};

struct Cmd {
    prompt: String,
    cmd: String,
    idx: usize,
    tablen: usize,
}

impl Cmd {
    fn new() -> Self {
        Self {
            prompt: String::from(">> "),
            cmd: String::from(""),
            idx: 0,
            tablen: 4,
        }
    }

    fn set_tablen(&mut self, tablen: usize) {
        self.tablen = tablen;
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

    fn process_esc(&mut self) {
        let c1 = getchar();
        let c2 = getchar();

        match (c1, c2) {
            (b'[', b'C') => {
                // →
                self.cursor_forward();
            }
            (b'[', b'D') => {
                // ←
                self.cursor_back();
            }
            (b'[', b'A') => {
                // ↑
            }
            (b'[', b'B') => {
                // ↓
            }
            _ => {
                print!("{}", ESC as char);
                print!("{}", c1 as char);
                print!("{}", c2 as char);
            }
        }
    }

    fn process_tab(&mut self) {
        for _ in 0..4 {
            self.add(' ')
        }
    }

    // 如果形成一行则返回 Some, 否则返回 None
    fn process_char(&mut self, c: u8) -> Option<&str> {
        match c {
            LF | CR => {
                // 敲下回车
                println!("");
                return Some(&self.cmd);
            }

            // 退格
            BS | DL => self.backspace(),
            // ctrl + l
            FF => clear_screen(),
            // ctrl + u
            NAK => self.clear(),
            ESC => self.process_esc(),
            TAB => self.process_tab(),

            WS..=WAVES => self.add(c as char),
            _ => self.add(c as char),
        }
        None
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

    loop {
        cmd.fresh();
        let c = getchar();
        // print!("{}?", c);
        let line = cmd.process_char(c);
        match line {
            None => continue,
            Some(line) => {
                let ret = shell_cmd(line);
                if ret < 0 {
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
        ["quit"] => shutdown(),
        // cd 命令针对当前 shell
        ["cd", paths @ ..] => cd(paths),
        _ => normal_cmd(line),
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
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
