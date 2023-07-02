use crate::memory::address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr};
use crate::memory::address_space::kernel_token;
use crate::memory::frame::api::{frame_alloc_n, frame_dealloc};
use crate::memory::frame::PhysFrame;

use crate::memory::page_table::PageTable;
use crate::sync::up::UPIntrFreeCell;

use alloc::vec::Vec;
use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal};

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<PhysFrame>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub struct VirtioHal;

#[allow(unused_variables)]
unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, direction: BufferDirection) -> (usize, NonNull<u8>) {
        let trakcers = frame_alloc_n(pages);
        // last 即是分配的 dma 连续内存块的起始页号
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access()
            .append(&mut trakcers.unwrap());
        let paddr: PhysAddr = ppn_base.into();
        let vaddr = NonNull::new(paddr.0 as _).unwrap();
        (paddr.0, vaddr)
    }

    unsafe fn dma_dealloc(paddr: usize, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let pa = PhysAddr::from(paddr);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: usize, size: usize) -> NonNull<u8> {
        NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> usize {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        let paddr = PageTable::from_token(kernel_token())
            .translate_vaddr(VirtAddr::from(vaddr))
            .unwrap()
            .0;
        paddr
    }

    unsafe fn unshare(paddr: usize, buffer: NonNull<[u8]>, direction: BufferDirection) {
        // todo!()
    }

    // fn virt_to_phys(vaddr: usize) -> usize {
    //     PageTable::from_token(kernel_token())
    //         .translate_vaddr(VirtAddr::from(vaddr))
    //         .unwrap()
    //         .0
    // }
}
