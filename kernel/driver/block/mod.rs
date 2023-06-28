mod virtio_blk;

use alloc::sync::Arc;
use component::fs::block_device::BlockDevice;
// use virtio_drivers::MmioTransport;

pub type BlockDeviceImpl = virtio_blk::VirtIOBlock;
