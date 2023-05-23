use alloc::{collections::VecDeque, sync::Arc};

use super::IScheduler;

pub struct FIFO<PCB> {
    // 就绪队列
    ready_queue: VecDeque<Arc<PCB>>,
}

/// A simple FIFO scheduler.
impl<PCB> FIFO<PCB> {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl<PCB> IScheduler<PCB> for FIFO<PCB> {
    fn add_ready(&mut self, task: Arc<PCB>) {
        self.ready_queue.push_back(task);
    }
    fn fetch(&mut self) -> Option<Arc<PCB>> {
        self.ready_queue.pop_front()
    }
}
