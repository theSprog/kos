use core::ptr::NonNull;

use crate::driver::bus::virtio::VirtioHal;
use crate::sync::up::UPIntrFreeCell;

use super::BlockDevice;
use alloc::collections::BTreeMap;
use spin::Mutex;
use virtio_drivers::device::blk::{BlkResp, RespStatus, VirtIOBlk};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::Transport;

const VIRTIO0: usize = 0x10001000;
pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<VirtioHal, MmioTransport>>,
    // condvars: BTreeMap<u16, Condvar>,
}

impl VirtIOBlock {
    pub fn new() -> VirtIOBlock {
        let header = NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();

        unsafe {
            Self {
                virtio_blk: UPIntrFreeCell::new(VirtIOBlk::new(transport).unwrap()),
            }
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

// pub struct VirtIOBlock {
//     // virtio_blk: UPIntrFreeCell<VirtIOBlk<'static, VirtioHal>>,
//     // condvars: BTreeMap<u16, Condvar>,
// }

// impl BlockDevice for VirtIOBlock {
//     // fn read_block(&self, block_id: usize, buf: &mut [u8]) {
//     //     todo!()
//     // }

//     // fn write_block(&self, block_id: usize, buf: &[u8]) {
//     //     todo!()
//     // }

//     // fn read_block(&self, block_id: usize, buf: &mut [u8]) {
//     //     let nb = *DEV_NON_BLOCKING_ACCESS.exclusive_access();
//     //     if nb {
//     //         let mut resp = BlkResp::default();
//     //         let task_cx_ptr = self.virtio_blk.exclusive_session(|blk| {
//     //             let token = unsafe { blk.read_block_nb(block_id, buf, &mut resp).unwrap() };
//     //             self.condvars.get(&token).unwrap().wait_no_sched()
//     //         });
//     //         schedule(task_cx_ptr);
//     //         assert_eq!(
//     //             resp.status(),
//     //             RespStatus::Ok,
//     //             "Error when reading VirtIOBlk"
//     //         );
//     //     } else {
//     //         self.virtio_blk
//     //             .exclusive_access()
//     //             .read_block(block_id, buf)
//     //             .expect("Error when reading VirtIOBlk");
//     //     }
//     // }
//     // fn write_block(&self, block_id: usize, buf: &[u8]) {
//     //     let nb = *DEV_NON_BLOCKING_ACCESS.exclusive_access();
//     //     if nb {
//     //         let mut resp = BlkResp::default();
//     //         let task_cx_ptr = self.virtio_blk.exclusive_session(|blk| {
//     //             let token = unsafe { blk.write_block_nb(block_id, buf, &mut resp).unwrap() };
//     //             self.condvars.get(&token).unwrap().wait_no_sched()
//     //         });
//     //         schedule(task_cx_ptr);
//     //         assert_eq!(
//     //             resp.status(),
//     //             RespStatus::Ok,
//     //             "Error when writing VirtIOBlk"
//     //         );
//     //     } else {
//     //         self.virtio_blk
//     //             .exclusive_access()
//     //             .write_block(block_id, buf)
//     //             .expect("Error when writing VirtIOBlk");
//     //     }
//     // }
//     // fn handle_irq(&self) {
//     //     self.virtio_blk.exclusive_session(|blk| {
//     //         while let Ok(token) = blk.pop_used() {
//     //             self.condvars.get(&token).unwrap().signal();
//     //         }
//     //     });
//     // }
// }

// impl VirtIOBlock {
//     pub fn new() -> Self {
//         todo!()
//         // let virtio_blk = unsafe {
//         //     UPIntrFreeCell::new(
//         //         VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
//         //     )
//         // };
//         // let mut condvars = BTreeMap::new();
//         // let channels = virtio_blk.exclusive_access().virt_queue_size();
//         // for i in 0..channels {
//         //     let condvar = Condvar::new();
//         //     condvars.insert(i, condvar);
//         // }
//         // Self {
//         //     virtio_blk,
//         //     condvars,
//         // }
//     }
// }
