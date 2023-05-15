pub mod context;
pub mod switch;

use core::todo;

use crate::{
    loader::{get_app_data, get_num_app},
    memory::{
        address::*,
        address_space::{AddressSpace, MapPermission, KERNEL_SPACE},
        kernel_view::get_kernel_view,
    },
    sbi::shutdown,
    trap::{context::TrapContext, trap_handler},
    unicore::UPSafeCell,
    *,
};

use self::context::TaskContext;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskStatus {
    Ready,   // 准备运行
    Running, // 正在运行
    Died,    // 已退出
}

// Task Control Block, 任务控制块
pub struct TCB {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,

    pub address_space: AddressSpace, // 应用程序的地址空间
    pub trap_cx_ppn: PhysPageNum,    // 位于应用地址空间次高页的 Trap 上下文的物理页号
    pub base_size: usize, // base_size 统计了应用数据的大小，也就是在应用地址空间中从 0x0 开始到用户栈结束一共包含多少字节
}

impl TCB {
    pub fn new(elf_data: &[u8], app_id: usize) -> TCB {
        let kernel_view = get_kernel_view();
        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(elf_data);

        // 查询 TrapContext 的物理页号
        let trap_cx_ppn = address_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let task_status = TaskStatus::Ready;
        // 在内核空间中申请内核栈
        let (kernel_stack_bottom, kernel_stack_top) = kernel_view.kernel_stack_range(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_segment(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let tcb = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            address_space,
            trap_cx_ppn,
            base_size: user_sp,
        };

        // 为用户空间准备 TrapContext
        let trap_cx = tcb.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        tcb
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.address_space.token()
    }
}

struct TaskManagerInner {
    tasks: Vec<TCB>,
    current_task_idx: usize,
}
pub struct TaskManager {
    num_app: usize,                      // 所管理的 app 数量
    inner: UPSafeCell<TaskManagerInner>, // 只是内部可变性, 而非结构体可变
}

impl TaskManager {
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;
        inner.tasks[current].get_trap_cx()
    }
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

use alloc::vec::Vec;
use lazy_static::lazy_static;
use logger::info;
lazy_static! {
    pub(crate) static ref TASK_MANAGER: TaskManager = {
        info!("TASK_MANAGER initializing...");
        let num_app = get_num_app();
        info!("App number: {}", num_app);

        let mut tasks: Vec<TCB> = Vec::new();
        for i in 0..num_app {
            info!("App-{} is managing by TASK_MANAGER", i);
            tasks.push(TCB::new(get_app_data(i), i));
        }

        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task_idx: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn mark_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;
        // 让出 cpu
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_died(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;
        // 标记已死亡
        inner.tasks[current].task_status = TaskStatus::Died;
    }

    // 返回下一个 app_id (不存在返回 None)
    fn select_next(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;

        // 构造范围 range
        let next_start = current + 1;
        let next_end = next_start + self.num_app;

        // 在 [next_start, next_end) 区间查找
        // 有可能没有其他任务可调度, 从而再次调度当前(current)任务
        // 因为 (next_end - 1) % num_app = current
        for app_id in next_start..next_end {
            let app_id = app_id % self.num_app;
            if inner.tasks[app_id].task_status == TaskStatus::Ready {
                return Some(app_id);
            }
        }

        // 不存在下一个
        None
    }

    // 调度下一个 app 运行
    fn schedule(&self) {
        // 选出下一个 app
        if let Some(next) = self.select_next() {
            // 由于后面要修改 current_task_idx 所以需要 mut
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task_idx;

            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task_idx = next;

            // 下面的写法其实违背了可变与不可变借用规则, 但是我们转为指针来规避它
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut _;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const _;

            // 必须提前释放，不然下一个 app 在别处访问 self.inner 时由于此处没有释放会导致 borrowMut Error
            drop(inner);

            // 开始切换现场
            self.switch_to(current_task_cx_ptr, next_task_cx_ptr);

            // 除了第一个 switch_to(由 start_app 函数调用), 之后的 switch_to 将会回到此处, 从而回到用户态
            // 上下文切换完毕，回到 user mode
        } else {
            info!("All applications completed!");
            shutdown();
        }
    }

    fn start(&self) -> ! {
        info!("Now we starting app(s)!");
        todo!("prepare to continue");
        let mut inner = self.inner.exclusive_access();
        assert!(!inner.tasks.is_empty());

        // 从第 0 个任务开始
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let app_ctx = &task0.task_cx as *const TaskContext;
        drop(inner);

        self.start_app(app_ctx);
    }

    fn start_app(&self, app_ctx: *const TaskContext) -> ! {
        let mut _unused = TaskContext::default();
        // 从一个无用的 _unused 切换到 app_ctx
        // 第一次 switch_to 会跳转到 __restore 从而直接进入用户态
        self.switch_to(&mut _unused, app_ctx);

        // 不可能再回到此处，因为 select_next 不可能选到 _unused (它都没有注册)
        unreachable!()
    }

    /// switch_to 内部是一段汇编,
    /// 它会将当前现场保存至 current_task_cx_ptr 结构体中, 因此需要 mut
    /// 而目标现场只需要读取其内容到 CPU 上, 所以可以 const
    fn switch_to(
        &self,
        current_task_ctx_ptr: *mut TaskContext,
        next_task_ctx_ptr: *const TaskContext,
    ) {
        unsafe { crate::task::switch::__switch(current_task_ctx_ptr, next_task_ctx_ptr) }
    }
}

// 公有接口
pub fn start() {
    TASK_MANAGER.start();
}

pub fn suspend_and_run_next() {
    TASK_MANAGER.mark_suspended();
    TASK_MANAGER.schedule();
}

pub fn exit_and_run_next() {
    TASK_MANAGER.mark_died();
    TASK_MANAGER.schedule();
}
