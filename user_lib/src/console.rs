use core::fmt::{self, Write};

use crate::write;

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
