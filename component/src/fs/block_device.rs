use core::any::Any;

use alloc::{collections::BTreeMap, sync::Arc};
use logger::info;
use spin::Mutex;

use super::{block, SECTOR_SIZE};
use crate::{cast, cast_mut};

pub trait BlockDevice: Send + Sync + Any {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
}

pub struct BlockCache {
    cache: [u8; block::SIZE],
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
    modified: bool,
}

impl BlockCache {
    /// Load a new BlockCache from disk.
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; block::SIZE];
        let lower_bid = block_id * block::SECTORS_PER_BLOCK;

        // 底层是以 SECTOR_SIZE 为单位的
        for i in 0..block::SECTORS_PER_BLOCK {
            block_device.read_block(
                lower_bid + i,
                &mut cache[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE],
            );
        }

        Self {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }

    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    /// # Safety
    pub unsafe fn get_ref<T>(&self, offset: usize) -> &T {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= block::SIZE);
        let addr = self.addr_of_offset(offset);
        cast!(addr, T)
    }

    /// # Safety
    pub unsafe fn get_mut<T>(&mut self, offset: usize) -> &mut T {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= block::SIZE);
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        cast_mut!(addr, T)
    }

    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;

            let lower_bid = self.block_id * block::SECTORS_PER_BLOCK;
            // 底层是以 SECTOR_SIZE 为单位的
            for i in 0..block::SECTORS_PER_BLOCK {
                self.block_device.write_block(
                    lower_bid + i,
                    &self.cache[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE],
                );
            }
        }
    }

    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(unsafe { self.get_ref(offset) })
    }

    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(unsafe { self.get_mut(offset) })
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}

const BLOCK_CACHE_SIZE: usize = 32;

#[derive(Default)]
pub struct BlockCacheManager {
    map: BTreeMap<usize, Arc<Mutex<BlockCache>>>,
    block_device: Option<Arc<dyn BlockDevice>>,
}

impl BlockCacheManager {
    pub fn get_block_cache(&mut self, block_id: usize) -> Arc<Mutex<BlockCache>> {
        // 如果已经在缓存中
        if let Some(block_cache) = self.map.get(&block_id) {
            block_cache.clone()
        } else {
            // 保留还有引用的
            if self.map.len() == BLOCK_CACHE_SIZE {
                if let Some((&key, _)) = self
                    .map
                    .iter()
                    .find(|(_, cache)| Arc::strong_count(cache) == 1)
                {
                    self.map.remove(&key);
                }
            }

            let block_cache = Arc::new(Mutex::new(BlockCache::new(
                block_id,
                Arc::clone(
                    self.block_device
                        .as_ref()
                        .expect("block_device haven't been registered yet"),
                ),
            )));
            self.map.insert(block_id, block_cache.clone());
            block_cache
        }
    }

    pub fn flush(&mut self) {
        for (_, block_cache) in self.map.iter_mut() {
            block_cache.lock().sync();
        }
    }
}

pub fn register_block_device(block_device: impl BlockDevice) {
    let old: Option<Arc<dyn BlockDevice>> = super::BLOCK_CACHE_MANAGER
        .lock()
        .block_device
        .replace(Arc::new(block_device));
    assert!(old.is_none(), "block device double register");
}

fn block_nth(block_id: usize) -> Arc<Mutex<BlockCache>> {
    super::BLOCK_CACHE_MANAGER.lock().get_block_cache(block_id)
}

pub fn read<T, V>(block_id: usize, offset: usize, operation: impl FnOnce(&T) -> V) -> V {
    block_nth(block_id).lock().read(offset, operation)
}

pub fn modify<T, V>(block_id: usize, offset: usize, operation: impl FnOnce(&mut T) -> V) -> V {
    block_nth(block_id).lock().modify(offset, operation)
}

pub fn sync(block_id: usize) {
    block_nth(block_id).lock().sync()
}

pub fn flush() {
    super::BLOCK_CACHE_MANAGER.lock().flush()
}
