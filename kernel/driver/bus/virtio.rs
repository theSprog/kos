use crate::memory::address::PhysPageNum;
use crate::memory::address::StepByOne;
use crate::memory::address::VirtAddr;
use crate::{
    memory::frame::{
        api::{frame_alloc, frame_alloc_n, frame_dealloc},
        PhysFrame,
    },
    sync::up::UPIntrFreeCell,
};
use alloc::vec::Vec;
use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<PhysFrame>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let trakcers = frame_alloc_n(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access()
            .append(&mut trakcers.unwrap());

        todo!();
        (ppn_base.into(), NonNull::new(0 as *mut u8).unwrap())
    }

    // paddr 是起始地址
    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let mut ppn_base: PhysPageNum = paddr.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            // 前进一步
            ppn_base.step();
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        // NonNull::new(VirtAddr(paddr).into())
        todo!()
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        todo!()
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, direction: BufferDirection) {
        todo!()
    }
}
