pub mod pid;
pub mod processor;
pub mod scheduler;

mod stack;

use core::cell::{RefMut, UnsafeCell};

use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    memory::{address::*, address_space::AddressSpace, kernel_view::get_kernel_view},
    sync::unicore::UPSafeCell,
    task::{context::TaskContext, TaskStatus, TCB},
    trap::context::TrapContext,
    TRAP_CONTEXT,
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
    pub fn inner(&self) -> RefMut<'_, PCBInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn set_priority(&mut self, priority: u8) {
        self.inner.exclusive_access().priority = priority;
    }
    pub fn priority(&self) -> u8 {
        self.inner.exclusive_access().priority
    }

    pub fn new(elf_data: &[u8], prog_name: &str) -> Self {
        let kernel_view = get_kernel_view();

        // 分配 pid
        let pid = pid::api::pid_alloc();
        // 确定内核栈位置
        let kernel_stack = KernelStack::new(&pid);
        let tcb = TCB::new(elf_data, pid.0);

        // 每一个 pcb 默认优先级都是 100
        Self {
            pid,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(PCBInner::new_bare(tcb, 100, prog_name)) },
        }
    }

    pub fn fork(self: &Arc<PCB>) -> Arc<PCB> {
        // 访问父进程
        let mut parent_inner = self.inner();
        // 拷贝用户空间
        let address_space = AddressSpace::from_fork(&mut parent_inner.tcb.address_space);

        // 分配 pid 和 内核栈
        let pid_handle = pid::api::pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let trap_cx_ppn = address_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let new_tcb = TCB {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            address_space,
            trap_cx_ppn,
            base_size: parent_inner.tcb.base_size,
        };

        let pcb_inner = PCBInner {
            priority: parent_inner.priority, // 与父进程同优先级
            tcb: new_tcb,
            count: 0, // 新建进程所用时间片为 0
            // 父进程是 self, 没有子进程
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            prog_name: Some(String::from(parent_inner.prog_name())),
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
        let trap_cx = new_pcb.inner().trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        new_pcb

        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }

    pub fn exec(&self, elf_data: &[u8]) {
        todo!()
    }
}

pub struct PCBInner {
    tcb: TCB,

    prog_name: Option<String>,
    // 进程优先级, 0~255
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
    pub fn new_bare(tcb: TCB, priority: u8, prog_name: &str) -> Self {
        Self {
            priority,
            tcb,
            count: 0,
            parent: None,
            children: Vec::new(),
            prog_name: Some(String::from(prog_name)),
            exit_code: 0,
        }
    }

    pub fn tcb(&mut self) -> &'static mut TCB {
        let tcb = &mut self.tcb as *mut TCB;
        unsafe { tcb.as_mut().unwrap() }
    }

    pub fn prog_name(&self) -> &str {
        if let Some(prog_name) = &self.prog_name {
            return prog_name;
        }
        ""
    }

    pub fn status(&self) -> TaskStatus {
        self.tcb.task_status
    }
    pub fn set_status(&mut self, status: TaskStatus) {
        self.tcb.task_status = status;
    }
    pub fn inc_count(&mut self) {
        self.count += 1;
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
}
