use core::panic::PanicInfo;

use crate::console::*;
use crate::sbi::shutdown;
use crate::{debug, error, info, trace, warn};

// 如果外部没有禁用 panic, 就定义 panic
#[cfg(not(feature = "disable_panic"))]
#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    // 如果有位置信息
    if let Some(location) = info.location() {
        error!(
            "Kernel Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Kernel Panicked: {}", info.message().unwrap());
    }
    shutdown()
}
