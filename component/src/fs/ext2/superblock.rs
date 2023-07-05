use crate::ceil_index;

use alloc::string::ToString;
use bitflags::bitflags;
use core::fmt::{self, Debug};

use crate::util::{
    self,
    str::{bytes_to_str, uuid_str},
    time::LocalTime,
};

use super::{block, disk_inode::Ext2Inode};

pub const EXT2_MAGIC: u16 = 0xef53;

/// Filesystem is free of errors
pub const FS_UNKNOWN: u16 = 0;
pub const FS_CLEAN: u16 = 1;
/// Filesystem has errors
pub const FS_ERR: u16 = 2;

/// Ignore errors
pub const ERR_IGNORE: u16 = 1;
/// Remount as read-only on error
pub const ERR_RONLY: u16 = 2;
/// Panic on error
pub const ERR_PANIC: u16 = 3;

/// Creator OS is Linux
pub const OS_LINUX: u32 = 0;
/// Creator OS is Hurd
pub const OS_HURD: u32 = 1;
/// Creator OS is Masix
pub const OS_MASIX: u32 = 2;
/// Creator OS is FreeBSD
pub const OS_FREEBSD: u32 = 3;
/// Creator OS is a BSD4.4-Lite derivative
pub const OS_LITE: u32 = 4;

#[repr(C)]
#[derive(Clone)]
pub struct Superblock {
    // taken from https://wiki.osdev.org/Ext2
    /// Total number of inodes in file system
    pub inodes_count: u32,
    /// Total number of blocks in file system
    pub blocks_count: u32,
    /// Number of blocks reserved for superuser (see offset 80)
    pub r_blocks_count: u32,
    /// Total number of unallocated blocks
    pub free_blocks_count: u32,
    /// Total number of unallocated inodes
    pub free_inodes_count: u32,
    /// Block number of the block containing the superblock
    pub first_data_block: u32,
    /// log2 (block size) - 10. (In other words, the number to shift 1,024
    /// to the left by to obtain the block size)
    pub log_block_size: u32,
    /// log2 (fragment size) - 10. (In other words, the number to shift
    /// 1,024 to the left by to obtain the fragment size)
    pub log_frag_size: i32,
    /// Number of blocks in each block group
    pub blocks_per_group: u32,
    /// Number of fragments in each block group
    pub frags_per_group: u32,
    /// Number of inodes in each block group
    pub inodes_per_group: u32,
    /// Last mount time (in POSIX time)
    pub mtime: u32,
    /// Last written time (in POSIX time)
    pub wtime: u32,
    /// Number of times the volume has been mounted since its last
    /// consistency check (fsck)
    pub mnt_count: u16,
    /// Number of mounts allowed before a consistency check (fsck) must be
    /// done
    pub max_mnt_count: i16,
    /// Ext2 signature (0xef53), used to help confirm the presence of Ext2
    /// on a volume
    pub magic: u16,
    /// File system state (see `FS_CLEAN` and `FS_ERR`)
    pub state: u16,
    /// What to do when an error is detected (see `ERR_IGNORE`, `ERR_RONLY` and
    /// `ERR_PANIC`)
    pub errors: u16,
    /// Minor portion of version (combine with Major portion below to
    /// construct full version field)
    pub rev_minor: u16,
    /// POSIX time of last consistency check (fsck)
    pub lastcheck: u32,
    /// Interval (in POSIX time) between forced consistency checks (fsck)
    pub checkinterval: u32,
    /// Operating system ID from which the filesystem on this volume was
    /// created
    pub creator_os: u32,
    /// Major portion of version (combine with Minor portion above to
    /// construct full version field)
    pub rev_major: u32,
    /// User ID that can use reserved blocks
    pub block_uid: u16,
    /// Group ID that can use reserved blocks
    pub block_gid: u16,

    /// First non-reserved inode in file system.
    pub first_inode: u32,
    /// SectorSize of each inode structure in bytes.
    pub inode_size: u16,
    /// Block group that this superblock is part of (if backup copy)
    pub block_group: u16,
    /// Optional features present (features that are not required to read
    /// or write, but usually result in a performance increase)
    pub features_opt: FeaturesOptional,
    /// Required features present (features that are required to be
    /// supported to read or write)
    pub features_req: FeaturesRequired,
    /// Features that if not supported, the volume must be mounted
    /// read-only
    pub features_ronly: FeaturesROnly,
    /// File system ID (what is output by blkid)
    pub fs_id: [u8; 16],
    /// Volume name (C-style string: characters terminated by a 0 byte)
    pub volume_name: [u8; 16],
    /// Path volume was last mounted to (C-style string: characters
    /// terminated by a 0 byte)
    pub last_mnt_path: [u8; 64],
    /// Compression algorithms used (see Required features above)
    pub compression: u32,
    /// Number of blocks to preallocate for files
    pub prealloc_blocks_files: u8,
    /// Number of blocks to preallocate for directories
    pub prealloc_blocks_dirs: u8,
    #[doc(hidden)]
    _unused: [u8; 2],
    /// Journal ID (same style as the File system ID above)
    pub journal_id: [u8; 16],
    /// Journal inode
    pub journal_inode: u32,
    /// Journal device
    pub journal_dev: u32,
    /// Head of orphan inode list
    pub journal_orphan_head: u32,
    #[doc(hidden)]
    _reserved: [u8; 788],
}

