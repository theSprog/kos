pub mod fifo;

use alloc::sync::Arc;
pub use fifo::FIFOManager;

// 进程管理器通用接口
pub trait IPCBManager<PCB> {
    fn add_ready(&mut self, task: Arc<PCB>);
    fn fetch(&mut self) -> Option<Arc<PCB>>;
}
