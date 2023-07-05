// IOError, Read/Write & Seek trait

use alloc::{string::String, vec::Vec};

pub const SEEK_SET: usize = 0;
pub const SEEK_CURRENT: usize = 1;
pub const SEEK_END: usize = 2;

#[derive(Debug)]
pub enum SeekFrom {
    Start(usize),
    Current(isize),
    End(isize),
}

pub enum IOError {}

pub type IOResult<T> = Result<T, IOError>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize>;
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> IOResult<usize>;
    fn read_to_string(&mut self, buf: &mut String) -> IOResult<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize>;
    fn flush(&mut self) -> IOResult<()>;
    fn write_all(&mut self, mut buf: &[u8]) -> IOResult<()> {
        todo!()
    }
}
