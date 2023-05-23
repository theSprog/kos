use component::process::IScheduler;
use logger::info;

use crate::process::PCB;
use crate::{sync::unicore::UPSafeCell, Scheduler};
use alloc::sync::Arc;

lazy_static! {
    pub(crate) static ref SCHEDULER: UPSafeCell<Scheduler> = unsafe {
        info!("SCHEDULER initializing...");
        UPSafeCell::new(Scheduler::new())
    };
}

// scheduler 实际上是依赖外部实现
pub fn add_ready(task: Arc<PCB>) {
    SCHEDULER.exclusive_access().add_ready(task)
}

pub fn fetch() -> Option<Arc<PCB>> {
    SCHEDULER.exclusive_access().fetch()
}
