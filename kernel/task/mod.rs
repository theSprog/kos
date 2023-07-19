pub mod context;
pub mod switch;
pub mod uresource;

use crate::loader::load_app;
use crate::process::{scheduler, PCB};

use crate::sync::unicore::{UPIntrFreeCell, UPIntrRefMut};
use crate::{memory::address::*, trap::context::TrapContext, *};

use self::context::TaskContext;
use self::uresource::TCBUserResource;

use alloc::sync::{Arc, Weak};
use logger::{debug, info};
use process::kstack::KernelStack;

// INIT 进程名称
pub const INIT: &str = "init";

lazy_static! {
    /// init 进程
    pub static ref INITPROC: Arc<PCB> =
    {
        info!("'{INIT}' process initializing...");
        if let Some(init_data) = load_app(INIT) {
           PCB::new_once(&init_data, INIT)
        }else {
            panic!("Failed to find '{INIT}' app");
        }
    };
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskStatus {
    Ready,   // 就绪态
    Running, // 正在运行
    Died,    // 已退出
    Blocked, // 阻塞

    Interruptable,   // 可中断睡眠
    Uninterruptable, // 不可中断睡眠
}

#[allow(clippy::upper_case_acronyms)]
pub struct TCB {
    pcb: Weak<PCB>,          // TCB 所属的进程
    pub kstack: KernelStack, //任务（线程）的内核栈

    // 内部可变性
    inner: UPIntrFreeCell<TCBInner>,
}

impl TCB {
    pub fn new(pcb: &Arc<PCB>, ustack_base: usize, alloc_uresource: bool) -> Self {
        debug!(
            "new tcb for pid={}, alloc_uresource={}",
            pcb.pid(),
            alloc_uresource
        );

        let resource = TCBUserResource::new(pcb.clone(), ustack_base, alloc_uresource);
        // 查询 TrapContext 的物理页号
        let trap_ctx_ppn = resource.trap_ctx_ppn_ex();

        // 为该线程分配内核栈
        let kstack = KernelStack::alloc();
        let kstack_top = kstack.get_top();

        Self {
            pcb: Arc::downgrade(pcb),
            kstack,
            inner: unsafe {
                UPIntrFreeCell::new(TCBInner {
                    resource: Some(resource),
                    trap_ctx_ppn,
                    task_ctx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    priority: 3,
                    count: 0,
                })
            },
        }
    }

    pub fn ex_inner(&self) -> UPIntrRefMut<'_, TCBInner> {
        self.inner.exclusive_access()
    }

    pub fn ustack_base(&self) -> usize {
        self.ex_inner().ustack_base()
    }

    pub fn pid(&self) -> usize {
        self.pcb().unwrap().pid()
    }

    pub fn pcb(&self) -> Option<Arc<PCB>> {
        self.pcb.upgrade()
    }

    pub fn trap_ctx_ppn(&self) -> PhysPageNum {
        self.inner.exclusive_access().trap_ctx_ppn
    }
}

impl Drop for TCB {
    fn drop(&mut self) {
        debug!("tcb drop, tid: {}", self.inner.exclusive_access().tid());
    }
}

#[derive(Debug)]
pub struct TCBInner {
    resource: Option<TCBUserResource>, // 主线程不必分配资源, 从线程都要分配资源
    // 线程优先级, 1~5
    // 有些调度算法不会关注优先级, 例如 FIFO
    priority: u8,
    // 线程运行时间片
    count: usize,

    // 位于应用地址空间次高页的 TrapContext 的"物理"页号,
    // 目的是为了内核也能访问的这本来属于用户空间的内容,
    // 否则的话 TrapContext 位于用户空间内核怎么能访问到它呢
    trap_ctx_ppn: PhysPageNum,

    task_ctx: TaskContext,

    task_status: TaskStatus,

    exit_code: Option<i32>,
}

impl TCBInner {
    pub fn resource(&self) -> &TCBUserResource {
        self.resource.as_ref().unwrap()
    }
    pub fn tid(&self) -> usize {
        self.resource.as_ref().unwrap().tid
    }

    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn ustack_base(&self) -> usize {
        self.resource.as_ref().unwrap().ustack_base()
    }

    pub fn set_ustack_base(&mut self, ustack_base: usize) {
        self.resource.as_mut().unwrap().ustack_base = ustack_base
    }

    pub fn ustack_top(&self) -> usize {
        self.resource.as_ref().unwrap().ustack_top()
    }

    pub fn task_ctx(&mut self) -> &mut TaskContext {
        &mut self.task_ctx
    }

    pub fn trap_ctx(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }

    pub fn set_status(&mut self, task_status: TaskStatus) {
        self.task_status = task_status;
    }

    pub fn inc_count(&mut self) {
        // 60 是 1-5 的最小公倍数
        self.count += 60;
    }

    pub fn trap_ctx_ppn(&self) -> PhysPageNum {
        self.trap_ctx_ppn
    }

    pub fn set_trap_ctx_ppn(&mut self, trap_ctx_ppn: PhysPageNum) {
        self.trap_ctx_ppn = trap_ctx_ppn;
    }
}

// 公有接口
pub mod api {

    use super::*;

    pub fn init() {
        info!("adding 'init' tcb to shceduler");
        scheduler::add_ready(INITPROC.ex_inner().main_tcb());
    }
}
