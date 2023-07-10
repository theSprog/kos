use core::arch::asm;

use component::memory::buddy::LockedHeap;
use sys_interface::config::USER_HEAP_SIZE;

use crate::{exit, sbrk};

pub static mut CRT0_SP: *const usize = core::ptr::null_mut();

// 应用程序入口点
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    read_crt0();
    clear_bss();
    let heap_start = sbrk(0);
    // 拨动堆顶指针
    sbrk(USER_HEAP_SIZE);
    HEAP.lock().init(heap_start, USER_HEAP_SIZE);
    let exit_code = main();

    // 刷新一行
    println!("");
    // 进程退出后调用 exit
    exit(exit_code);

    // 应该不可达
    unreachable!()
}

fn read_crt0() {
    unsafe {
        asm!("mv {}, fp", out(reg) CRT0_SP);
    }
}

// 定义弱符号 main, 如果用户没有定义 main 则会进入该函数
// 否则会进入用户定义的 main 中
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        // bss 起始处
        fn start_bss();
        // bss 结束处
        fn end_bss();
    }

    let start_bss = start_bss as usize;
    let end_bss = end_bss as usize;
    // 将 bss 清零
    unsafe {
        // 优化后的版本, 更快
        core::ptr::write_bytes(start_bss as *mut u8, 0, end_bss - start_bss);
    }
}

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
