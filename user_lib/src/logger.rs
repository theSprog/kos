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
        if crate::logger::LogLevel::ERROR >= crate::LOG_LEVEL {
            crate::log!(crate::logger::Color::Red, "ERROR", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if  crate::logger::LogLevel::WARN >= crate::LOG_LEVEL{
            crate::log!(crate::logger::Color::Yellow, "WARN", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if crate::logger::LogLevel::INFO >= crate::LOG_LEVEL {
            crate::log!(crate::logger::Color::Blue, "INFO", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if crate::logger::LogLevel::DEBUG >= crate::LOG_LEVEL {
            crate::log!(crate::logger::Color::Green, "DEBUG", $($arg)*)
        }
    }
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if crate::logger::LogLevel::TRACE >= crate::LOG_LEVEL {
            crate::log!(crate::logger::Color::Gray, "TRACE", $($arg)*)
        }
    }
}
