use core::arch::asm;

use component::crt0::{self, Entry, Reader};
use component::memory::buddy::LockedHeap;
use logger::info;
use sys_interface::config::USER_HEAP_SIZE;

use crate::{exit, sbrk};

// 应用程序入口点
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    read_stack();
    clear_bss();
    let heap_start = sbrk(0);
    // 拨动堆顶指针
    sbrk(USER_HEAP_SIZE);
    HEAP.lock().init(heap_start, USER_HEAP_SIZE);
    let exit_code = main();
    // 进程退出后调用 exit
    exit(exit_code);

    // 应该不可达
    unreachable!()
}

fn read_stack() {
    unsafe {
        let mut fp: *const usize;
        asm!("mv {}, fp", out(reg) fp);
        // 测试是否得到内核放在栈上的数据
        let reader = Reader::from_ptr(fp);

        assert_eq!(reader.count(), 3);

        let mut reader_arg = reader.done();
        assert_eq!(reader_arg.next(), Some("cmd"));
        assert_eq!(reader_arg.next(), Some("args1"));
        assert_eq!(reader_arg.next(), Some("args2"));
        assert_eq!(reader_arg.next(), None);

        let mut reader_env = reader_arg.done();
        assert_eq!(reader_env.next(), Some("HOME=/root"));
        assert_eq!(reader_env.next(), None);

        let mut reader_aux = reader_env.done();
        assert_eq!(reader_aux.next(), Some(Entry::Gid(1000)));
        assert_eq!(reader_aux.next(), Some(Entry::Uid(1001)));
        assert_eq!(reader_aux.next(), Some(Entry::Platform("RISCV")));
        assert_eq!(reader_aux.next(), None);
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
