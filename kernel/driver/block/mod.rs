mod virtio_blk;

use alloc::sync::Arc;
use component::fs::block_device::BlockDevice;

type BlockDeviceImpl = virtio_blk::VirtIOBlock;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}
