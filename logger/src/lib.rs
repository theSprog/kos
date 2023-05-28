#![no_main]
#![no_std]
#![allow(clippy::crate_in_macro_def)]

use core::{
    arch::asm,
    fmt::{self, Write},
};

// pub extern crate qemu_config;
pub extern crate riscv;

// kernel 专属
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

// 用户态专属
#[inline(always)]
fn logger_syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

struct Console;

#[allow(dead_code)]
impl Console {
    fn kernel_console_write(&mut self, s: &str) {
        s.chars().for_each(|c: char| {
            const SBI_CONSOLE_PUTCHAR: usize = 1;
            logger_sbi_call(SBI_CONSOLE_PUTCHAR, c as usize, 0, 0);
        });
    }

    fn user_console_write(&mut self, s: &str) {
        const SYS_STDOUT: usize = 1;
        // 见 sys_interface 接口定义
        logger_syscall(SYSCALL_WRITE, [SYS_STDOUT, s.as_ptr() as usize, s.len()]);
    }
}

impl Write for Console {
    #[allow(unused_variables)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        #[cfg(feature = "kernel")]
        {
            self.kernel_console_write(s)
        }

        #[cfg(feature = "user")]
        {
            self.user_console_write(s);
        }

        Ok(())
    }
}

pub fn logger_print(args: fmt::Arguments) {
    // write_fmt 最终调用 write_str
    Console.write_fmt(args).unwrap();
}

use spin::Mutex;
use sys_interface::syscall::SYSCALL_WRITE;
static LOGGER_LOCK: Mutex<()> = Mutex::new(());
#[allow(unused_variables)]
pub fn print(color: i32, level: &'static str, args: fmt::Arguments) {
    let _lock = LOGGER_LOCK.lock();
    #[cfg(feature = "kernel")]
    {
        logger_print(format_args!(
            "\x1B[90m[{:10} ms]\x1B[0m\x1B[{}m[{}]\t[kernel] {}\x1B[0m\n",
            crate::logger_time_ms!(),
            color,
            level,
            args
        ))
    }

    #[cfg(feature = "user")]
    {
        logger_print(format_args!(
            "\x1B[{}m[{}] [user] {}\x1B[0m\n",
            color, level, args
        ))
    }
}

#[derive(Debug)]
pub enum Color {
    Red = 31,
    Yellow = 93,
    Purple = 35,
    Green = 32,
    Blue = 94,
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

#[macro_export]
macro_rules! logger_time_ms {
    () => {
        $crate::logger_now() / qemu_config::MILLI_UNIT
    };
}

#[macro_export]
macro_rules! logger_time_us {
    () => {
        $crate::logger_now() / qemu_config::MICRO_UNIT
    };
}

#[macro_export]
macro_rules! logger_time_s {
    () => {
        $crate::logger_now() / qemu_config::SECOND_UNIT
    };
}

/// 通用打印
/// 类似样式
/// [        79 ms][INFO]   [kernel] Application exited with code 1
#[macro_export]
macro_rules! log {
    ($color:expr, $level:literal, $($arg:tt)*) => {
        $crate::print(($color as i32), $level, format_args!($($arg)*))
    }
}

/// 宏定义中加 $ 和不加 $ 区别:
/// 加 $ 的 $crate 是去当前 crate(即 logger crate) 中寻找
/// 不加 $ 的 $crate 是去使用 logger 的那个 crate 中寻找
/// 比如 crate::LOG_LEVEL 就是由外部 crate 自定义的, 用以控制打印层级, 所以不加 $
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if $crate::LogLevel::ERROR >= crate::LOG_LEVEL {
            $crate::log!($crate::Color::Red, "ERROR", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if  $crate::LogLevel::WARN >= crate::LOG_LEVEL{
            $crate::log!($crate::Color::Yellow, "WARN", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if $crate::LogLevel::INFO >= crate::LOG_LEVEL {
            $crate::log!($crate::Color::Purple, "INFO", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::LogLevel::DEBUG >= crate::LOG_LEVEL {
            $crate::log!($crate::Color::Green, "DEBUG", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if $crate::LogLevel::TRACE >= crate::LOG_LEVEL {
            $crate::log!($crate::Color::Blue, "TRACE", $($arg)*)
        }
    }
}
