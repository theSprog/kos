use core::panic::PanicInfo;

use crate::console::*;
use crate::sbi::shutdown;

// 如果外部没有禁用 panic, 就定义 panic
#[cfg(not(feature = "disable_panic"))]
#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    // 如果有位置信息

    use logger::error;
    if let Some(location) = info.location() {
        error!(
            "Kernel panic at {}:{}\nDetail: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Kernel panic: {}", info.message().unwrap());
    }
    shutdown()
}
