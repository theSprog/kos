use crate::vfs::meta::VfsMetadata;
use crate::vfs::IOError;
use crate::vfs::IOErrorKind;
use crate::vfs::VfsError;
use crate::vfs::VfsInode;
use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::Mutex;

use super::SeekFrom;
use super::{File, UserBuffer};

pub struct OSInodeInner {
    offset: usize,
    inode: Box<dyn VfsInode>,
}

pub struct OSInode {
    readable: bool,
    writable: bool,
    // 通过 mutex, OSInodeInner 变成可变的了, 可以向内读写数据
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

    pub fn set_offset(&self, offset: usize) {
        self.inner.lock().offset = offset;
    }

    pub fn read_all(&self) -> Vec<u8> {
        let inner = self.inner.lock();
        let mut ret = alloc::vec![0; inner.inode.metadata().size() as usize];
        let len = inner
            .inode
            .read_at(inner.offset, &mut ret[inner.offset..])
            .unwrap();
        assert_eq!(len, ret.len());
        ret
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
            let offset = inner.offset;
            let read_size = inner.inode.read_at(offset, *slice)?;
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

    fn truncate(&self, length: usize) -> Result<(), VfsError> {
        let mut inner = self.inner.lock();
        inner.inode.set_len(length)?;
        Ok(())
    }

    fn seek(&self, offset: isize, whence: usize) -> Result<(), VfsError> {
        let whence = SeekFrom::from(whence);
        match whence {
            SeekFrom::Start => {
                if offset < 0 {
                    return Err(IOError::new(IOErrorKind::BadSeek).into());
                }
                self.set_offset(offset as usize);
            }
            SeekFrom::Current => {
                let cur_offset = self.inner.lock().offset as isize;
                if cur_offset + offset < 0 {
                    return Err(IOError::new(IOErrorKind::BadSeek).into());
                }
                self.set_offset((cur_offset + offset) as usize);
            }
            SeekFrom::End => {
                let meta = self.inner.lock().inode.metadata();
                let end_offset = meta.size() as isize;
                if end_offset + offset < 0 {
                    return Err(IOError::new(IOErrorKind::BadSeek).into());
                }
                self.set_offset((end_offset + offset) as usize);
            }
        }
        Ok(())
    }

    fn metadata(&self) -> Result<Box<dyn VfsMetadata>, VfsError> {
        Ok(self.inner.lock().inode.metadata())
    }
}
