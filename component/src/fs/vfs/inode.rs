use core::fmt::Debug;

use alloc::{boxed::Box, string::String};

use super::{
    error::VfsResult,
    meta::{VfsMetadata, VfsPermissions},
};

pub trait VfsInode: Debug {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize>;
    fn write_at(&mut self, offset: usize, buf: &[u8]) -> VfsResult<usize>;
    fn set_len(&mut self, len: usize) -> VfsResult<()>;

    fn metadata(&self) -> Box<dyn VfsMetadata>;

    fn set_permissions(&mut self, permissions: &VfsPermissions) -> VfsResult<()>;
    fn read_symlink(&self) -> VfsResult<String>;
}
