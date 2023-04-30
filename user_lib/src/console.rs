use core::fmt::{self, Write};

use crate::write;

struct OUT;

// 标识符
const STDOUT: usize = 1;

impl Write for OUT {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    OUT.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[derive(Debug)]
pub(crate) enum Color {
    Red = 31,
    Yellow = 93,
    Blue = 34,
    Green = 32,
    Gray = 90,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LogLevel {
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
        $crate::console::print(format_args!("\x1B[{}m[{}]\t{}\x1B[0m\n", ($color as i32), $level, format_args!($($arg)*)))
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if crate::console::LogLevel::ERROR >= crate::LOG_LEVEL {
            crate::log!(crate::console::Color::Red, "ERROR", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if  crate::console::LogLevel::WARN >= crate::LOG_LEVEL{
            crate::log!(crate::console::Color::Yellow, "WARN", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if crate::console::LogLevel::INFO >= crate::LOG_LEVEL {
            crate::log!(crate::console::Color::Blue, "INFO", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if crate::console::LogLevel::DEBUG >= crate::LOG_LEVEL {
            crate::log!(crate::console::Color::Green, "DEBUG", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if crate::console::LogLevel::TRACE >= crate::LOG_LEVEL {
            crate::log!(crate::console::Color::Gray, "TRACE", $($arg)*)
        }
    }
}
