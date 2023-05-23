pub mod fifo;

use alloc::sync::Arc;
pub use fifo::FIFO;

// 进程调度器通用接口
pub trait IScheduler<PCB> {
    fn add_ready(&mut self, task: Arc<PCB>);
    fn fetch(&mut self) -> Option<Arc<PCB>>;
}
