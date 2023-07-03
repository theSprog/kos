use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use crate::cast_mut;

use super::{block::DataBlock, block_device, vfs::meta::VfsFileType};

use super::{
    allocator::Ext2Allocator, blockgroup::Ext2BlockGroupDesc, inode::Inode, superblock::Superblock,
};

#[derive(Debug)]
pub struct Ext2Layout {
    blocks_per_group: u32,
    inodes_per_group: u32,

    superblock: Arc<Mutex<Superblock>>,
    blockgroups: Arc<Vec<Mutex<Ext2BlockGroupDesc>>>,
}

impl Ext2Layout {
    pub fn new(superblock: Superblock, blockgroups: Vec<Ext2BlockGroupDesc>) -> Self {
        let blocks_per_group = superblock.blocks_per_group;
        let inodes_per_group = superblock.inodes_per_group;

        let superblock = Arc::new(Mutex::new(superblock));
        // 为每一个成员加上锁
        let blockgroups = Arc::new(blockgroups.into_iter().map(Mutex::new).collect::<Vec<_>>());

        Self {
            blocks_per_group,
            inodes_per_group,
            superblock,
            blockgroups,
        }
    }

    pub fn flush(&self) {
        block_device::modify(0, 1024, |sb: &mut Superblock| {
            sb.clone_from(&self.superblock.lock());
        });

        block_device::modify(1, 0, |data: &mut DataBlock| {
            let bg_size = core::mem::size_of::<Ext2BlockGroupDesc>();
            for (idx, bg) in self.blockgroups.iter().enumerate() {
                let dst = &mut data[idx * bg_size..];
                let disk_bg = cast_mut!(dst.as_mut_ptr(), Ext2BlockGroupDesc);
                disk_bg.clone_from(&bg.lock())
            }
        });
    }

    pub fn superblock(&self) -> Arc<Mutex<Superblock>> {
        self.superblock.clone()
    }

    pub fn blockgroups(&self) -> Arc<Vec<Mutex<Ext2BlockGroupDesc>>> {
        self.blockgroups.clone()
    }

    pub fn blocks_per_group(&self) -> u32 {
        self.blocks_per_group
    }
    pub fn inodes_per_group(&self) -> u32 {
        self.inodes_per_group
    }

    pub fn root_inode(
        &self,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        self.inode_nth(2, layout, allocator).with_parent(2)
    }

    pub fn inode_nth(
        &self,
        inode_id: usize,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        // 拿到所在 block_group 和 inode 内部偏移量
        let (blockgroup_idx, inode_inner_idx) = self.inode_idx(inode_id);
        let bg = self.blockgroups.get(blockgroup_idx).unwrap().lock();
        bg.get_inode(inode_id, inode_inner_idx, layout, allocator)
    }

    pub fn new_inode_nth(
        &self,
        inode_id: usize,
        filetype: VfsFileType,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        let (blockgroup_idx, inode_inner_idx) = self.inode_idx(inode_id);
        let bg = self.blockgroups.get(blockgroup_idx).unwrap().lock();
        bg.new_inode(inode_id, inode_inner_idx, filetype, layout, allocator)
    }

    fn inode_idx(&self, inode_id: usize) -> (usize, usize) {
        let inode_seq: usize = inode_id - 1;
        let blockgroup_idx = inode_seq / self.inodes_per_group as usize;
        let inode_innner_idx = inode_seq % self.inodes_per_group as usize;
        (blockgroup_idx, inode_innner_idx)
    }
}
