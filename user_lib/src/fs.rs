use crate::{
    close, err_msg,
    io::{IOError, IOResult, Read, SeekFrom, Write, SEEK_CURRENT, SEEK_END, SEEK_SET},
    lseek, open, read, write, OpenFlags,
};

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, IOError>;
    fn reset(&mut self) -> Result<(), IOError>;
}

#[derive(Debug)]
pub struct File {
    fd: usize,
}

impl Drop for File {
    fn drop(&mut self) {
        let err = close(self.fd);
        if err < 0 {
            panic!("{}", err_msg(err));
        }
    }
}

impl File {
    pub fn new(fd: usize) -> File {
        Self { fd }
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, IOError> {
        let sys_res = match pos {
            SeekFrom::Start(offset) => lseek(self.fd, offset as isize, SEEK_SET),
            SeekFrom::Current(offset) => lseek(self.fd, offset, SEEK_CURRENT),
            SeekFrom::End(offset) => lseek(self.fd, offset, SEEK_END),
        };
        match sys_res {
            0 => Ok(sys_res as u64),
            err => Err(sys_res.into()),
        }
    }

    fn reset(&mut self) -> Result<(), IOError> {
        match lseek(self.fd, 0, SEEK_SET) {
            0 => Ok(()),
            err => Err(err.into()),
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        let read_result = read(self.fd, buf);
        if read_result < 0 {
            return Err(IOError::from(read_result));
        }
        Ok(read_result as usize)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        let write_result = write(self.fd, buf);
        if write_result < 0 {
            return Err(IOError::from(write_result));
        }
        Ok(write_result as usize)
    }
}

pub struct OpenOptions {
    flags: OpenFlags,
    mode: u16,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            flags: OpenFlags::RDONLY,
            mode: 0,
        }
    }

    pub fn read(mut self, read: bool) -> Self {
        if read {
            self.flags |= OpenFlags::RDONLY;
        }
        self
    }

    pub fn write(mut self, write: bool) -> Self {
        if write {
            self.flags |= OpenFlags::WRONLY;
        }
        self
    }

    pub fn append(mut self, append: bool) -> Self {
        if append {
            self.flags |= OpenFlags::APPEND;
        }
        self
    }

    pub fn trunc(mut self, trunc: bool) -> Self {
        if trunc {
            self.flags |= OpenFlags::TRUNC;
        }
        self
    }

    pub fn create(mut self, create: bool, mode: u16) -> Self {
        if create {
            self.flags |= OpenFlags::CREATE;
            self.mode = mode;
        }
        self
    }

    pub fn open<T: AsRef<str>>(&self, path: T) -> IOResult<File> {
        let path = path.as_ref();
        let err = open(path, self.flags.clone(), self.mode);
        if err < 0 {
            return Err(err.into());
        }
        let fd = err as usize;
        Ok(File::new(fd))
    }
}
