pub mod context;
pub mod stack;
pub mod switch;

use crate::{
    config::*,
    debug, info,
    loader::{get_num_app, init_app_ctx},
    sbi::shutdown,
    unicore::UPSafeCell,
};

use self::{context::TaskContext, stack::*};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskStatus {
    UnInit,  // 未初始化
    Ready,   // 准备运行
    Running, // 正在运行
    Died,    // 已退出
}

// Task Control Block, 任务控制块
#[derive(Clone, Copy, Debug)]
pub struct TCB {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub user_stack: Option<&'static UserStack>,
    pub kernel_stack: Option<&'static KernelStack>,
}

impl TCB {
    pub fn new() -> TCB {
        TCB {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::default(),
            user_stack: None,
            kernel_stack: None,
        }
    }
}

#[derive(Clone, Copy)]
struct TaskManagerInner {
    tasks: [TCB; MAX_APP_NUM],
    current_task_idx: usize,
}
pub struct TaskManager {
    num_app: usize,                      // 所管理的 app 数量
    inner: UPSafeCell<TaskManagerInner>, // 只是内部可变性, 而非结构体可变
}

use lazy_static::lazy_static;
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        info!("APP_NUM: {}", num_app);

        {
            let kernel_stack_start = KERNEL_STACKS[0].data.as_ptr() as usize;
            let kernel_stack_end = KERNEL_STACKS.last().unwrap().get_sp();
            let kernel_stack_size = kernel_stack_end - kernel_stack_start;

            debug!("Kernel-Stacks Address:\t [0x{:x}..0x{:x}), single_size:0x{:x}, num:{}, total_size: 0x{:x}",
            kernel_stack_start,
            kernel_stack_end,
            kernel_stack_size / MAX_APP_NUM,
            MAX_APP_NUM,
            kernel_stack_size);

            let user_stack_start = USER_STACKS[0].data.as_ptr() as usize;
            let user_stack_end = USER_STACKS.last().unwrap().get_sp();
            let user_stack_size = user_stack_end - user_stack_start;

            debug!("User-Stacks Address:\t [0x{:x}..0x{:x}), single_size:0x{:x}, num:{}, total_size: 0x{:x}",
            user_stack_start,
            user_stack_end,
            user_stack_size / MAX_APP_NUM,
            MAX_APP_NUM,
            user_stack_size);
        }


        let mut tasks = [TCB::new(); MAX_APP_NUM];


        // 初始化, 但只初始化前 num_app 个
        tasks.iter_mut().take(num_app).enumerate().for_each(|task_pack| {
            let (app_id, task) = task_pack;
            info!("Init app {}", app_id);
            task.kernel_stack = Some(&KERNEL_STACKS[app_id]);
            task.user_stack = Some(&USER_STACKS[app_id]);
            // 将 ra 设置为 __restore 地址, 返回时 jmp 到该地方开始回到用户态
            task.task_cx = TaskContext::goto_restore(init_app_ctx(task, app_id));
            task.task_status = TaskStatus::Ready;
        });

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
    fn print_task_info(&self) {
        for (app_id, task) in self
            .inner
            .exclusive_access()
            .tasks
            .iter()
            .take(get_num_app())
            .enumerate()
        {
            debug!(
                "app_id: {}, task_status: {:?}, task_cx: {:#x?}",
                app_id, task.task_status, task.task_cx
            );
        }
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;
        // 让出 cpu
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task_idx;
        // 标记已死亡
        inner.tasks[current].task_status = TaskStatus::Died;
    }

    // return next app_id if exists
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

        None
    }

    fn run_next_task(&self) {
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
            // use crate::board::QEMUExit;
            // crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
    }

    fn start(&self) -> ! {
        info!("Now we starting app(s)!");

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
    TASK_MANAGER.print_task_info();
    TASK_MANAGER.start();
}

pub fn suspend_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}