impl Debug for Superblock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Superblock")
            .field("inodes_count", &self.inodes_count)
            .field("blocks_count", &self.blocks_count)
            .field("r_blocks_count", &self.r_blocks_count)
            .field("free_blocks_count", &self.free_blocks_count)
            .field("free_inodes_count", &self.free_inodes_count)
            .field("first_data_block", &self.first_data_block)
            .field("block_size", &self.block_size())
            .field("frag_size", &self.frag_size())
            .field("blocks_per_group", &self.blocks_per_group)
            .field("frags_per_group", &self.frags_per_group)
            .field("inodes_per_group", &self.inodes_per_group)
            .field(
                "mtime",
                &LocalTime::from_posix(self.mtime as u64).to_string(),
            )
            .field(
                "wtime",
                &LocalTime::from_posix(self.wtime as u64).to_string(),
            )
            .field("mnt_count", &self.mnt_count)
            .field("max_mnt_count", &self.max_mnt_count)
            .field("magic", &format_args!("{:#X}", self.magic))
            .field("state", &self.state)
            .field("errors", &self.errors)
            .field("rev_minor", &self.rev_minor)
            .field(
                "lastcheck",
                &LocalTime::from_posix(self.lastcheck as u64).to_string(),
            )
            .field("checkinterval", &self.checkinterval)
            .field("creator_os", &self.creator_os)
            .field("rev_major", &self.rev_major)
            .field("block_uid", &self.block_uid)
            .field("block_gid", &self.block_gid)
            .field("first_inode", &self.first_inode)
            .field("inode_size", &self.inode_size)
            .field("block_group", &self.block_group)
            .field("features_opt", &self.features_opt)
            .field("features_req", &self.features_req)
            .field("features_ronly", &self.features_ronly)
            .field("fs_id", &uuid_str(&self.fs_id))
            .field("volume_name", &bytes_to_str(&self.volume_name))
            .field("last_mnt_path", &bytes_to_str(&self.last_mnt_path))
            .field("compression", &self.compression)
            .field("prealloc_blocks_files", &self.prealloc_blocks_files)
            .field("prealloc_blocks_dirs", &self.prealloc_blocks_dirs)
            .finish()
    }
}

impl Superblock {
    #[inline]
    pub fn block_size(&self) -> usize {
        1024 << self.log_block_size
    }

    #[inline]
    pub fn frag_size(&self) -> usize {
        1024 << self.log_frag_size
    }

    #[inline]
    pub fn inode_size(&self) -> usize {
        self.inode_size as usize
    }

    pub fn check_valid(&self) {
        assert_eq!(
            self.magic, EXT2_MAGIC,
            "magic number error, this maybe not ext2"
        );
        assert_eq!(self.state, FS_CLEAN);
        assert_eq!(self.block_size(), block::SIZE);
        assert_eq!(self.inode_size(), core::mem::size_of::<Ext2Inode>());
    }

    // 统计有多少 group
    pub fn blockgroup_count(&self) -> u32 {
        let by_blocks = ceil_index!(self.blocks_count, self.blocks_per_group);
        let by_inodes = ceil_index!(self.inodes_count, self.inodes_per_group);
        assert_eq!(by_blocks, by_inodes);
        by_blocks
    }
}

bitflags! {
    /// Optional features
    #[derive(Debug, Clone)]
    pub struct FeaturesOptional: u32 {
        /// Preallocate some number of (contiguous?) blocks (see
        /// `Superblock::prealloc_blocks_dirs`) to a directory when creating a new one
        const PREALLOCATE = 0x0001;
        /// AFS server inodes exist
        const AFS = 0x0002;
        /// File system has a journal (Ext3)
        const JOURNAL = 0x0004;
        /// Inodes have extended attributes
        const EXTENDED_INODE = 0x0008;
        /// File system can resize itself for larger partitions
        const SELF_RESIZE = 0x0010;
        /// Directories use hash index
        const HASH_INDEX = 0x0020;
    }
}

bitflags! {
    /// Required features. If these are not supported; can't mount
    #[derive(Debug, Clone)]
    pub struct FeaturesRequired: u32 {
        /// Compression is used
        const REQ_COMPRESSION = 0x0001;
        /// Directory entries contain a type field
        const REQ_DIRECTORY_TYPE = 0x0002;
        /// File system needs to replay its journal
        const REQ_REPLAY_JOURNAL = 0x0004;
        /// File system uses a journal device
        const REQ_JOURNAL_DEVICE = 0x0008;
    }
}

bitflags! {
    /// ROnly features. If these are not supported; remount as read-only
    #[derive(Debug, Clone)]
    pub struct FeaturesROnly: u32 {
        /// Sparse superblocks and group descriptor tables
        const RONLY_SPARSE = 0x0001;
        /// File system uses a 64-bit file size
        const RONLY_FILE_SIZE_64 = 0x0002;
        /// Directory contents are stored in the form of a Binary Tree
        const RONLY_BTREE_DIRECTORY = 0x0004;
    }
}
