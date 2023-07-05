use crate::driver::block::BlockDeviceImpl;
use crate::vfs::VfsError;
use crate::vfs::VirtualFileSystem;
use crate::KernelFileSystem;
use alloc::vec::Vec;
use component::fs::vfs::VfsErrorKind;
use core::ops::Deref;
use core::ops::DerefMut;
use logger::info;

lazy_static! {
    pub static ref VFS: VirtualFileSystem = {
        info!("VirtualFileSystem initializing...");
        let kfs = KernelFileSystem::open(BlockDeviceImpl::new());
        VirtualFileSystem::new(kfs)
    };
}

pub mod inode;
pub mod stdio;

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }

    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}

impl Deref for UserBuffer {
    type Target = Vec<&'static mut [u8]>;

    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

impl DerefMut for UserBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffers
    }
}

pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> Result<usize, VfsError>;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> Result<usize, VfsError>;

    /// default untrancable
    fn truncate(&self, length: usize) -> Result<(), VfsError> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn seek(&self, offset: isize, whence: usize) -> Result<(), VfsError> {
        Err(VfsErrorKind::NotSupported.into())
    }
}

pub enum SeekFrom {
    Start = 0,
    Current = 1,
    End = 2,
}

impl From<usize> for SeekFrom {
    fn from(value: usize) -> Self {
        match value {
            0 => SeekFrom::Start,
            1 => SeekFrom::Current,
            2 => SeekFrom::End,
            _ => panic!("Why got {}", value),
        }
    }
}

pub fn init() {
    VFS.init();
}
