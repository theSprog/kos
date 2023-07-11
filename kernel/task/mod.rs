pub mod context;
pub mod switch;

use crate::loader::load_app;
use crate::memory::address_space;
use crate::process::{scheduler, PCB};

use crate::sync::unicore::UPSafeCell;
use crate::{
    memory::{address::*, address_space::KERNEL_SPACE, kernel_view::get_kernel_view},
    trap::{context::TrapContext, trap_handler},
    *,
};

use self::context::TaskContext;

use alloc::sync::{Arc, Weak};
use logger::info;
use process::stack::KernelStack;

// INIT 进程名称
pub const INIT: &str = "init";

lazy_static! {
    /// init 进程
    pub static ref INITPROC: Arc<PCB> =
    {
        info!("{INIT} process initializing...");
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
    Zombie,  // 僵尸

    Interruptable,   // 可中断睡眠
    Uninterruptable, // 不可中断睡眠
}

#[allow(clippy::upper_case_acronyms)]
pub struct TCB {
    pub pcb: Weak<PCB>,              // TCB 所属的进程
    pub kstack: KernelStack,         //任务（线程）的内核栈
    pub inner: UPSafeCell<TCBInner>, // 内部可变性
}

#[derive(Debug)]
pub struct TCBInner {
    // 线程优先级, 1~5
    // 有些调度算法不会关注优先级, 例如 FIFO
    pub priority: u8,

    // 线程运行时间片
    pub count: usize,

    // 位于应用地址空间次高页的 TrapContext 的"物理"页号,
    // 目的是为了内核也能访问的这本来属于用户空间的内容,
    // 否则的话 TrapContext 位于用户空间内核怎么能访问到它呢
    pub trap_ctx_ppn: PhysPageNum,

    pub task_ctx: TaskContext,

    pub task_status: TaskStatus,

    // base_size 统计了应用数据的大小，也就是在应用地址空间中从 0x0 开始到用户栈结束一共包含多少字节
    pub base_size: usize,
}

impl TCBInner {
    pub fn task_ctx(&mut self) -> &mut TaskContext {
        &mut self.task_ctx
    }

    pub fn trap_ctx(&mut self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }

    pub fn set_status(&mut self, task_status: TaskStatus) {
        self.task_status = task_status;
    }

    pub fn inc_count(&mut self) {
        // 60 是 1-5 的最小公倍数
        self.count += 60;
    }
}

impl TCB {
    pub fn new_once(pcb: &Arc<PCB>, user_sp: usize, entry_point: usize, priority: u8) -> TCB {
        assert!((1..=5).contains(&priority)); // 1-5 优先级
        let kernel_view = get_kernel_view();

        let pid = pcb.get_pid();
        // 查询 TrapContext 的物理页号
        let address_space = pcb.address_space();
        let trap_cx_ppn = address_space.trap_ctx_ppn();

        let task_status = TaskStatus::Ready;

        let kstack = KernelStack::alloc();
        let kstack_top = kstack.get_top();

        let tcb = Self {
            pcb: Arc::downgrade(pcb),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TCBInner {
                    priority,
                    task_status,
                    count: 0,
                    task_ctx: TaskContext::goto_trap_return(kstack_top),
                    trap_ctx_ppn: trap_cx_ppn,
                    base_size: user_sp,
                })
            },
        };

        // 为用户空间准备 TrapContext
        let trap_cx = tcb.trap_ctx_ppn();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        );

        tcb
    }

    pub fn ex_inner(&self) -> core::cell::RefMut<'_, TCBInner> {
        self.inner.exclusive_access()
    }

    pub fn pcb(&self) -> Option<Arc<PCB>> {
        self.pcb.upgrade()
    }

    // pub fn set_pcb(&mut self, pcb: &Arc<PCB>) {
    //     info!("{}", Arc::strong_count(pcb));
    //     self.mut_self().pcb = Arc::downgrade(pcb);
    //     info!("{}", self.pcb.strong_count());
    // }

    pub fn trap_ctx(&self) -> &'static mut TrapContext {
        self.inner.exclusive_access().trap_ctx_ppn.get_mut()
    }

    pub fn trap_ctx_ppn(&self) -> &'static mut TrapContext {
        self.inner.exclusive_access().trap_ctx_ppn.get_mut()
    }

    pub fn set_trap_ctx_ppn(&self, ppn: PhysPageNum) {
        self.inner.exclusive_access().trap_ctx_ppn = ppn;
    }

    pub fn set_base_size(&self, base_size: usize) {
        self.inner.exclusive_access().base_size = base_size;
    }

    pub fn priority(&self) -> u8 {
        self.inner.exclusive_access().priority
    }

    pub fn base_size(&self) -> usize {
        self.inner.exclusive_access().base_size
    }
}

// 公有接口
pub mod api {

    use super::*;

    pub fn init() {
        info!("adding init tcb to shceduler");
        let init_tcb = INITPROC.ex_inner().get_tcb(0);
        scheduler::add_ready(init_tcb);
    }
}
