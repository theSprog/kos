use crate::driver::bus::virtio::VirtioHal;
use crate::sync::up::UPIntrFreeCell;
use super::BlockDevice;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<'static, VirtioHal>>,
    // condvars: BTreeMap<u16, Condvar>,
}

impl VirtIOBlock {
    pub fn new() -> VirtIOBlock {
        unsafe {
            let virtio_blk = UPIntrFreeCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            );

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
