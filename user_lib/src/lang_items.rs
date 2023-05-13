use crate::syscall::sys_exit;

macro_rules! error {
    ($($arg:tt)*)=> {
        $crate::console::print(format_args!("\x1B[31m[ERROR] {}\x1B[0m\n", format_args!($($arg)*)))
    };
}

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err_msg = panic_info.message().unwrap();
    if let Some(location) = panic_info.location() {
        error!(
            "panic at {}:{} {}",
            location.file(),
            location.line(),
            err_msg
        )
    } else {
        error!("panic: {}", err_msg);
    }
    // 一旦发生 panic exit 直接退出
    sys_exit(1);
    // 最后的防线，如果无法退出就自旋
    loop {}
}
