use crate::memory::address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr};
use crate::memory::address_space::kernel_token;
use crate::memory::frame::api::{frame_alloc, frame_alloc_n, frame_dealloc};
use crate::memory::frame::PhysFrame;

use crate::memory::page_table::PageTable;
use crate::sync::up::UPIntrFreeCell;

use alloc::vec::Vec;
use core::ptr::NonNull;
use virtio_drivers::Hal;

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<PhysFrame>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let trakcers = frame_alloc_n(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access()
            .append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        pa.0
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PhysAddr::from(pa);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        PageTable::from_token(kernel_token())
            .translate_vaddr(VirtAddr::from(vaddr))
            .unwrap()
            .0
    }
}
