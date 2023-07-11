use core::borrow::BorrowMut;

use component::process::IScheduler;
use logger::info;

use crate::process::PCB;
use crate::task::TCB;
use crate::{sync::unicore::UPSafeCell, KernelScheduler};
use alloc::sync::Arc;

use super::pid::PID_MAP;

lazy_static! {
    pub(crate) static ref SCHEDULER: UPSafeCell<KernelScheduler> = unsafe {
        info!("SCHEDULER initializing...");
        UPSafeCell::new(KernelScheduler::new())
    };
}

// scheduler 实际上是依赖外部实现
pub fn add_ready(tcb: Arc<TCB>) {
    let pcb = tcb.pcb().unwrap();
    let pid = pcb.get_pid();
    PID_MAP.exclusive_access().insert(pid, pcb.clone());
    SCHEDULER.exclusive_access().add_ready(tcb)
}

pub fn fetch() -> Option<Arc<TCB>> {
    SCHEDULER.exclusive_access().fetch()
}
