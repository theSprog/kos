use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use super::vfs::error::{IOError, IOErrorKind, VfsResult};

use super::{blockgroup::Ext2BlockGroupDesc, layout::Ext2Layout, superblock::Superblock};

#[derive(Debug)]
pub struct Ext2Allocator {
    blocks_per_group: u32,
    inodes_per_group: u32,

    superblock: Arc<Mutex<Superblock>>,
    blockgroups: Arc<Vec<Mutex<Ext2BlockGroupDesc>>>,
}
impl Ext2Allocator {
    pub(crate) fn new(layout: Arc<Ext2Layout>) -> Ext2Allocator {
        Self {
            blocks_per_group: layout.blocks_per_group(),
            inodes_per_group: layout.inodes_per_group(),
            superblock: layout.superblock(),
            blockgroups: layout.blockgroups(),
        }
    }

    fn free_blocks(&self) -> u32 {
        let sb = self.superblock.lock();
        sb.free_blocks_count - sb.r_blocks_count
    }

    fn inc_free_blocks(&mut self, n: usize) {
        self.superblock.lock().free_blocks_count += n as u32;
    }

    fn dec_free_blocks(&mut self, n: usize) {
        self.superblock.lock().free_blocks_count -= n as u32;
    }

    fn inc_free_inode(&mut self) {
        self.superblock.lock().free_inodes_count += 1;
    }

    fn dec_free_inode(&mut self) {
        self.superblock.lock().free_inodes_count -= 1;
    }

    fn free_inodes(&self) -> u32 {
        self.superblock.lock().free_inodes_count
    }

    // 将 block_id 分解成 bg 索引和 bg 内偏移
    fn decomposition_block_id(&self, block_id: u32) -> (usize, usize) {
        (
            (block_id / self.blocks_per_group) as usize,
            (block_id % self.blocks_per_group) as usize,
        )
    }

    fn decomposition_inode_id(&self, inode_id: u32) -> (usize, usize) {
        // 特别注意 inode_id 是从 1 开始的, 转为索引要减一
        let inode_idx = inode_id - 1;
        (
            (inode_idx / self.inodes_per_group) as usize,
            (inode_idx % self.inodes_per_group) as usize,
        )
    }

    pub(crate) fn alloc_inode(&mut self, is_dir: bool) -> VfsResult<u32> {
        if self.free_inodes() == 0 {
            return Err(IOError::new(IOErrorKind::NoFreeInodes).into());
        }

        // 到此则有可用 inode
        self.dec_free_inode();
        for bg in self.blockgroups.iter() {
            let mut bg = bg.lock();
            if bg.free_blocks_count == 0 {
                continue;
            }
            return Ok(bg.alloc_inode(is_dir));
        }

        unreachable!()
    }

    pub(crate) fn dealloc_inode(&mut self, inode_id: u32, is_dir: bool) -> VfsResult<()> {
        // 找出属于哪个块组, 块组内偏移多少
        let (bg_idx, inner_idx) = self.decomposition_inode_id(inode_id);

        let bg = self.blockgroups.get(bg_idx).unwrap();
        bg.lock().dealloc_inode(inner_idx as u32, is_dir);
        self.inc_free_inode();

        Ok(())
    }

    pub(crate) fn alloc_data(&mut self, needed: usize) -> VfsResult<Vec<u32>> {
        if needed > self.free_blocks() as usize {
            return Err(IOError::new(IOErrorKind::NoFreeBlocks).into());
        }
        let mut ret = Vec::new();
        if needed == 0 {
            return Ok(ret);
        }

        let mut unmet = needed;
        // 需要分别更新 superblock 的 free_blocks 和 blockgroups 的 free_blocks_count
        for bg in self.blockgroups.iter() {
            let mut bg = bg.lock();
            // 每一个 bg 都尽力分配 unmet 个块, 返回分配的块数
            let allocated = bg.alloc_blocks(unmet);
            unmet -= allocated.len();
            ret.extend(allocated);
            if unmet == 0 {
                break;
            }
        }

        // 扣除 free_blocks
        self.dec_free_blocks(needed);
        // 前面判断有空间, 因此跳出循环时必然 unmet == 0
        assert_eq!(unmet, 0);
        Ok(ret)
    }

    pub(crate) fn dealloc_data(&mut self, mut freed: Vec<u32>) -> VfsResult<()> {
        let mut slots = alloc::vec![0; self.blockgroups.len()];

        // 让所有同一 blockgroup 的聚集在连续一块
        freed.sort();

        // 标出分别属于哪一个 blockgroup
        for bid in &freed {
            let bg_idx = (*bid / self.blocks_per_group) as usize;
            slots[bg_idx] += 1;
        }

        let mut offset = 0;
        for (idx, bg) in self.blockgroups.iter().enumerate() {
            let mut bg = bg.lock();
            let bg_blocks = &freed[offset..offset + slots[idx]]
                .iter()
                .map(|&block_id| (block_id % self.blocks_per_group) as u32)
                .collect::<Vec<_>>();

            bg.dealloc_blocks(bg_blocks);
            offset += slots[idx];
        }

        self.inc_free_blocks(freed.len());
        assert_eq!(offset, freed.len());

        Ok(())
    }
}
