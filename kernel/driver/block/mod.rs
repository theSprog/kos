mod virtio_blk;

use component::fs::block_device::BlockDevice;

pub type BlockDeviceImpl = virtio_blk::VirtIOBlock;
