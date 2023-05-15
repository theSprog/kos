use core::panic::PanicInfo;

use crate::sbi::shutdown;

#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    // 如果有位置信息

    use logger::error;

    if let Some(location) = info.location() {
        error!(
            "Kernel panic at {}:{} Detail:\n{}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Kernel panic: {}", info.message().unwrap());
    }
    shutdown()
}
