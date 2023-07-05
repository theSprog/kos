pub mod inode;
pub mod pipe;
pub mod stdio;
pub mod userbuf;

pub use userbuf::UserBuffer;

use crate::driver::block::BlockDeviceImpl;
use crate::vfs::meta::VfsMetadata;
use crate::vfs::VfsError;
use crate::vfs::VfsErrorKind;
use crate::vfs::VirtualFileSystem;
use crate::KernelFileSystem;
use alloc::boxed::Box;
use component::fs::block_device;
use logger::info;

lazy_static! {
    pub static ref VFS: VirtualFileSystem = {
        info!("VirtualFileSystem initializing...");
        block_device::register_block_device(BlockDeviceImpl::new());
        let kfs = KernelFileSystem::new();
        VirtualFileSystem::new(kfs)
    };
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

    fn metadata(&self) -> Result<Box<dyn VfsMetadata>, VfsError> {
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
