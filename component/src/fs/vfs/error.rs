//! Error and Result definitions

use crate::alloc::string::ToString;
use core::fmt;

use alloc::string::String;

pub type VfsResult<T> = core::result::Result<T, VfsError>;

#[derive(Debug)]
pub struct VfsError {
    path: String,
    additional: String,
    kind: VfsErrorKind,
}

impl VfsError {
    pub fn new<T: Into<String>>(path: T, kind: VfsErrorKind, additional: String) -> VfsError {
        VfsError {
            path: path.into(),
            additional,
            kind,
        }
    }
}

impl From<VfsErrorKind> for VfsError {
    fn from(kind: VfsErrorKind) -> Self {
        let path = match &kind {
            VfsErrorKind::IOError(io_err) => io_err.path().to_string(),
            _ => "NOT FILLED BY VFS LAYER".into(),
        };

        Self::new(path, kind, "AN ERROR OCCURRED".into())
    }
}

impl From<IOError> for VfsError {
    fn from(err: IOError) -> Self {
        Self::from(VfsErrorKind::IOError(err))
    }
}

impl VfsError {
    // Path filled by the VFS crate rather than the implementations
    pub(crate) fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_additional<T: ToString>(mut self, additional: T) -> Self {
        self.additional = additional.to_string();
        self
    }

    pub fn kind(&self) -> &VfsErrorKind {
        &self.kind
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} for '{}': {}",
            self.additional,
            self.path,
            self.kind()
        )
    }
}

/// The kinds of errors that can occur
#[derive(Debug)]
pub enum VfsErrorKind {
    /// A generic I/O error
    ///
    /// Certain standard I/O errors are normalized to their VfsErrorKind counterparts
    IOError(IOError),

    // FSError(FSError),
    /// The file or directory at the given path could not be found
    FileNotFound,

    /// The given path is invalid, e.g. because contains '.' or '..'
    InvalidPath(String),

    /// There is already a directory at the given path
    DirectoryExists,

    /// There is already a file at the given path
    FileExists,

    /// Functionality not supported by this filesystem
    NotSupported,

    /// Generic error variant
    Other(String),
}

impl fmt::Display for VfsErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VfsErrorKind::IOError(err) => {
                write!(f, "IO error: {:?}", err)
            }
            // VfsErrorKind::FSError(err) => {
            //     write!(f, "FS error: {:?}", err)
            // }
            VfsErrorKind::FileNotFound => {
                write!(f, "The file or directory could not be found")
            }
            VfsErrorKind::InvalidPath(path) => {
                write!(f, "The path is invalid: {}", path)
            }
            VfsErrorKind::Other(msg) => {
                write!(f, "FileSystem error: {}", msg)
            }
            VfsErrorKind::NotSupported => {
                write!(f, "Functionality not supported by this filesystem")
            }
            VfsErrorKind::DirectoryExists => {
                write!(f, "Directory already exists")
            }
            VfsErrorKind::FileExists => {
                write!(f, "File already exists")
            }
        }
    }
}

#[derive(Debug)]
pub struct IOError {
    kind: IOErrorKind,
    path: String,
}

impl IOError {
    pub fn new(kind: IOErrorKind) -> Self {
        Self {
            kind,
            path: String::new(),
        }
    }

    pub fn with_path<T: Into<String>>(self, path: T) -> Self {
        Self {
            path: path.into(),
            ..self
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn kind(&self) -> &IOErrorKind {
        &self.kind
    }
}

#[derive(Debug)]
pub enum IOErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    NotASymlink,
    TooLongTargetSymlink,
    DirectoryNotEmpty,
    IsADirectory,
    TooLargeFile,
    TooLongFileName,
    TooManyLinks,
    InvalidFilename,
    NoFreeBlocks,
    NoFreeInodes,
}
