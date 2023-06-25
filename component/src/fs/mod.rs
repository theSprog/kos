pub mod block_device;

pub mod ext2;
pub mod fat32;
pub mod vfs;

pub mod block {
    pub const SIZE: usize = 4096;
    pub const LOG_SIZE: usize = 12;
    pub const BITS: usize = SIZE * 8;
    pub const MASK: usize = SIZE - 1;

    pub type DataBlock = [u8; SIZE];
    pub type BitmapBlock = [u64; SIZE / 64];
}

use alloc::vec::Vec;
use block_device::BlockCacheManager;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::default());
}

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
