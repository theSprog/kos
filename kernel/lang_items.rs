use core::{arch::asm, panic::PanicInfo};

use crate::{process::processor, sbi::shutdown};

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

    unsafe { backtrace() }

    shutdown()
}

macro_rules! red {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[31m", $fmt, "\x1b[0m", "\n") $(, $($arg)+)?));
    }
}

unsafe fn backtrace() {
    let mut fp: usize;
    let stop = processor::api::current_kstack_top();
    asm!("mv {}, s0", out(reg) fp);
    red!("---START BACKTRACE---");
    for _ in 0..16 {
        if fp == stop {
            break;
        }
        red!("{:#x}", *((fp - 8) as *const usize));
        fp = *((fp - 16) as *const usize);
    }
    red!("---END   BACKTRACE---");
}
