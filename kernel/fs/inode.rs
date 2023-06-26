use crate::driver::block::BlockDeviceImpl;
use crate::vfs::VfsError;
use crate::vfs::VfsInode;
use crate::KernelFileSystem;
use alloc::{boxed::Box, sync::Arc};
use bitflags::bitflags;
use component::fs::vfs::VirtualFileSystem;
use spin::Mutex;

use super::{File, UserBuffer};

lazy_static! {
    pub static ref VFS: VirtualFileSystem = {
        let kfs = KernelFileSystem::open(BlockDeviceImpl::new());
        VirtualFileSystem::new(kfs)
    };
}

pub struct OSInodeInner {
    offset: usize,
    inode: Box<dyn VfsInode>,
}

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: Mutex<OSInodeInner>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Box<dyn VfsInode>) -> Self {
        Self {
            readable,
            writable,
            inner: Mutex::new(OSInodeInner { offset: 0, inode }),
        }
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }

    fn read(&self, mut buf: UserBuffer) -> Result<usize, VfsError> {
        // 两个进程无法同时访问同个文件
        let mut inner = self.inner.lock();
        let mut total_read_size = 0usize;

        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice)?;
            // 无数据可读
            if read_size == 0 {
                break;
            }
            // 记录偏移量
            inner.offset += read_size;
            total_read_size += read_size;
        }

        Ok(total_read_size)
    }

    fn write(&self, buf: UserBuffer) -> Result<usize, VfsError> {
        // 两个进程无法同时访问同个文件
        let mut inner = self.inner.lock();
        let mut total_write_size = 0usize;

        for slice in buf.buffers.iter() {
            let offset = inner.offset;
            let write_size = inner.inode.write_at(offset, *slice)?;
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }

        Ok(total_write_size)
    }
}

bitflags! {
    #[derive(Debug, Clone)]
    pub struct OpenFlags: u32 {
        // 只读
        const RDONLY = 0;
        // 只写
        const WRONLY = 1 << 0;
        // 可读可写
        const RDWR = 1 << 1;
        // 不存在则创建
        const CREATE = 1 << 9;
        // 清空
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    pub fn read(&self) -> bool {
        // 可读可写 || 只读
        self.contains(OpenFlags::RDWR) || self.contains(OpenFlags::RDONLY)
    }
    pub fn write(&self) -> bool {
        // 可读可写 || 只写
        self.contains(OpenFlags::RDWR) || self.contains(OpenFlags::WRONLY)
    }
    pub fn create(&self) -> bool {
        self.contains(OpenFlags::CREATE)
    }
    pub fn truncate(&self) -> bool {
        self.contains(OpenFlags::TRUNC)
    }
}
