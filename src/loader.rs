use core::arch::asm;

use crate::{debug, info, sbi::shutdown, trap::context::TrapContext, unicore::UPSafeCell};

// 用户栈大小, 8K
pub const USER_STACK_SIZE: usize = 4096 * 2;
// 内核栈大小, 8K
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
// 最多允许 16 个 app
pub const MAX_APP_NUM: usize = 16;
// 起始基地址
pub const BASE_ADDRESS: usize = 0x80400000;
// 每个 app 的 size 上限, 128K
pub const APP_SIZE_LIMIT: usize = 0x20000;

// 内核栈, .bss 段中
#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct KernelStack {
    pub(crate) data: [u8; KERNEL_STACK_SIZE],
}
impl KernelStack {
    pub fn new() -> KernelStack {
        KernelStack {
            data: [0; KERNEL_STACK_SIZE],
        }
    }

    pub fn push_context(&self, cx: TrapContext) -> usize {
        // 预留栈空间
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        cx_ptr as usize
    }

    // 获取栈顶地址, 即数组结尾
    pub fn get_sp(&self) -> usize {
        KERNEL_STACK_SIZE + self.data.as_ptr() as usize
    }
}
// 用户程序栈, .bss 段中
#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack {
    pub(crate) data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    pub fn new() -> UserStack {
        UserStack {
            data: [0; USER_STACK_SIZE],
        }
    }
    // 获取栈顶地址, 即数组结尾
    pub fn get_sp(&self) -> usize {
        USER_STACK_SIZE + self.data.as_ptr() as usize
    }
}

pub static KERNEL_STACKS: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

pub static USER_STACKS: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

// 获取 app 的内存起始地址
#[inline]
pub fn get_app_base(app_id: usize) -> usize {
    BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    // 读取指针处对应的值，使用 ptr::read_volatile
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

// 将 app_id 初始化并且返回 context 地址
pub fn init_app_ctx(app_id: usize) -> usize {
    KERNEL_STACKS[app_id].push_context(TrapContext::app_init_context(
        get_app_base(app_id),
        USER_STACKS[app_id].get_sp(),
    ))
}

pub struct AppManager {
    // app 数量
    pub(crate) num_app: usize,
    // 当前正在执行的 app 数量
    pub(crate) current_app: usize,
    // 每个 app 的起始地址, 最后一个 usize 代表 app_end 地址
    pub(crate) app_start: [usize; MAX_APP_NUM + 1],
}

// 只有在 AppManager 第一次被使用到的时候，才会进行实际的初始化工作
lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        // new 是 unsafe 操作
        UPSafeCell::new({
            extern "C" {
                // 找到外部符号 _num_app
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            // link_app.S 中的一个 .quad 是一个 usize 宽
            let num_app = num_app_ptr.read_volatile();  // 首个 usize 代表 app 个数
            // 之所以要 +1 是因为最后还有个 app_??_end 也要占用空间
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            // 从首个 usize 之后的地方开始读 app 数据
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

impl AppManager {
    fn print_app_info(&self) {
        debug!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            debug!(
                // 我们暂时使用内存模拟硬盘
                // app 硬盘地址是一个左闭右开的区间
                "[kernel] hard-disk address: app-{} [ {:#x}, {:#x} )",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    // batch 批处理形式加载程序
    // 将指定 app_id 的应用程序加载到 [start..start+len) 这块地址上
    // 这需要保证 源app文件 在链接时也指定自己应该放进这块地址
    unsafe fn load_app(&self, app_id: usize, start: usize, len: usize) {
        info!(
            "[kernel] Loading app-{} into [{:#x}..{:#x})",
            app_id,
            start,
            start + len
        );
        use core::slice::{from_raw_parts, from_raw_parts_mut};

        // 清空 app 地址空间
        from_raw_parts_mut(start as *mut u8, len).fill(0);

        // 硬盘上 app 的起始地址和长度
        let hd_app_start = self.app_start[app_id] as *const u8;
        let hd_app_len = self.app_start[app_id + 1] - self.app_start[app_id];

        // 由于只分配了长度为 len 的内存，所以需要检查长度，不能大于 len
        assert!(
            hd_app_len <= len,
            "APP-{}: too big app(size:{:#x}), but just allocated {:#x} memory",
            app_id,
            hd_app_len,
            len
        );
        // app 源地址数据
        let app_src = from_raw_parts(hd_app_start, hd_app_len);
        // app 目的地址
        let app_dst = from_raw_parts_mut(start as *mut u8, app_src.len());
        // 将 app 从源地址搬运到目的地址
        app_dst.copy_from_slice(app_src);
        asm!("fence.i");
    }

    // 一次性加载所有程序
    pub fn load_apps(&self) {
        // 加载所有 app
        for app_id in 0..self.num_app {
            let app_base = BASE_ADDRESS + app_id * APP_SIZE_LIMIT;
            unsafe {
                self.load_app(app_id, app_base, APP_SIZE_LIMIT);
            }
        }
    }
}

pub fn init() {
    info!("Loader Initialization");
    APP_MANAGER.exclusive_access().print_app_info();
    APP_MANAGER.exclusive_access().load_apps();
}
