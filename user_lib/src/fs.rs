// use crate::io;

use crate::io::{IOError, IOResult, Read, SeekFrom, Write};

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, IOError>;
    fn reset(&mut self) -> Result<(), IOError>;
}

pub struct File {
    fd: usize,
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, IOError> {
        todo!("{:?}", pos);
    }

    fn reset(&mut self) -> Result<(), IOError> {
        todo!()
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        todo!()
    }

    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> IOResult<usize> {
        todo!()
    }

    fn read_to_string(&mut self, buf: &mut alloc::string::String) -> IOResult<usize> {
        todo!()
    }
}
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        todo!()
    }

    fn flush(&mut self) -> IOResult<()> {
        todo!()
    }
}

pub struct OpenOptions {}
impl OpenOptions {
    pub fn new() -> Self {
        todo!()
    }
}
