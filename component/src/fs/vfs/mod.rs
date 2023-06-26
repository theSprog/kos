mod dir;
mod filesystem;
mod inode;
mod io;
mod path;

pub mod error;
pub mod meta;

use core::fmt::Display;

use alloc::{boxed::Box, string::ToString, vec::Vec};

pub use dir::VfsDirEntry;
pub use error::*;
pub use filesystem::FileSystem;
pub use inode::VfsInode;
pub use path::VfsPath;

use crate::fs::block_device;

use self::meta::VfsMetadata;

#[derive(Debug)]
pub struct VirtualFileSystem {
    fs: Box<dyn FileSystem>,
}

impl Display for VirtualFileSystem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fs)
    }
}

impl VirtualFileSystem {
    pub fn new(fs: impl FileSystem) -> Self {
        Self { fs: Box::new(fs) }
    }

    fn parse_path(path: &str) -> VfsResult<VfsPath> {
        if !path.starts_with('/') {
            return Err(VfsErrorKind::InvalidPath(path.to_string()).into());
        }

        Ok(VfsPath::from(path))
    }

    pub fn read_dir<T: AsRef<str>>(&self, path: T) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.read_dir(vpath)
    }

    pub fn exists<T: AsRef<str>>(&self, path: T) -> VfsResult<bool> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.exists(vpath)
    }

    pub fn metadata<T: AsRef<str>>(&self, path: T) -> VfsResult<Box<dyn VfsMetadata>> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.metadata(vpath)
    }

    pub fn link<T: AsRef<str>>(&self, to_path: T, from_path: T) -> VfsResult<()> {
        let vpath_to = Self::parse_path(to_path.as_ref())?;
        let vpath_from = Self::parse_path(from_path.as_ref())?;
        self.fs.link(vpath_to, vpath_from)
    }

    pub fn symlink<T: AsRef<str>>(&self, to_path: T, from_path: T) -> VfsResult<()> {
        let vpath_to = Self::parse_path(to_path.as_ref())?;
        let vpath_from = Self::parse_path(from_path.as_ref())?;
        self.fs.symlink(vpath_to, vpath_from)
    }

    pub fn open_file<T: AsRef<str>>(&self, path: T) -> VfsResult<Box<dyn VfsInode>> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.open_file(vpath)
    }

    pub fn create_file<T: AsRef<str>>(&self, path: T) -> VfsResult<Box<dyn VfsInode>> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.create_file(vpath)
    }

    pub fn create_dir<T: AsRef<str>>(&self, path: T) -> VfsResult<Box<dyn VfsInode>> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.create_dir(vpath)
    }

    pub fn remove_file<T: AsRef<str>>(&self, path: T) -> VfsResult<()> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.remove_file(vpath)
    }

    pub fn remove_dir<T: AsRef<str>>(&self, path: T) -> VfsResult<()> {
        let vpath = Self::parse_path(path.as_ref())?;
        // 在本文件系统下删除根目录是不允许的
        if vpath.is_empty() {
            let err: VfsError = VfsErrorKind::InvalidPath(path.as_ref().to_string()).into();
            return Err(err.with_additional("Forbidden to remove root directory!"));
        }
        self.fs.remove_dir(vpath)
    }

    pub fn flush(&self) {
        self.fs.flush();
        block_device::flush();
    }
}

impl Drop for VirtualFileSystem {
    fn drop(&mut self) {
        self.flush();
    }
}
