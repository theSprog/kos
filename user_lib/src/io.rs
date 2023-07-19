// IOError, Read/Write & Seek trait

use alloc::{string::String, vec::Vec};
use core::fmt::Debug;
use sys_interface::syserr;

use crate::err_msg;
pub type IOResult<T> = Result<T, IOError>;

pub const SEEK_SET: usize = 0;
pub const SEEK_CURRENT: usize = 1;
pub const SEEK_END: usize = 2;

#[derive(Debug)]
pub enum SeekFrom {
    Start(usize),
    Current(isize),
    End(isize),
}

pub enum IOError {
    NotFound(isize),         // syserr::ENOENT
    PermissionDenied(isize), // syserr::EPERM
    AlreadyExists(isize),    // syserr::EEXIST
    NotADirectory(isize),    // syserr::ENOTDIR
    IsADirectory(isize),     // syserr::EISDIR,
    TooLargeFile(isize),     //  syserr::EFBIG,
    TooLongFileName(isize),  // syserr::ENAMETOOLONG,
    TooManyLinks(isize),     // syserr::EMLINK,
    InvalidArgument(isize),  // syserr::EINVAL,
    NoFreeSpace(isize),      // syserr::ENOSPC,
    BadSeek(isize),          // syserr::ESPIPE,
}

impl From<isize> for IOError {
    fn from(value: isize) -> Self {
        match value {
            syserr::ENOENT => Self::NotFound(syserr::ENOENT),
            syserr::EPERM => Self::PermissionDenied(syserr::EPERM),
            syserr::EEXIST => Self::AlreadyExists(syserr::EEXIST),
            syserr::ENOTDIR => Self::NotADirectory(syserr::ENOTDIR),
            syserr::EISDIR => Self::IsADirectory(syserr::EISDIR),
            syserr::EFBIG => Self::TooLargeFile(syserr::EFBIG),
            syserr::ENAMETOOLONG => Self::TooLongFileName(syserr::ENAMETOOLONG),
            syserr::EMLINK => Self::TooManyLinks(syserr::EMLINK),
            syserr::EINVAL => Self::InvalidArgument(syserr::EINVAL),
            syserr::ENOSPC => Self::NoFreeSpace(syserr::ENOSPC),
            syserr::ESPIPE => Self::BadSeek(syserr::ESPIPE),
            _ => todo!("value: {:?}", value),
        }
    }
}

impl From<&IOError> for isize {
    fn from(value: &IOError) -> Self {
        *match value {
            IOError::NotFound(errno) => errno,
            IOError::PermissionDenied(errno) => errno,
            IOError::AlreadyExists(errno) => errno,
            IOError::NotADirectory(errno) => errno,
            IOError::IsADirectory(errno) => errno,
            IOError::TooLargeFile(errno) => errno,
            IOError::TooLongFileName(errno) => errno,
            IOError::TooManyLinks(errno) => errno,
            IOError::InvalidArgument(errno) => errno,
            IOError::NoFreeSpace(errno) => errno,
            IOError::BadSeek(errno) => errno,
        }
    }
}

impl Debug for IOError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?}", err_msg(self.into()))
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize>;

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> IOResult<usize> {
        let mut temp_buf = alloc::vec![0; 4096]; // 临时缓冲区
        let mut total_bytes_read = 0;

        loop {
            let bytes_read = self.read(&mut temp_buf)?;
            if bytes_read == 0 {
                break;
            }
            buf.extend_from_slice(&temp_buf[..bytes_read]);
            total_bytes_read += bytes_read;
        }

        Ok(total_bytes_read)
    }

    fn read_to_string(&mut self, buf: &mut String) -> IOResult<usize> {
        let mut vec_buf = Vec::new();
        let bytes_read = self.read_to_end(&mut vec_buf)?;
        *buf = String::from_utf8_lossy(&vec_buf).into_owned();
        Ok(bytes_read)
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize>;
    fn flush(&mut self) -> IOResult<()> {
        todo!()
    }
    fn write_all(&mut self, mut buf: &[u8]) -> IOResult<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(n) => {
                    if n == 0 {
                        // 如果返回的写入字节数为0，表示到达了文件末尾或出现了错误
                        panic!("failed to write whole buffer");
                    }
                    // 更新缓冲区的偏移量
                    buf = &buf[n..];
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(())
    }
}



