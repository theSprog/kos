use core::fmt::{self, Write};

use crate::write;

struct OUT;

// 标识符
const STDOUT: usize = 1;

impl Write for OUT {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 通过内核系统调用打印
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
