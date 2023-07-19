use alloc::sync::Arc;
use logger::*;
use sys_interface::syserr;

use crate::{
    memory::address_space::kernel_token,
    process::{
        processor::{self},
        scheduler,
    },
    task::TCB,
    trap::{context::TrapContext, trap_handler},
};

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let tcb = processor::api::current_tcb().unwrap();
    let pcb = tcb.pcb().unwrap();
    let ustack_base = tcb.ustack_base();
    // create a new thread
    let new_tcb = Arc::new(TCB::new(&pcb, ustack_base, true));
    pcb.ex_add_tcb(new_tcb.clone());

    let new_tcb_inner = new_tcb.ex_inner();
    let tid = new_tcb_inner.tid();

    let new_tcb_trap_ctx = new_tcb_inner.trap_ctx();
    *new_tcb_trap_ctx = TrapContext::app_init_context(
        entry,
        new_tcb_inner.ustack_top(),
        kernel_token(),
        new_tcb.kstack.get_top(),
        trap_handler as usize,
    );
    new_tcb_trap_ctx.x[10] = arg;
    drop(new_tcb_inner);

    // add new tcb to scheduler
    scheduler::add_ready(new_tcb);

    tid as isize
}

pub fn sys_waittid(tid: usize) -> isize {
    let tcb = processor::api::current_tcb().unwrap();
    let pcb = tcb.pcb().unwrap();
    let tcb_inner = tcb.ex_inner();
    let mut pcb_inner = pcb.ex_inner();

    // 不能等待自身, 也不能等待不存在的线程
    if tcb_inner.tid() == tid
        || tid >= pcb_inner.tcb_slots_len()
        || pcb_inner.get_tcb(tid).is_none()
    {
        return syserr::EINVAL;
    }
    drop(tcb_inner);

    let waited_tcb = pcb_inner.get_tcb(tid).unwrap();
    let exit_code = waited_tcb.ex_inner().exit_code();
    // 查看退出码
    if let Some(exit_code) = exit_code {
        drop(waited_tcb);
        // 线程已退出, take tcb
        let died_tcb = pcb_inner.take_tcb_nth(tid).unwrap();
        // tcb 析构要用到 pcb, 因此先 drop 析构 pcb_inner, 防止多次 borrow
        drop(pcb_inner);
        assert_eq!(Arc::strong_count(&died_tcb), 1);
        exit_code as isize
    } else {
        // 线程还未退出
        syserr::EAGAIN
    }
}

pub fn sys_exit(exit_code: i32) -> ! {
    processor::api::exit_and_run_next(exit_code)
}

pub fn sys_sched_yield() -> isize {
    // 处理方式就是挂起当前，并且运行下一个
    processor::api::suspend_and_run_next();
    0
}
