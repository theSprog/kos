pub mod pid;
pub mod processor;
pub mod scheduler;

mod stack;

use core::cell::RefMut;

use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    loader::load_app,
    memory::{
        address_space::{AddressSpace, KERNEL_SPACE},
        kernel_view::get_kernel_view,
    },
    sync::unicore::UPSafeCell,
    task::{context::TaskContext, TaskStatus, TCB},
    trap::{context::TrapContext, trap_handler},
};

use self::{pid::Pid, stack::KernelStack};

pub struct PCB {
    // 在初始化之后就不再变化的元数据
    // pid 进程唯一标识符
    pub pid: Pid,
    // KernelStack 只是一个 pid, 目的是 RAII, PCB 析构时自动释放内核栈资源
    pub kernel_stack: KernelStack,

    // 在运行过程中可能发生变化的元数据
    inner: UPSafeCell<PCBInner>,
}

impl PCB {
    pub fn ex_inner(&self) -> RefMut<'_, PCBInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn set_priority(&mut self, priority: u8) {
        self.ex_inner().set_priority(priority);
    }
    pub fn priority(&self) -> u8 {
        self.ex_inner().priority()
    }

    pub fn new(elf_data: &[u8], cmd: &str) -> Self {
        let kernel_view = get_kernel_view();

        // 分配 pid
        let pid = pid::api::pid_alloc();
        // 确定内核栈位置
        let kernel_stack = KernelStack::new(&pid);
        let tcb = TCB::new_once(elf_data, pid.0);

        // init 默认优先级是 3, 或者继承自父优先级
        Self {
            pid,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(PCBInner::new_bare(tcb, 3, cmd)) },
        }
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // 访问父进程
        let mut parent_inner = self.ex_inner();
        // 拷贝用户空间
        let address_space = AddressSpace::from_fork(&mut parent_inner.tcb.address_space);

        // 分配 pid 和 内核栈
        let pid_handle = pid::api::pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let trap_cx_ppn = address_space.trap_ppn();

        let new_tcb = TCB {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            address_space,
            trap_cx_ppn,
            base_size: parent_inner.tcb.base_size,
            fd_table: parent_inner.tcb.fd_table.clone(),
        };

        let pcb_inner = PCBInner {
            priority: parent_inner.priority, // 与父进程同优先级
            tcb: new_tcb,
            count: 0, // 新建进程所用时间片为 0
            // 父进程是 self, 没有子进程
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            cmd: String::from(parent_inner.cmd()),
            exit_code: 0,
        };

        let new_pcb = Arc::new(PCB {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(pcb_inner) },
        });

        // add child
        parent_inner.children.push(new_pcb.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = new_pcb.ex_inner().trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        new_pcb

        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }

    pub fn exec(&self, app_name: &str, args: Vec<String>, envs: Vec<String>) -> isize {
        // self 即子进程自身
        let pid = processor::api::current_pid();
        let app = load_app(app_name);
        if app.is_none() {
            return -1;
        }

        let elf_data = app.unwrap();

        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(elf_data, pid);
        let trap_cx_ppn = address_space.trap_ppn();

        // **** access inner exclusively
        let mut inner = self.ex_inner();
        // 替换地址空间, 原来的地址空间全部被回收, 页表也更换了
        inner.tcb.address_space = address_space;
        // 更新 trap_cx ppn
        inner.tcb.trap_cx_ppn = trap_cx_ppn;
        // 更新 base_size
        inner.tcb.base_size = user_sp;
        // 更新名称
        inner.cmd = String::from(app_name);

        // 取出进程的 trap_cx 并更新
        let trap_cx = inner.trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(), // 复用子进程自身的 kernel_stack
            trap_handler as usize,
            pid,
        );

        // 压入 crt0 栈
        inner.tcb.address_space.push_crt0(trap_cx, &args, &envs);

        // 注意: 在执行 execve() 后:
        // 子进程的程序映像被替换为新的程序，但是文件描述符表不会受到影响，仍然保持不变。
        // 因此，子进程在 execve() 执行后会继续使用父进程继承的文件描述符。

        // 如果希望子进程在 execve() 执行后关闭或重定向文件描述符，
        // 可以在调用 execve() 之前手动关闭或重定向这些文件描述符
        0
    }
}

pub struct PCBInner {
    tcb: TCB,

    cmd: String,
    // 进程优先级, 1~5
    // 有些调度算法不会关注优先级, 例如 FIFO
    priority: u8,

    // 进程运行的时间片, 每用一个 +1
    count: usize,

    // 树形结构, 父子进程, 父进程有多个子进程指向它
    // weak 智能指针将不会影响父进程的引用计数
    parent: Option<Weak<PCB>>,
    children: Vec<Arc<PCB>>,

    // 退出码
    exit_code: i32,
}

impl PCBInner {
    pub fn new_bare(tcb: TCB, priority: u8, cmd: &str) -> Self {
        assert!((1..=5).contains(&priority)); // 1-5 优先级
        Self {
            priority,
            tcb,
            count: 0,
            parent: None,
            children: Vec::new(),
            cmd: String::from(cmd),
            exit_code: 0,
        }
    }

    pub fn tcb(&mut self) -> &'static mut TCB {
        let tcb = &mut self.tcb as *mut TCB;
        unsafe { tcb.as_mut().unwrap() }
    }

    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: u8) {
        assert!((1..=5).contains(&priority));
        self.priority = priority
    }

    pub fn status(&self) -> TaskStatus {
        self.tcb.task_status
    }
    pub fn set_status(&mut self, status: TaskStatus) {
        self.tcb.task_status = status;
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
    pub fn inc_count(&mut self) {
        // 60 是 1-5 的最小公倍数
        self.count += 60;
    }
    pub fn is_zombie(&self) -> bool {
        self.status() == TaskStatus::Zombie
    }

    pub fn user_token(&self) -> usize {
        self.tcb.address_space.token()
    }
    pub fn trap_cx(&self) -> &'static mut TrapContext {
        self.tcb.trap_cx_ppn.get_mut()
    }
    pub fn task_cx(&mut self) -> &'static mut TaskContext {
        let ctx = &mut self.tcb.task_cx as *mut TaskContext;
        unsafe { ctx.as_mut().unwrap() }
    }

    pub fn children_mut(&mut self) -> &mut Vec<Arc<PCB>> {
        &mut self.children
    }

    pub fn children(&self) -> &Vec<Arc<PCB>> {
        &self.children
    }
}
