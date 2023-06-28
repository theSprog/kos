use core::ptr::NonNull;

use logger::info;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use super::BlockDevice;
use crate::driver::bus::virtio::VirtioHal;
use crate::sync::up::UPIntrFreeCell;

const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<VirtioHal, MmioTransport>>,
    // condvars: BTreeMap<u16, Condvar>,
}

impl VirtIOBlock {
    pub fn new() -> Self {
        unsafe {
            let header = VIRTIO0 as *mut VirtIOHeader;
            let transport = MmioTransport::new(NonNull::new(header).unwrap()).unwrap();
            let virtio_blk = UPIntrFreeCell::new(VirtIOBlk::new(transport).unwrap());

            Self { virtio_blk }
        }
    }
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.virtio_blk
            .exclusive_access()
            .read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.virtio_blk
            .exclusive_access()
            .write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}
