pub mod block_device;

pub mod ext2;
pub mod fat32;
pub mod vfs;

pub const SECTOR_SIZE: usize = 512;

pub mod block {
    pub use super::SECTOR_SIZE;

    pub const SIZE: usize = 4096;
    pub const LOG_SIZE: usize = 12;
    pub const BITS: usize = SIZE * 8;
    pub const MASK: usize = SIZE - 1;
    pub const SECTORS: usize = 8;

    pub type DataBlock = [u8; SIZE];
}

use alloc::vec::Vec;
use block_device::BlockCacheManager;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::default());
}
