use crate::cast;

use alloc::{sync::Arc, vec::Vec};
use core::fmt::{self, Debug};
use spin::Mutex;

use crate::fs::block_device;

use super::{
    block::{self, DataBlock},
    vfs::meta::VfsFileType,
};

use super::{
    address::Address, allocator::Ext2Allocator, disk_inode::Ext2Inode, inode::Inode,
    layout::Ext2Layout,
};

#[repr(C)]
#[derive(Clone)]
pub struct Ext2BlockGroupDesc {
    /// Block address of block usage bitmap
    pub block_bitmap_addr: u32,
    /// Block address of inode usage bitmap
    pub inode_bitmap_addr: u32,
    /// Starting block address of inode table
    pub inode_table_block: u32,
    /// Number of unallocated blocks in group
    pub free_blocks_count: u16,
    /// Number of unallocated inodes in group
    pub free_inodes_count: u16,
    /// Number of directories in group
    pub dirs_count: u16,
    #[doc(hidden)]
    _reserved: [u8; 14],
}

const UNIT_WIDTH: usize = 64;
type BitmapBlock = [u64; block::SIZE / UNIT_WIDTH];

impl Ext2BlockGroupDesc {
    pub(crate) fn find(count: u32) -> Vec<Self> {
        block_device::read(1, 0, |data: &DataBlock| {
            let mut vec: Vec<Ext2BlockGroupDesc> = Vec::new();
            let mut offset = 0;
            for _ in 0..count {
                let current = &data[offset..];
                let desc = cast!(current.as_ptr(), Ext2BlockGroupDesc);
                vec.push(desc.clone());
                offset += core::mem::size_of::<Ext2BlockGroupDesc>();
            }
            vec
        })
    }

    fn block_bitmap_bid(&self) -> usize {
        self.block_bitmap_addr as usize
    }

    fn inode_bitmap_bid(&self) -> usize {
        self.inode_bitmap_addr as usize
    }

    fn inode_table_bid(&self) -> usize {
        self.inode_table_block as usize
    }

    /// inode_inner_idx 指的是 inode 在 block group 中的内部偏移
    pub fn get_inode(
        &self,
        inode_id: usize,
        inode_inner_idx: usize,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        let address = Address::new(
            self.inode_table_bid(),
            (inode_inner_idx * core::mem::size_of::<Ext2Inode>()) as isize,
        );
        Inode::read(inode_id, address, layout, allocator)
    }

    pub fn new_inode(
        &self,
        inode_id: usize,
        inode_inner_idx: usize,
        filetype: VfsFileType,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        let address = Address::new(
            self.inode_table_bid(),
            (inode_inner_idx * core::mem::size_of::<Ext2Inode>()) as isize,
        );
        Inode::new(inode_id, address, filetype, layout, allocator)
    }

    #[inline]
    fn decomposition(&self, bit_idx: u32) -> (usize, usize) {
        (bit_idx as usize / UNIT_WIDTH, bit_idx as usize % UNIT_WIDTH)
    }

    // 调用该函数必然成功, 所有的检查应该在外部完成
    pub fn alloc_inode(&mut self, is_dir: bool) -> u32 {
        assert_ne!(self.free_inodes_count, 0);
        // 不要忘记更新 free_inodes_count
        self.free_inodes_count -= 1;

        block_device::modify(self.inode_bitmap_bid(), 0, |bitmap: &mut BitmapBlock| {
            use core::ops::Not;
            for (pos, bits) in bitmap.iter_mut().enumerate() {
                let neg_bits = bits.not();
                while neg_bits != 0 {
                    let inner_pos = neg_bits.trailing_zeros() as usize;
                    *bits |= 1 << inner_pos;

                    if is_dir {
                        self.dirs_count += 1;
                    }

                    // 特别注意 inode 从 1 开始计数
                    return (pos * UNIT_WIDTH + inner_pos + 1) as u32;
                }
            }

            unreachable!()
        })
    }

    pub fn dealloc_inode(&mut self, bit_idx: u32, is_dir: bool) {
        self.free_inodes_count += 1;

        block_device::modify(self.inode_bitmap_bid(), 0, |bitmap: &mut BitmapBlock| {
            let (pos, inner_pos) = self.decomposition(bit_idx);
            assert_ne!(bitmap[pos] & (1u64 << inner_pos), 0);
            bitmap[pos] -= 1u64 << inner_pos;
        });

        if is_dir {
            self.dirs_count -= 1;
        }
    }

    // 调用该函数必然成功, 所有的检查应该在外部完成
    // 在本 blockgroup 中尽力分配 num 个 block, 但是不一定能完成
    pub fn alloc_blocks(&mut self, num: usize) -> Vec<u32> {
        assert_ne!(num, 0);

        let mut vec = Vec::new();
        // 不能提前更新 free_blocks_count 因为不一定有 num 个满足
        block_device::modify(self.block_bitmap_bid(), 0, |bitmap: &mut BitmapBlock| {
            use core::ops::Not;
            for (pos, bits) in bitmap.iter_mut().enumerate() {
                let mut neg_bits = bits.not();
                while neg_bits != 0 {
                    let inner_pos = neg_bits.trailing_zeros() as usize;
                    *bits |= 1 << inner_pos;
                    // 不要忘记更新 free_blocks_count
                    self.free_blocks_count -= 1;
                    vec.push((pos * UNIT_WIDTH + inner_pos) as u32);

                    if vec.len() == num {
                        return vec;
                    }

                    neg_bits &= neg_bits - 1;
                }
            }

            // num 没有完全满足
            vec
        })
    }

    // 参数 bg_blocks 只是自己所管辖的 blockgroup 内的相对 block 而不是全局 block_id
    pub fn dealloc_blocks(&mut self, bg_blocks: &[u32]) {
        if bg_blocks.is_empty() {
            return;
        }

        // 提前批量更新 free_blocks_count
        self.free_blocks_count += bg_blocks.len() as u16;

        block_device::modify(self.block_bitmap_bid(), 0, |bitmap: &mut BitmapBlock| {
            for bg_bid in bg_blocks {
                let (pos, inner_pos) = self.decomposition(*bg_bid);
                assert_ne!(bitmap[pos] & (1u64 << inner_pos), 0);
                bitmap[pos] -= 1u64 << inner_pos;
            }
        });
    }
}

impl Debug for Ext2BlockGroupDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockGroupDescriptor")
            .field("block_bitmap_addr", &self.block_bitmap_addr)
            .field("inode_bitmap_addr", &self.inode_bitmap_addr)
            .field("inode_table_block", &self.inode_table_block)
            .field("free_blocks_count", &self.free_blocks_count)
            .field("free_inodes_count", &self.free_inodes_count)
            .field("dirs_count", &self.dirs_count)
            .finish()
    }
}
