use alloc::vec::Vec;
use logger::info;

use crate::sync::unicore::UPSafeCell;

lazy_static! {
    static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> = unsafe {
        info!("PID_ALLOCATOR Initializing...");
        UPSafeCell::new(PidAllocator::new())
    };
}

pub struct Pid(pub usize);
impl From<usize> for Pid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Drop for Pid {
    // PidHandle 析构的时候, PID_ALLOCATOR 中也应该释放资源
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}
struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    fn new() -> Self {
        PidAllocator {
            current: 1,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Pid {
        if let Some(pid) = self.recycled.pop() {
            Pid(pid)
        } else {
            self.current += 1;
            Pid(self.current - 1)
        }
    }
    fn dealloc(&mut self, pid: usize) {
        // current 是目前尚未分配的 pid 的下界
        assert!(pid < self.current);
        // 不可能在可重复利用的集合中
        assert!(
            self.recycled.iter().all(|&ppid| ppid != pid),
            "pid {} has been deallocated!",
            pid
        );
        self.recycled.push(pid);
    }
}

pub mod api {
    use super::*;
    pub fn pid_alloc() -> Pid {
        PID_ALLOCATOR.exclusive_access().alloc()
    }
}
