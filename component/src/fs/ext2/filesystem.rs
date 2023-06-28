use core::fmt::{self, Display};

use alloc::{boxed::Box, string::ToString, sync::Arc, vec::Vec};
use logger::debug;
use spin::Mutex;

use super::block_device::{self, BlockDevice};

use super::superblock::{FS_CLEAN, FS_UNKNOWN};
use super::vfs::error::{IOError, IOErrorKind, VfsResult};
use super::vfs::meta::*;
use super::vfs::FileSystem;
use super::vfs::{VfsDirEntry, VfsInode, VfsPath};

use super::{
    allocator::Ext2Allocator, blockgroup::Ext2BlockGroupDesc, inode::Inode, layout::Ext2Layout,
    superblock::Superblock,
};

#[derive(Debug)]
pub struct Ext2FileSystem {
    layout: Arc<Ext2Layout>,
    allocator: Arc<Mutex<Ext2Allocator>>,
}

impl Display for Ext2FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self.layout)
    }
}

impl Ext2FileSystem {
    pub fn open(block_dev: impl BlockDevice) -> Self {
        block_device::register_block_device(block_dev);
        let superblock = block_device::read(0, 1024, |sb: &Superblock| {
            sb.check_valid();
            sb.clone()
        });

        let blockgroup_count = superblock.blockgroup_count();
        let blockgroups = Ext2BlockGroupDesc::find(blockgroup_count);

        let layout = Arc::new(Ext2Layout::new(superblock, blockgroups));
        let allocator = Arc::new(Mutex::new(Ext2Allocator::new(layout.clone())));

        Self { layout, allocator }
    }

    fn flush(&self) {
        self.layout.flush();
    }

    fn root_inode(&self) -> Inode {
        self.layout
            .root_inode(self.layout.clone(), self.allocator.clone())
    }
}

impl FileSystem for Ext2FileSystem {
    fn read_dir(&self, path: VfsPath) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        let root_inode: Inode = self.root_inode();
        let target = root_inode.walk(&path)?;
        target
            .read_dir()
            .map_err(|err| err.with_path(path.to_string()))
    }

    fn exists(&self, path: VfsPath) -> VfsResult<bool> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path);
        Ok(target.is_ok())
    }

    fn metadata(&self, path: VfsPath) -> VfsResult<Box<dyn VfsMetadata>> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path)?;
        Ok(Box::new(target.metadata()))
    }

    fn link(&self, to: VfsPath, from: VfsPath) -> VfsResult<()> {
        let root_inode = self.root_inode();
        // to 必须要存在
        let target = root_inode.walk(&to)?;
        let mut dir_inode = root_inode.walk(&from.parent())?;
        let child = dir_inode.select_child(from.last().unwrap());
        if child.is_err() {
            // child 尚不存在, 需要在当前 dir 下新建
            dir_inode.insert_hardlink(&from, &to, &target)?;
        } else {
            let mut child = child.unwrap();
            if child.is_dir() {
                // child 已存在且是 dir, 则在该 dir 下新建同名符号链接
                let mut new_from = from.clone();
                new_from.push(to.last().unwrap());
                child.insert_hardlink(&new_from, &to, &target)?;
            } else {
                // child 已存在但不是 dir, 则是 AlreadyExists Error
                return Err(IOError::new(IOErrorKind::AlreadyExists)
                    .with_path(&from)
                    .into());
            }
        }

        Ok(())
    }

    fn symlink(&self, to: VfsPath, from: VfsPath) -> VfsResult<()> {
        let root_inode = self.root_inode();
        let mut dir_inode = root_inode.walk(&from.parent())?;

        dir_inode.insert_symlink(&from, &to)
    }

    fn open_file(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path)?;
        Ok(Box::new(target))
    }

    fn create_file(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>> {
        let root_inode = self.root_inode();
        let mut dir_inode = root_inode.walk(&path.parent())?;
        dir_inode.insert_entry(&path, VfsFileType::RegularFile)
    }

    fn create_dir(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>> {
        let root_inode = self.root_inode();
        let mut dir_inode = root_inode.walk(&path.parent())?;
        dir_inode.insert_entry(&path, VfsFileType::Directory)
    }

    fn remove_file(&self, path: VfsPath) -> VfsResult<()> {
        let root_inode = self.root_inode();
        let mut dir_inode = root_inode.walk(&path.parent())?;
        dir_inode.remove_entry(&path)
    }

    fn remove_dir(&self, path: VfsPath) -> VfsResult<()> {
        let root_inode = self.root_inode();
        let mut dir_inode = root_inode.walk(&path.parent())?;
        dir_inode.remove_entry(&path)
    }

    fn flush(&self) {
        self.flush();
    }
}
