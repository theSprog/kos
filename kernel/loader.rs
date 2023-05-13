use core::arch::asm;

use logger::{debug, info};

use crate::{sbi::shutdown, task::TCB, trap::context::TrapContext, unicore::UPSafeCell, *};

// 获取 app 对应的内存起始地址
#[inline]
pub fn get_app_base(app_id: usize) -> usize {
    USER_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    // 读取指针处对应的值，使用 ptr::read_volatile
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

// 将 app_id 初始化并且返回 context 地址
pub fn init_app_ctx(tcb: &TCB, app_id: usize) -> usize {
    if let (Some(kernel_stack), Some(user_stack)) = (tcb.kernel_stack, tcb.user_stack) {
        return kernel_stack.push_context(TrapContext::app_init_context(
            get_app_base(app_id),
            user_stack.get_sp(),
        ));
    }

    panic!("kernel_stack or user_stack is not initialized!");
}

pub struct AppManager {
    // app 数量
    pub(crate) num_apps: usize,
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
            let num_apps = num_app_ptr.read_volatile();  // 首个 usize 代表 app 个数
            // 之所以要 +1 是因为最后还有个 app_??_end 也要占用空间
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];

            // 从首个 usize 之后的地方开始读 app 数据
            // 这就是为什么要 add(1) 跳过一个 usize
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_apps + 1);
            app_start[..=num_apps].copy_from_slice(app_start_raw);
            AppManager {
                num_apps,
                current_app: 0,
                app_start,
            }
        })
    };
}

impl AppManager {
    fn print_app_info(&self) {
        debug!("num_app = {}", self.num_apps);
        for i in 0..self.num_apps {
            debug!(
                // 我们暂时使用内存模拟硬盘
                // app 硬盘地址是一个左闭右开的区间
                "hard-disk address: app-{} [{:#x}, {:#x}), size: 0x{:x}",
                i,
                self.app_start[i],
                self.app_start[i + 1],
                self.app_start[i + 1] - self.app_start[i]
            );
        }
    }

    // 一次性加载所有程序
    pub fn load_apps(&self) {
        assert!(
            self.num_apps <= MAX_APP_NUM,
            "Too many apps, there are {} slots",
            MAX_APP_NUM
        );
        // 加载所有 app
        for app_id in 0..self.num_apps {
            let app_base = USER_BASE_ADDRESS + app_id * APP_SIZE_LIMIT;
            unsafe {
                self.load_app(app_id, app_base, APP_SIZE_LIMIT);
            }
        }
    }
    // batch 批处理形式加载程序
    // 将指定 app_id 的应用程序加载到 [start..start+len) 这块地址上
    // 这需要保证 源app文件 在链接时也指定自己应该放进这块地址
    unsafe fn load_app(&self, app_id: usize, start: usize, len: usize) {
        info!(
            "Loading app-{} into [{:#x}..{:#x})",
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
}

pub fn init() {
    info!("Loader initalizing");
    APP_MANAGER.exclusive_access().print_app_info();
    APP_MANAGER.exclusive_access().load_apps();
    info!("App(s) loaded successfully")
}
