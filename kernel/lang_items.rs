use core::{arch::asm, panic::PanicInfo};

use logger::info;

use crate::sbi::shutdown;

#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    use logger::error;

    // 如果有位置信息
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

fn print_stack_trace() -> () {
    unsafe {
        let mut fp: *const usize;
        asm!("mv {}, fp", out(reg) fp);

        info!("== Begin stack trace ==");
        while fp != core::ptr::null() {
            let saved_ra = *fp.sub(1);
            let saved_fp = *fp.sub(2);

            info!("0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);

            fp = saved_fp as *const usize;
        }
        info!("== End stack trace ==");
    }
}
