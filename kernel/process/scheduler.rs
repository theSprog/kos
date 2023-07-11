use component::process::IScheduler;
use logger::info;

use crate::process::PCB;
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
pub fn add_ready(pcb: Arc<PCB>) {
    let pid = pcb.getpid();
    PID_MAP.exclusive_access().insert(pid, pcb.clone());
    SCHEDULER.exclusive_access().add_ready(pcb)
}

pub fn fetch() -> Option<Arc<PCB>> {
    SCHEDULER.exclusive_access().fetch()
}
