use component::process::IPCBManager;
use logger::info;

use crate::process::PCB;
use crate::{sync::unicore::UPSafeCell, PCBManager};
use alloc::sync::Arc;

lazy_static! {
    pub(crate) static ref PCB_MANAGER: UPSafeCell<PCBManager> = unsafe {
        info!("PCB_MANAGER initializing...");
        UPSafeCell::new(PCBManager::new())
    };
}

// scheduler 实际上是依赖外部实现
pub fn add_ready(task: Arc<PCB>) {
    PCB_MANAGER.exclusive_access().add_ready(task)
}

pub fn fetch() -> Option<Arc<PCB>> {
    PCB_MANAGER.exclusive_access().fetch()
}
