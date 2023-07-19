use alloc::vec::Vec;
use component::process::IScheduler;
use logger::info;

use crate::task::TCB;
use crate::{sync::unicore::UPIntrFreeCell, KernelScheduler};
use alloc::sync::Arc;

use super::pid::PID_MAP;

lazy_static! {
    pub(crate) static ref SCHEDULER: UPIntrFreeCell<KernelScheduler> = unsafe {
        info!("SCHEDULER initializing...");
        UPIntrFreeCell::new(KernelScheduler::new())
    };
}

// scheduler 实际上是依赖外部实现
pub fn add_ready(tcb: Arc<TCB>) {
    let pcb = tcb.pcb().unwrap();
    let pid = pcb.pid();
    PID_MAP.exclusive_access().entry(pid).or_insert(pcb);
    SCHEDULER.exclusive_access().add_ready(tcb)
}

pub fn fetch() -> Option<Arc<TCB>> {
    SCHEDULER.exclusive_access().fetch()
}

pub fn count() -> usize {
    SCHEDULER.exclusive_access().count()
}

pub fn filter<P: Fn(Arc<TCB>) -> bool>(pred: P) -> Vec<Arc<TCB>> {
    SCHEDULER.exclusive_access().filter(pred)
}
