use alloc::sync::Arc;
use logger::info;

use crate::{
    memory::address_space::{kernel_token, KERNEL_SPACE},
    process::{processor, scheduler},
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
        KERNEL_SPACE.exclusive_access().token(),
        new_tcb.kstack.get_top(),
        trap_handler as usize,
    );
    new_tcb_trap_ctx.x[10] = arg;
    drop(new_tcb_inner);

    // add new tcb to scheduler
    scheduler::add_ready(new_tcb);

    info!("entry: {:?}, arg: {:?}, tid: {}", entry, arg, tid);

    tid as isize
}
