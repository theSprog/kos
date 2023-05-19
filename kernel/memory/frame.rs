use core::assert_eq;

use alloc::vec::Vec;
use logger::info;

use super::address::*;

use crate::lazy_static::lazy_static;
use crate::{
    memory::kernel_view::get_kernel_view, sync::unicore::UPSafeCell, MEMORY_END, PAGE_SIZE,
};
use component::util::*;
lazy_static! {
    pub(crate) static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = {
        info!("FRAME_ALLOCATOR Initializing...");
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) }
    };
}
type FrameAllocatorImpl = StackFrameAllocator;

pub fn init_frame_allocator() {
    let kernel_view = get_kernel_view();
    // free memory 区域
    let free_start = kernel_view.kernel_range().end;
    let free_end = MEMORY_END;
    assert_eq!(
        0,
        free_start % PAGE_SIZE,
        "unaligned free_start: {}",
        free_start
    );
    assert_eq!(0, free_end % PAGE_SIZE, "unaligned free_end: {}", free_end);

    info!(
        "Free memory range: [{:p}..{:p}), size: {}",
        free_start as *const u8,
        free_end as *const u8,
        human_size(free_end - free_start)
    );

    // init 参数是 PhysPageNum
    // 使用 into() 自动从物理地址中取出页号
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(free_start).into(),
        PhysAddr::from(free_end).into(),
    );
}

// 物理页分配器
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// 栈式空闲内存页管理器
pub struct StackFrameAllocator {
    current: usize,       //空闲内存的起始物理页号
    end: usize,           //空闲内存的结束物理页号
    recycled: Vec<usize>, // 释放的页可以被重复利用
}

impl StackFrameAllocator {
    /// 空闲内存起点终点(页号表示)
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        // 尝试从已经释放的页中分配内存
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current != self.end {
                self.current += 1;
                Some((self.current - 1).into())
            } else {
                // 实在没有内存页可分配了
                None
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // 有效性检查
        if ppn >= self.current || self.recycled.iter().find(|&v| *v == ppn).is_some() {
            // 未分配怎么可能被释放？
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // 加入 recycled 重复利用
        self.recycled.push(ppn);
    }
}

/// 物理页帧分配的接口是调用 frame_alloc 函数得到一个 PhysFrame （如果物理内存还有剩余），
/// 它就代表了一个物理页帧，当它的生命周期结束之后它所控制的物理页帧将被自动回收
pub struct PhysFrame {
    pub ppn: PhysPageNum,
}

impl PhysFrame {
    pub fn new(ppn: PhysPageNum) -> Self {
        // 清理页数据
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        api::frame_dealloc(self.ppn);
    }
}

pub mod api {
    use super::*;
    /// 公开给其他内核模块调用的分配/回收物理页帧的接口
    pub fn frame_alloc() -> Option<PhysFrame> {
        FRAME_ALLOCATOR
            .exclusive_access()
            .alloc()
            .map(|ppn| PhysFrame::new(ppn))
    }

    /// drop 隐式调用, 所以不公开
    pub(super) fn frame_dealloc(ppn: PhysPageNum) {
        FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
    }
}
