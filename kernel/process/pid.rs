use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use logger::info;

use crate::{process::PCB, sync::unicore::UPIntrFreeCell};

lazy_static! {
    static ref PID_ALLOCATOR: UPIntrFreeCell<RecycleAllocator> = unsafe {
        info!("PID_ALLOCATOR Initializing...");
        UPIntrFreeCell::new(RecycleAllocator::new())
    };

    // 保存 pid -> pcb 的映射关系
    pub static ref PID_MAP: UPIntrFreeCell<BTreeMap<usize, Arc<PCB>>> =
    unsafe {  info!("PID_MAP Initializing...");
    UPIntrFreeCell::new(BTreeMap::new()) };


}

pub struct Pid(pub usize);

impl Drop for Pid {
    // PidHandle 析构的时候, PID_ALLOCATOR 中也应该释放资源
    // 同时应该释放 pid -> pcb 的 map 映射
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

impl Pid {
    pub fn alloc() -> Pid {
        Pid(PID_ALLOCATOR.exclusive_access().alloc())
    }
}

impl From<usize> for Pid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Pid {
    pub fn unmap_pcb(&self) {
        match PID_MAP.exclusive_access().remove(&self.0) {
            Some(_pcb) => {
                // nothing to do so far
            }
            None => panic!("cannot unmap pid(={}) because there is no such key", self.0),
        }
    }
}

pub struct RecycleAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl RecycleAllocator {
    pub fn new() -> Self {
        RecycleAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.current);
        assert!(
            !self.recycled.iter().any(|i| *i == id),
            "id {} has been deallocated!",
            id
        );
        self.recycled.push(id);
    }
}
