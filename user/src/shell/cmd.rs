use core::ops::Deref;

use alloc::string::String;
use user_lib::{console::getchar, constant::*, *};

use crate::utils::clear_screen;

pub struct Cmd {
    prompt: String,
    cmd: String,
    cursor: usize,
    tablen: usize,
}

impl Cmd {
    pub fn new() -> Self {
        Self {
            prompt: String::from(">> "),
            cmd: String::from(""),
            cursor: 0,
            tablen: 4,
        }
    }

    pub fn set_tablen(&mut self, tablen: usize) {
        self.tablen = tablen;
    }

    // 更新内容到屏幕上
    pub fn fresh(&self) {
        print!("\x1B[2K\r"); // 删除当前行并将光标回到行首
        print!("{}{}", self.prompt, self.cmd);
        print!("\x1B[{}G", self.prompt.len() + self.cursor + 1); // 将光标设定在指定的列上
    }

    pub fn clear(&mut self) {
        self.cmd.clear();
        self.cursor = 0;
    }

    fn to_end(&mut self) {
        self.cursor = self.cmd.len();
    }

    fn to_start(&mut self) {
        self.cursor = 0;
    }

    fn add(&mut self, c: char) {
        self.cmd.insert(self.cursor, c);
        self.cursor += 1;
    }

    fn backspace(&mut self) {
        // cursor == 0 则无法再删除
        if self.cursor > 0 {
            self.cmd.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    fn cursor_back(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn cursor_forward(&mut self) {
        if self.cursor < self.cmd.len() {
            self.cursor += 1;
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
            (b'[', b'H') => {
                self.to_start();
            }
            (b'[', b'F') => {
                self.to_end();
            }

            _ => {
                print!("{}", ESC as char);
                print!("{}", c1 as char);
                print!("{}", c2 as char);
            }
        }
    }

    fn process_tab(&mut self) {
        for _ in 0..self.tablen {
            self.add(' ')
        }
    }

    // 换行
    fn new_line(&self) {
        println!("");
    }

    // 如果形成一行则返回 Some, 否则返回 None
    pub fn process_char(&mut self, c: u8) -> Option<&str> {
        match c {
            LF | CR => {
                self.new_line();
                // 敲下回车
                return Some(&self.cmd);
            }

            // ctrl + d 关机
            EOT => {
                self.new_line();
                shutdown();
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
