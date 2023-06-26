use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;

use super::block;
use super::vfs::error::{IOError, IOErrorKind, VfsResult};
use super::vfs::meta::{VfsFileType, VfsMetadata, VfsTimeStamp};
use super::vfs::VfsInode;
use super::{block_device, vfs::meta::VfsPermissions};

use super::address::Address;
use super::allocator::Ext2Allocator;
use super::disk_inode::Ext2Inode;
use super::layout::Ext2Layout;
use super::metadata::Ext2Metadata;

#[derive(Debug, Clone)]
pub struct Inode {
    address: Address,
    inode_id: usize,
    filetype: VfsFileType,

    layout: Arc<Ext2Layout>,
    allocator: Arc<Mutex<Ext2Allocator>>,

    parent_id: Option<usize>,
}
impl Inode {
    pub(crate) fn new(
        inode_id: usize,
        address: Address,
        filetype: VfsFileType,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Self {
        block_device::modify(
            address.block_id(),
            address.offset(),
            |ext2_inode: &mut Ext2Inode| ext2_inode.init(filetype),
        );

        Self {
            address,
            inode_id,
            filetype,
            layout,
            allocator,
            parent_id: None,
        }
    }

    pub(crate) fn read(
        inode_id: usize,
        address: Address,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        let filetype = block_device::read(
            address.block_id(),
            address.offset(),
            |ext2_inode: &Ext2Inode| ext2_inode.filetype(),
        );

        Self {
            address,
            inode_id,
            filetype,

            parent_id: None,
            layout,
            allocator,
        }
    }

    pub(crate) fn with_parent(self, parent_id: usize) -> Self {
        Self {
            parent_id: Some(parent_id),
            ..self
        }
    }

    pub fn inode_id(&self) -> usize {
        self.inode_id
    }

    pub fn parent_id(&self) -> usize {
        self.parent_id.unwrap()
    }

    pub fn layout(&self) -> Arc<Ext2Layout> {
        self.layout.clone()
    }

    pub fn allocator(&self) -> Arc<Mutex<Ext2Allocator>> {
        self.allocator.clone()
    }

    pub fn parent_inode(&self) -> Inode {
        self.layout
            .inode_nth(self.parent_id(), self.layout(), self.allocator())
    }

    pub fn size(&self) -> usize {
        block_device::read(
            self.address.block_id(),
            self.address.offset(),
            |disk_inode: &Ext2Inode| disk_inode.size(),
        )
    }

    pub fn timestamp(&self) -> VfsTimeStamp {
        block_device::read(
            self.address.block_id(),
            self.address.offset(),
            |disk_inode: &Ext2Inode| disk_inode.timestamp(),
        )
    }

    pub fn filetype(&self) -> VfsFileType {
        self.filetype
    }

    pub fn is_file(&self) -> bool {
        self.filetype.is_file()
    }
    pub fn is_dir(&self) -> bool {
        self.filetype.is_dir()
    }
    pub fn is_symlink(&self) -> bool {
        self.filetype.is_symlink()
    }

    fn block_id(&self) -> usize {
        self.address.block_id()
    }
    fn offset(&self) -> usize {
        self.address.offset()
    }

    pub(crate) fn read_disk_inode<V>(&self, f: impl FnOnce(&Ext2Inode) -> V) -> V {
        block_device::read(self.block_id(), self.offset(), f)
    }

    pub(crate) fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut Ext2Inode) -> V) -> V {
        block_device::modify(self.block_id(), self.offset(), f)
    }

    pub(crate) fn sync_disk_inode(&self) {
        block_device::sync(self.block_id());
    }

    pub fn metadata(&self) -> Ext2Metadata {
        self.read_disk_inode(|ext2_inode| {
            Ext2Metadata::new(
                ext2_inode.filetype(),
                ext2_inode.permissions(),
                ext2_inode.size(),
                ext2_inode.timestamp(),
                ext2_inode.uid(),
                ext2_inode.gid(),
                ext2_inode.hard_links(),
            )
        })
    }

    fn blocks_needed(old_size: usize, new_size: usize) -> usize {
        assert!(new_size > old_size);
        Ext2Inode::total_blocks(new_size) - Ext2Inode::total_blocks(old_size)
    }
    fn blocks_freed(old_size: usize, new_size: usize) -> usize {
        assert!(new_size < old_size);
        Ext2Inode::total_blocks(old_size) - Ext2Inode::total_blocks(new_size)
    }

    fn clear_from(&mut self, start: usize, len: usize) -> VfsResult<()> {
        assert!(start + len <= self.size());
        let buf = [0u8; block::SIZE];

        // 剩下要写入的字节数
        let mut rest = len;
        let mut offset = start;
        loop {
            let write_size = if rest < block::SIZE {
                let vec = alloc::vec![0u8; rest];
                self.write_at(offset, &vec)?
            } else {
                self.write_at(offset, &buf)?
            };
            rest -= write_size;
            if rest == 0 {
                break;
            }

            offset += write_size;
        }
        Ok(())
    }

    pub fn increase_to(&mut self, new_size: usize) -> VfsResult<()> {
        assert!(self.size() < new_size);
        let cur_offset = self.size();
        let needed_num = Self::blocks_needed(self.size(), new_size);
        let new_blocks = self.allocator.lock().alloc_data(needed_num)?;
        self.modify_disk_inode(|ext2_inode| {
            ext2_inode.increase_to(new_size, new_blocks);
        });
        // 扩充的空间用 0 填充
        self.clear_from(cur_offset, new_size - cur_offset)?;

        Ok(())
    }

    pub fn decrease_to(&mut self, new_size: usize) -> VfsResult<()> {
        assert!(
            self.size() > new_size,
            "now_size: {}, new_size: {}",
            self.size(),
            new_size
        );
        let freed_num = Self::blocks_freed(self.size(), new_size);
        let freed = self.modify_disk_inode(|ext2_inode| ext2_inode.decrease_to(new_size));
        assert_eq!(freed.len(), freed_num);

        self.allocator.lock().dealloc_data(freed)?;

        Ok(())
    }
}

impl VfsInode for Inode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
        Ok(self.read_disk_inode(|ext2_inode| ext2_inode.read_at(offset, buf)))
    }

    fn write_at(&mut self, offset: usize, buf: &[u8]) -> VfsResult<usize> {
        // 如果当前 size 不够则需要先扩容
        let end_offset = offset + buf.len();
        if self.size() < end_offset {
            self.increase_to(end_offset)?;
        }

        Ok(self.modify_disk_inode(|disk_inode| disk_inode.write_at(offset, buf)))
    }

    fn set_len(&mut self, len: usize) -> VfsResult<()> {
        use core::cmp::Ordering;
        match self.size().cmp(&len) {
            Ordering::Less => self.increase_to(len),
            Ordering::Equal => Ok(()),
            Ordering::Greater => self.decrease_to(len),
        }
    }

    fn metadata(&self) -> Box<dyn VfsMetadata> {
        // 有趣的是, 如果函数重名(比如这里的 metadata 和 Inode 的 metadata)
        // 并不会发生冲突, 而是结构体方法优先
        Box::new(self.metadata())
    }

    fn set_permissions(&mut self, permissions: &VfsPermissions) {
        self.modify_disk_inode(|disk_inode| disk_inode.set_permissions(permissions));
    }

    fn read_symlink(&self) -> VfsResult<String> {
        if !self.is_symlink() {
            return Err(IOError::new(IOErrorKind::NotASymlink).into());
        }
        Ok(self.read_symlink())
    }
}
