use core::panic::PanicInfo;

use crate::console::*;
use crate::sbi::shutdown;
use crate::{debug, error, info, trace, warn};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 如果有位置信息
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Panicked: {}", info.message().unwrap());
    }
    shutdown()
}
