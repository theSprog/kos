use component::fs::block;
use component::fs::block_device::BlockDevice;
use component::HandleIRQ;

use core::ptr::NonNull;

use logger::*;
use virtio_drivers::device::blk::{VirtIOBlk, SECTOR_SIZE};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::driver::bus::VirtioHal;
use crate::sync::unicore::UPIntrFreeCell;
use virtio_drivers::transport::Transport;

pub type BlockDeviceImpl = VirtIOBlock;

const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<VirtioHal, MmioTransport>>,
    // condvars: BTreeMap<u16, Condvar>,
}

impl VirtIOBlock {
    pub fn new() -> Self {
        unsafe {
            let header = VIRTIO0 as *mut VirtIOHeader;
            let mut transport = MmioTransport::new(NonNull::new(header).unwrap()).unwrap();

            debug!("blk max send_queue size: {}", transport.max_queue_size(0));
            debug!("blk max recv_queue size: {}", transport.max_queue_size(1));

            let virtio_blk = UPIntrFreeCell::new(VirtIOBlk::new(transport).unwrap());

            Self { virtio_blk }
        }
    }
}

impl HandleIRQ for VirtIOBlock {
    fn handle_irq(&self) {
        todo!()
    }
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut blk = self.virtio_blk.exclusive_access();
        let lower_bid = block_id * block::SECTORS;

        // 底层是以 SECTOR_SIZE 为单位的
        for i in 0..block::SECTORS {
            blk.read_block(
                lower_bid + i,
                &mut buf[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE],
            )
            .expect("Error when reading VirtIOBlk");
        }
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut blk = self.virtio_blk.exclusive_access();
        let lower_bid = block_id * block::SECTORS;

        for i in 0..block::SECTORS {
            blk.write_block(lower_bid + i, &buf[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE])
                .expect("Error when writing VirtIOBlk");
        }
    }
}
