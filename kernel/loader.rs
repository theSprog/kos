use core::arch::asm;

use logger::{debug, info};

use crate::{unicore::UPSafeCell, *};

extern "C" {
    pub fn _num_app();
}

pub fn get_num_app() -> usize {
    // 读取指针处对应的值，使用 ptr::read_volatile
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

pub fn load_app(app_id: usize) -> &'static [u8] {
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();

    // 之所以要 num_app+1 是因为最后还有个 app_??_end 也要占用空间, 用来表示结束
    // 从首个 usize 之后的地方开始读 app 数据, 这就是为什么要 add(1) 跳过一个 usize
    let app_start_addr = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start_addr[app_id] as *const u8,
            app_start_addr[app_id + 1] - app_start_addr[app_id],
        )
    }
}
