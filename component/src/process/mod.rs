pub mod fifo;

use alloc::{sync::Arc, vec::Vec};
pub use fifo::FIFO;

// 进程调度器通用接口
pub trait IScheduler<TCB> {
    fn add_ready(&mut self, task: Arc<TCB>);
    fn fetch(&mut self) -> Option<Arc<TCB>>;
    fn count(&self) -> usize;
    fn filter<P: Fn(Arc<TCB>) -> bool>(&mut self, filter: P) -> Vec<Arc<TCB>>;
}
