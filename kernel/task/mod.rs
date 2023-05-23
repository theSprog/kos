pub mod context;
pub mod switch;

use crate::process::scheduler;

use crate::{
    loader::load_app,
    memory::{
        address::*,
        address_space::{AddressSpace, KERNEL_SPACE},
        kernel_view::get_kernel_view,
    },
    trap::{context::TrapContext, trap_handler},
    *,
};

use self::context::TaskContext;

use alloc::sync::Arc;
use logger::info;

// INIT 进程名称
pub const INIT: &str = "init";

lazy_static! {
    /// init 进程
    pub static ref INITPROC: Arc<PCB> =
    {
        info!("{INIT} proc initializing...");
        if let Some((init_data, init_path)) = load_app(INIT) {
            Arc::new(PCB::new(init_data, &init_path))
        }else {
            panic!("Failed to find '{INIT}' proc");
        }
    };

    pub static ref TCB_ONCE: &'static bool = &false;
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

// Task Control Block, 任务控制块
// 不是 thread control block
pub struct TCB {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,

    // 应用程序的地址空间
    pub address_space: AddressSpace,

    // 位于应用地址空间次高页的 TrapContext 的"物理"页号, 目的是为了内核也能访问的这本来属于用户空间的内容, 否则的话 TrapContext 位于用户空间内核怎么能访问到它呢
    pub trap_cx_ppn: PhysPageNum,

    // base_size 统计了应用数据的大小，也就是在应用地址空间中从 0x0 开始到用户栈结束一共包含多少字节
    pub base_size: usize,
}

impl TCB {
    /// pid 在该函数内惟一的作用就是决定内核栈的位置
    /// task_cx 需要用到该位置
    /// 注意该函数只应该调用一次, 剩下的全都是用 fork 创建出来
    pub fn new_once(elf_data: &[u8], pid: usize) -> TCB {
        let kernel_view = get_kernel_view();

        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(elf_data, pid);

        // 查询 TrapContext 的物理页号
        let trap_cx_ppn = address_space.trap_ppn();

        let task_status = TaskStatus::Ready;

        // 不需要在内核空间中申请内核栈, 外部的进程已经完成这件事了
        let (_, kernel_stack_top) = kernel_view.kernel_stack_range(pid);
        // KERNEL_SPACE.exclusive_access().insert_framed_segment(
        //     kernel_stack_bottom.into(),
        //     kernel_stack_top.into(),
        //     MapPermission::R | MapPermission::W,
        // );

        let tcb = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            address_space,
            trap_cx_ppn,
            base_size: user_sp,
        };

        // 为用户空间准备 TrapContext
        let trap_cx = tcb.trap_cx_ppn.get_mut();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        tcb
    }
}

// 公有接口
pub mod api {

    use super::*;

    pub fn init() {
        scheduler::add_ready(INITPROC.clone());
    }
}
