use crate::sbi::shutdown;
use crate::trap::context::TrapContext;
use crate::unicore::UPSafeCell;
use crate::{debug, info, println};
use core::arch::asm;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use lazy_static::lazy_static;
use riscv::register::sstatus;

// 用户栈大小, 8K
const USER_STACK_SIZE: usize = 4096 * 2;
// 内核栈大小, 8K
const KERNEL_STACK_SIZE: usize = 4096 * 2;
// 最多允许 16 个 app
const MAX_APP_NUM: usize = 16;
// 起始基地址
const APP_BASE_ADDRESS: usize = 0x80400000;
// 每个 app 的 size 上限, 128K
const APP_SIZE_LIMIT: usize = 0x20000;

// 内核栈, .bss 段中
#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

// 用户程序栈, .bss 段中
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        // 预留栈空间
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }

    // 获取栈顶地址, 即数组结尾
    fn get_sp(&self) -> usize {
        KERNEL_STACK_SIZE + self.data.as_ptr() as usize
    }
}

impl UserStack {
    // 获取栈顶地址, 即数组结尾
    fn get_sp(&self) -> usize {
        USER_STACK_SIZE + self.data.as_ptr() as usize
    }
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

struct AppManager {
    // app 数量
    num_app: usize,
    // 当前正在执行的 app 数量
    current_app: usize,
    // 每个 app 的起始地址, 最后一个 usize 代表 app_end 地址
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            // 如果已经处理完毕
            info!("All application(s) completed, shutdown!");
            shutdown();
        }
        info!("[kernel] Loading app_{}", app_id);
        // 清空 app 地址空间
        from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        // app 源地址数据
        let app_src = from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        // app 目的地址
        let app_dst = from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        // 将 app 从源地址搬运到目的地址
        app_dst.copy_from_slice(app_src);
        asm!("fence.i");
    }

    pub fn print_app_info(&self) {
        debug!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            debug!(
                // app 地址是一个左闭右开的区间
                "[kernel] app_{} [ {:#x}, {:#x} )",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

// run apps
pub fn run_apps() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        // jmp 到 APP_BASE_ADDRESS 执行
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}

pub(crate) fn init() {
    print_app_info();
}
