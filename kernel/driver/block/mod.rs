mod virtio_blk;

use alloc::sync::Arc;
use component::fs::block_device::BlockDevice;

pub type BlockDeviceImpl = virtio_blk::VirtIOBlock;

