#![no_main]
#![no_std]

use core::{
    arch::asm,
    fmt::{self, Write},
};

pub extern crate qemu_config;
pub extern crate riscv;

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

pub fn logger_now() -> usize {
    riscv::register::time::read()
}

/// 通用打印
/// 类似样式
/// [        79 ms][INFO]   [kernel] Application exited with code 1
#[macro_export]
macro_rules! log {
    ($color:expr, $level:literal, $($arg:tt)*) => {
        $crate::logger_print(format_args!("\x1B[90m[{:10} ms]\x1B[0m\x1B[{}m[{}]\t[kernel] {}\x1B[0m\n",  $crate::logger_time_ms!(), ($color as i32), $level, format_args!($($arg)*)))
    }
}

#[macro_export]
macro_rules! logger_time_ms {
    () => {
        $crate::logger_now() / $crate::qemu_config::MILLI_UNIT
    };
}

#[macro_export]
macro_rules! logger_time_us {
    () => {
        $crate::logger_now() / $crate::qemu_config::MICRO_UNIT
    };
}

#[macro_export]
macro_rules! logger_time_s {
    () => {
        $crate::logger_now() / $crate::qemu_config::SECOND_UNIT
    };
}

/// 宏定义中加 $ 和不加 $ 区别:
/// 加 $ 的 $crate 是去当前 crate(即 logger crate) 中寻找
/// 不加 $ 的 $crate 是去使用 logger 的那个 crate 中寻找
/// 比如 crate::LOG_LEVEL 就是由外部 crate 自定义的, 用以控制打印层级, 所以不加 $
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
