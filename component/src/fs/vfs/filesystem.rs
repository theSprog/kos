use core::fmt::{Debug, Display};

use alloc::{boxed::Box, vec::Vec};

use super::{
    dir::VfsDirEntry,
    error::{VfsErrorKind, VfsResult},
    inode::VfsInode,
    meta::VfsMetadata,
    path::VfsPath,
};

pub trait FileSystem: Debug + Display + Sync + Send + 'static {
    fn read_dir(&self, path: VfsPath) -> VfsResult<Vec<Box<dyn VfsDirEntry>>>;
    fn exists(&self, path: VfsPath) -> VfsResult<bool>;
    fn metadata(&self, path: VfsPath) -> VfsResult<Box<dyn VfsMetadata>>;
    fn link(&self, to: VfsPath, from: VfsPath) -> VfsResult<()>;
    fn symlink(&self, to: VfsPath, from: VfsPath) -> VfsResult<()>;
    fn open_file(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>>;
    fn create_file(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>>;
    fn remove_file(&self, path: VfsPath) -> VfsResult<()>;
    fn create_dir(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>>;
    fn remove_dir(&self, path: VfsPath) -> VfsResult<()>;

    fn move_file(&self, src: &str, dest: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    // / Copies the src path to the destination path within the same filesystem (optional)
    // fn copy_file(&self, _src: &str, _dest: &str) -> VfsResult<()> {
    //     Err(VfsErrorKind::NotSupported.into())
    // }
    // /// Moves the src path to the destination path within the same filesystem (optional)

    // /// Moves the src directory to the destination path within the same filesystem (optional)
    // fn move_dir(&self, _src: &str, _dest: &str) -> VfsResult<()> {
    //     Err(VfsErrorKind::NotSupported.into())
    // }

    fn flush(&self);
}
