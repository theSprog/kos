use crate::syscall::sys_exit;
use logger::error;

#[allow(unreachable_code)]
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
