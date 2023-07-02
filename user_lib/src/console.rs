use core::fmt::{self, Write};

use crate::{read, write};

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 标识符
        const STDOUT: usize = 1;
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

use spin::Mutex;
static LOGGER_LOCK: Mutex<()> = Mutex::new(());
pub fn print(args: fmt::Arguments) {
    let _guard = LOGGER_LOCK.lock();
    Console.write_fmt(args).unwrap();
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

#[macro_export]
macro_rules! red {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[31m", $fmt, "\x1b[0m", "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! green {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[32m", $fmt, "\x1b[0m", "\n") $(, $($arg)+)?));
    }
}

pub fn getchar() -> u8 {
    const STDIN: usize = 0;
    let mut c = [0u8; 1];
    let recv = read(STDIN, &mut c);
    assert_eq!(recv, 1);
    c[0]
}
