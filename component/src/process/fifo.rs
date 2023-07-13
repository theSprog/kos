use core::ops::{Deref, DerefMut};

use alloc::{collections::VecDeque, sync::Arc, vec::Vec};

use super::IScheduler;

pub struct FIFO<TCB> {
    // 就绪队列
    ready_queue: VecDeque<Arc<TCB>>,
}

/// A simple FIFO scheduler.
impl<TCB> FIFO<TCB> {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl<TCB> IScheduler<TCB> for FIFO<TCB> {
    fn add_ready(&mut self, task: Arc<TCB>) {
        self.ready_queue.push_back(task);
    }
    fn fetch(&mut self) -> Option<Arc<TCB>> {
        self.ready_queue.pop_front()
    }
    fn count(&self) -> usize {
        self.ready_queue.len()
    }

    fn filter<P: Fn(Arc<TCB>) -> bool>(&mut self, filter: P) -> Vec<Arc<TCB>> {
        let mut vec = alloc::vec![];
        for _ in 0..self.count() {
            let tcb = self.fetch().unwrap();
            if filter(tcb.clone()) {
                vec.push(tcb);
            } else {
                self.add_ready(tcb);
            }
        }
        vec
    }
}

impl<PCB> Deref for FIFO<PCB> {
    type Target = VecDeque<Arc<PCB>>;

    fn deref(&self) -> &Self::Target {
        &self.ready_queue
    }
}
