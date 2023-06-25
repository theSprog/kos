use core::fmt::{self, Display};

use super::vfs::meta::{VfsFileType, VfsMetadata, VfsPermissions, VfsTimeStamp};

#[derive(Debug)]
pub struct Ext2Metadata {
    filetype: VfsFileType,
    permissions: VfsPermissions,
    size: usize,
    timestamp: VfsTimeStamp,
    uid: u16,
    gid: u16,
    hard_links: u16,
}
impl Ext2Metadata {
    pub fn new(
        filetype: VfsFileType,
        permissions: VfsPermissions,
        size: usize,
        timestamp: VfsTimeStamp,
        uid: u16,
        gid: u16,
        hard_links: u16,
    ) -> Self {
        Self {
            filetype,
            permissions,
            size,
            timestamp,
            uid,
            gid,
            hard_links,
        }
    }
}

impl Display for Ext2Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ext2Metadata")
    }
}

impl VfsMetadata for Ext2Metadata {
    fn filetype(&self) -> VfsFileType {
        self.filetype
    }

    fn permissions(&self) -> VfsPermissions {
        self.permissions
    }

    fn size(&self) -> u64 {
        self.size as u64
    }

    fn timestamp(&self) -> VfsTimeStamp {
        self.timestamp
    }

    fn uid(&self) -> u16 {
        self.uid
    }

    fn gid(&self) -> u16 {
        self.gid
    }

    fn hard_links(&self) -> u16 {
        self.hard_links
    }
}
