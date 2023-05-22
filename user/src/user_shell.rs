#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

use alloc::string::String;
use user_lib::console::getchar;
use user_lib::constant::*;
use user_lib::{exec, fork, waitpid};
const PROMPT: &str = ">> ";

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    run_shell()
}

fn run_shell() -> i32 {
    let mut line: String = String::new();
    print!("{}", PROMPT);

    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    line.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        // child process
                        if exec(&line) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                    line.clear();
                }
                print!("{}", PROMPT);
            }

            // 退格
            BS | DL => {
                if !line.is_empty() {
                    backspace(&mut line);
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}

fn backspace(line: &mut String) {
    // 光标回退
    cursor_back();
    print!(" ");
    cursor_back();
    line.pop();
}

fn cursor_back() {
    print!("{}", BS as char);
}
