#![no_main]
#![no_std]

use core::{
    arch::asm,
    fmt::{self, Write},
};

#[inline(always)]
fn logger_sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret
}
fn logger_console_putchar(c: usize) {
    const SBI_CONSOLE_PUTCHAR: usize = 1;
    logger_sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().for_each(|c| logger_console_putchar(c as usize));
        Ok(())
    }
}

pub fn logger_print(args: fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

#[derive(Debug)]
pub enum Color {
    Red = 31,
    Yellow = 93,
    Blue = 35,
    Green = 32,
    Gray = 34,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    ERROR = 50,
    WARN = 40,
    INFO = 30,
    DEBUG = 20,
    TRACE = 10,
}

// 通用打印
#[macro_export]
macro_rules! log {
    ($color:expr, $level:literal, $($arg:tt)*) => {
        $crate::logger_print(format_args!("\x1B[{}m[{}]\t{}\x1B[0m\n", ($color as i32), $level, format_args!($($arg)*)))
    }
}

/// 宏定义中加 $ 和不加 $ 区别:
/// 加 $ 的 $crate 是去当前 crate(即 logger crate) 中寻找
/// 不加 $ 的 $crate 是去使用 logger 的那个 crate 中寻找
/// 比如 crate::LOG_LEVEL 是由外部 crate 自定义的, 用以控制打印层级
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if $crate::LogLevel::ERROR >= crate::LOG_LEVEL {
            $crate::log!(crate::logger::Color::Red, "ERROR", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if  $crate::LogLevel::WARN >= crate::LOG_LEVEL{
            $crate::log!(crate::logger::Color::Yellow, "WARN", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if $crate::LogLevel::INFO >= crate::LOG_LEVEL {
            $crate::log!(crate::logger::Color::Blue, "INFO", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::LogLevel::DEBUG >= crate::LOG_LEVEL {
            $crate::log!(crate::logger::Color::Green, "DEBUG", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if $crate::LogLevel::TRACE >= crate::LOG_LEVEL {
            $crate::log!(crate::logger::Color::Gray, "TRACE", $($arg)*)
        }
    }
}
