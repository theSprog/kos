use core::assert_eq;

use super::address::*;
use alloc::vec::Vec;
use component::util::human_size::*;
use logger::info;

use crate::{
    memory::kernel_view::get_kernel_view, sync::unicore::UPSafeCell, MEMORY_END, PAGE_SIZE,
};

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
        debug_size(free_end - free_start)
    );

    // init 参数是 PhysPageNum
    // 使用 into() 自动从物理地址中取出页号
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(free_start).into(),
        PhysAddr::from(free_end).into(),
    );
}

// 物理页分配器
pub trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;

    // 一次性分配连续许多页
    fn alloc_n(&mut self, n: usize) -> Option<Vec<PhysPageNum>>;
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
        } else if self.current != self.end {
            self.current += 1;
            Some((self.current - 1).into())
        } else {
            // 实在没有内存页可分配了
            None
        }
    }

    fn alloc_n(&mut self, n: usize) -> Option<Vec<PhysPageNum>> {
        if self.current + n >= self.end {
            None
        } else {
            self.current += n;
            let arr: Vec<usize> = (1..n + 1).collect();
            // 以倒序的方式形成 vector, 例如 [3,2,1], last() 是 base 起始地址
            let v = arr.iter().map(|x| (self.current - x).into()).collect();
            Some(v)
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // 有效性检查
        if ppn >= self.current || self.recycled.iter().any(|v| *v == ppn) {
            // 未分配怎么可能被释放？出错
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
        let bytes_array = ppn.get_bytes_array();
        unsafe {
            // 清理页数据
            core::ptr::write_bytes(bytes_array.as_mut_ptr(), 0, bytes_array.len());
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
            .map(PhysFrame::new) // 把 PhysPageNum 转为 PhysFrame
    }

    pub fn frame_alloc_n(n: usize) -> Option<Vec<PhysFrame>> {
        FRAME_ALLOCATOR
            .exclusive_access()
            .alloc_n(n)
            .map(|x| x.iter().map(|&t| PhysFrame::new(t)).collect())
    }

    /// drop 隐式调用, 所以不公开
    pub fn frame_dealloc(ppn: PhysPageNum) {
        FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
    }
}
