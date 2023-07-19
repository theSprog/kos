// use crate::sync::{Mutex, UPIntrFreeCell};
// use crate::task::{
//     block_current_and_run_next, block_current_task, current_task, wakeup_task, TaskContext,
//     TaskControlBlock,
// };
use crate::{
    process::processor::api::{block_current_tcb, current_tcb, wakeup_tcb},
    task::{context::TaskContext, TCB},
};
use alloc::{collections::VecDeque, sync::Arc};

use super::unicore::UPIntrFreeCell;

pub struct KCondvar {
    pub inner: UPIntrFreeCell<CondvarInner>,
}

pub struct CondvarInner {
    pub wait_queue: VecDeque<Arc<TCB>>,
}

impl KCondvar {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(CondvarInner {
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    // 唤醒
    pub fn signal(&self) {
        let mut inner = self.inner.exclusive_access();
        if let Some(task) = inner.wait_queue.pop_front() {
            wakeup_tcb(task);
        }
    }

    // 等待, 事件满足前不再调用
    pub fn wait_no_sched(&self) -> *mut TaskContext {
        self.inner.exclusive_session(|inner| {
            inner.wait_queue.push_back(current_tcb().unwrap());
        });
        block_current_tcb()
    }
    
    /*
    pub fn wait(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.wait_queue.push_back(current_task().unwrap());
        drop(inner);
        block_current_and_run_next();
    }
    */

    // pub fn wait_with_mutex(&self, mutex: Arc<dyn Mutex>) {
    //     mutex.unlock();
    //     self.inner.exclusive_session(|inner| {
    //         inner.wait_queue.push_back(current_task().unwrap());
    //     });
    //     block_current_and_run_next();
    //     mutex.lock();
    // }
}
