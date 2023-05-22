use core::arch::asm;

use crate::{sync::unicore::UPSafeCell, *};
use alloc::vec::Vec;
use logger::{debug, info, trace};

extern "C" {
    pub fn _num_app();
    pub fn _app_names();
}

pub fn get_num_app() -> usize {
    // 读取指针处对应的值，使用 ptr::read_volatile
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

lazy_static! {
    pub static ref APP_CONTAINER: Vec<&'static str> = {
        info!("APP_CONTAINER initializing...");
        let num_app_ptr = _num_app as usize as *const usize;
        let num_app = get_num_app();

        let start = _app_names as usize as *const u8;
        let apps = get_app_names(num_app, start);

        debug!("avaliable apps: {:?}", apps);
        apps
    };
}

pub fn init() {
    assert!(APP_CONTAINER.len() > 0, "There must be at least one app!");
}

pub fn load_app(app_name: &str) -> &'static [u8] {
    if let Some(data) = get_app_data_by_name(app_name) {
        return data;
    }
    panic!("failed to find app '{app_name}'");
}

fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    // 我们假设 app 的 name 声明与存放的序关系是一致的
    // 例如首先声明 "app1", 那么地址处也是首先存放 app1 的数据
    trace!("extracting app data from '{}'", name);
    let num_app = get_num_app();
    (0..num_app)
        .find(|&i| APP_CONTAINER[i] == name)
        .map(|i| get_app_data_by_id(i + 1))
}

fn get_app_data_by_id(pid: usize) -> &'static [u8] {
    // // 之所以要 num_app+1 是因为最后还有个 app_??_end 也要占用空间, 用来表示结束
    // // 从首个 usize 之后的地方开始读 app 数据, 这就是为什么要 add(1) 跳过一个 usize
    assert!(pid > 0);
    // 之所以要 -1 是因为 0 已经被 idle 占据, 第一个用户进程从 1 开始, 但是却在第 0 个位置
    let app_id = pid - 1;
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}

fn get_app_names(num_app: usize, mut start: *const u8) -> Vec<&'static str> {
    let mut apps = Vec::new();

    unsafe {
        for _ in 0..num_app {
            let mut end = start;
            while end.read_volatile() != '\0' as u8 {
                end = end.add(1);
            }
            let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
            let str = core::str::from_utf8(slice).unwrap();
            apps.push(str);

            // 跳过 \0
            start = end.add(1);
        }
    }
    apps
}
