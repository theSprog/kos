use alloc::{collections::VecDeque, sync::Arc};

use super::IPCBManager;

pub struct FIFOManager<PCB> {
    // 就绪队列
    ready_queue: VecDeque<Arc<PCB>>,
}

/// A simple FIFO scheduler.
impl<PCB> FIFOManager<PCB> {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl<PCB> IPCBManager<PCB> for FIFOManager<PCB> {
    fn add_ready(&mut self, task: Arc<PCB>) {
        self.ready_queue.push_back(task);
    }
    fn fetch(&mut self) -> Option<Arc<PCB>> {
        self.ready_queue.pop_front()
    }
}
