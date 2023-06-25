use core::fmt::Debug;
use core::fmt::Display;

pub trait VfsMetadata: Debug + Display + 'static {
    fn filetype(&self) -> VfsFileType;
    fn permissions(&self) -> VfsPermissions;
    fn size(&self) -> u64;
    fn timestamp(&self) -> VfsTimeStamp;
    fn uid(&self) -> u16;
    fn gid(&self) -> u16;
    fn hard_links(&self) -> u16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsFileType {
    RegularFile,
    Directory,
    CharDev,
    BlockDev,
    FIFO,
    Socket,
    SymbolicLink,
}

impl VfsFileType {
    pub fn is_file(&self) -> bool {
        self == &VfsFileType::RegularFile
    }
    pub fn is_dir(&self) -> bool {
        self == &VfsFileType::Directory
    }
    pub fn is_symlink(&self) -> bool {
        self == &VfsFileType::SymbolicLink
    }
}

impl Display for VfsFileType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VfsFileType::RegularFile => write!(f, "."),
            VfsFileType::Directory => write!(f, "d"),
            VfsFileType::FIFO => write!(f, "f"),
            VfsFileType::SymbolicLink => write!(f, "l"),
            _ => todo!(),
            // VfsFileType::CharDev => write!(f, "CharDev"),
            // VfsFileType::BlockDev => write!(f, "BlockDev"),
            // VfsFileType::Socket => write!(f, "Socket"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VfsPermissions {
    user: VfsPermission,
    group: VfsPermission,
    others: VfsPermission,
}

impl VfsPermissions {
    pub fn new<T: Into<VfsPermission>>(user: T, group: T, others: T) -> Self {
        Self {
            user: user.into(),
            group: group.into(),
            others: others.into(),
        }
    }

    pub fn empty() -> Self {
        Self {
            user: VfsPermission::empty(),
            group: VfsPermission::empty(),
            others: VfsPermission::empty(),
        }
    }

    pub fn all() -> Self {
        Self {
            user: VfsPermission::all(),
            group: VfsPermission::all(),
            others: VfsPermission::all(),
        }
    }

    // 单独修改
    pub fn with_user<T: Into<VfsPermission>>(self, user: T) -> Self {
        Self {
            user: user.into(),
            ..self
        }
    }
    pub fn with_group<T: Into<VfsPermission>>(self, group: T) -> Self {
        Self {
            group: group.into(),
            ..self
        }
    }
    pub fn with_others<T: Into<VfsPermission>>(self, others: T) -> Self {
        Self {
            others: others.into(),
            ..self
        }
    }

    pub fn user(&self) -> VfsPermission {
        self.user
    }
    pub fn group(&self) -> VfsPermission {
        self.group
    }
    pub fn others(&self) -> VfsPermission {
        self.others
    }
}

impl Display for VfsPermissions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.user)?;
        write!(f, "{}", self.group)?;
        write!(f, "{}", self.others)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VfsPermission {
    read: bool,
    write: bool,
    execute: bool,
}

impl VfsPermission {
    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        Self {
            read,
            write,
            execute,
        }
    }

    pub fn empty() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }

    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }

    pub fn read(&self) -> bool {
        self.read
    }
    pub fn write(&self) -> bool {
        self.write
    }
    pub fn execute(&self) -> bool {
        self.execute
    }
}

impl From<u8> for VfsPermission {
    fn from(value: u8) -> Self {
        Self {
            read: (value & 0x4) != 0,
            write: (value & 0x2) != 0,
            execute: (value & 0x1) != 0,
        }
    }
}

impl Display for VfsPermission {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.read { "r" } else { "-" },
            if self.write { "w" } else { "-" },
            if self.execute { "x" } else { "-" }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VfsTimeStamp {
    atime: u64,
    mtime: u64,
    ctime: u64,
    dtime: u64,
}

impl VfsTimeStamp {
    pub fn new(atime: u64, ctime: u64, mtime: u64, dtime: u64) -> VfsTimeStamp {
        Self {
            atime,
            mtime,
            ctime,
            dtime,
        }
    }
    pub fn atime(&self) -> u64 {
        self.atime
    }
    pub fn mtime(&self) -> u64 {
        self.mtime
    }
    pub fn ctime(&self) -> u64 {
        self.ctime
    }
    pub fn dtime(&self) -> u64 {
        self.dtime
    }
}
