use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
        const APPEND = 1 << 11;  // Append to the end of the file
        const NONBLOCK = 1 << 12;  // Non-blocking mode
        const SYNC = 1 << 13;  // Synchronous I/O
        const EXCLUSIVE = 1 << 14;  // Exclusive file access
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

    pub fn append(&self) -> bool {
        self.contains(OpenFlags::APPEND)
    }
}
