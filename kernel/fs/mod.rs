use crate::driver::block::BlockDeviceImpl;
use crate::vfs::VfsError;
use crate::vfs::VirtualFileSystem;
use crate::KernelFileSystem;
use alloc::vec::Vec;
use component::fs::vfs::VfsErrorKind;
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
}

pub fn init() {
    VFS.flush();
}
