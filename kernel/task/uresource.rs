use alloc::sync::{Arc, Weak};
use logger::*;
use sys_interface::config::{MAX_THREADS, PAGE_SIZE, USER_STACK_SIZE};

use crate::{
    memory::{
        address::{PhysPageNum, VirtAddr},
        segment::MapPermission,
    },
    process::PCB,
    TRAP_CONTEXT,
};

#[derive(Debug)]
pub struct TCBUserResource {
    pub tid: usize,
    pub ustack_base: usize,
    pub pcb: Weak<PCB>,
}

impl Drop for TCBUserResource {
    fn drop(&mut self) {
        self.dealloc_tid();
        self.dealloc_uresource();
    }
}

impl TCBUserResource {
    // 主线程不用 alloc_uresource, 其他派生出的线程需要
    pub fn new(pcb: Arc<PCB>, ustack_base: usize, alloc_uresource: bool) -> Self {
        let tid = pcb.ex_inner().alloc_tid();
        let tcb_uresource = Self {
            tid,
            ustack_base,
            pcb: Arc::downgrade(&pcb),
        };
        if alloc_uresource {
            // main 线程不应该申请 user resource, 该方法只能给从线程调用
            assert_ne!(tid, 0);
            tcb_uresource.alloc_uresource();
        } else {
            // 不分配资源的必然是 main 线程
            assert_eq!(tid, 0);
        }
        tcb_uresource
    }

    pub fn ustack_base(&self) -> usize {
        self.ustack_base
    }

    pub fn ustack_top(&self) -> usize {
        ustack_bottom(self.ustack_base, self.tid) + USER_STACK_SIZE
    }

    /// 在进程地址空间中实际映射线程的用户栈和 Trap 上下文。
    pub fn alloc_uresource(&self) {
        let pcb = self.pcb.upgrade().unwrap();
        let mut pcb_inner = pcb.ex_inner();
        // 分配用户栈空间
        let ustack_bottom = ustack_bottom(self.ustack_base, self.tid);
        let ustack_top = ustack_bottom + USER_STACK_SIZE;

        pcb_inner.address_space().insert_framed_segment_lazy(
            ustack_bottom.into(),
            ustack_top.into(),
            MapPermission::R | MapPermission::W | MapPermission::U,
        );

        // 分配 trap context 空间
        let trap_cx_bottom = trap_ctx_bottom(self.tid);
        let trap_cx_top = trap_cx_bottom + PAGE_SIZE;
        pcb_inner.address_space().insert_framed_segment(
            trap_cx_bottom.into(),
            trap_cx_top.into(),
            MapPermission::R | MapPermission::W,
        );
    }

    pub fn dealloc_tid(&self) {
        let pcb = self.pcb.upgrade().unwrap();
        let mut pcb_inner = pcb.ex_inner();
        pcb_inner.dealloc_tid(self.tid);
    }

    fn dealloc_uresource(&self) {
        // dealloc tid
        let pcb = self.pcb.upgrade().unwrap();
        let mut pcb_inner = pcb.ex_inner();

        // 释放用户栈
        let ustack_bottom_vaddr: VirtAddr = ustack_bottom(self.ustack_base, self.tid).into();
        pcb_inner
            .address_space()
            .free_user_segment(ustack_bottom_vaddr.into());

        // 释放 trap_ctx
        let trap_ctx_bottom_vaddr: VirtAddr = trap_ctx_bottom(self.tid).into();
        pcb_inner
            .address_space()
            .free_user_segment(trap_ctx_bottom_vaddr.into());
    }

    // 找到该线程所属的 trap_ctx_ppn
    // 该方法会用到独占地址空间(ex_address_space), 因此有 ex 标记
    pub fn trap_ctx_ppn_ex(&self) -> PhysPageNum {
        let pcb = self.pcb.upgrade().unwrap();
        let trap_ctx_bottom_vaddr: VirtAddr = trap_ctx_bottom(self.tid).into();
        pcb.ex_address_space()
            .translate_vpn(trap_ctx_bottom_vaddr.into())
            .unwrap()
            .ppn()
    }
}

// 线程所属的 trap ctx 底部
fn trap_ctx_bottom(tid: usize) -> usize {
    assert!(tid < MAX_THREADS);
    TRAP_CONTEXT - tid * PAGE_SIZE
}

// 线程所属的栈底部
fn ustack_bottom(ustack_base: usize, tid: usize) -> usize {
    assert!(tid < MAX_THREADS);
    ustack_base + tid * (PAGE_SIZE + USER_STACK_SIZE)
}
