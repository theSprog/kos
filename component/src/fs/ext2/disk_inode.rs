use alloc::vec::{IntoIter, Vec};
use bitflags::bitflags;

use super::block;
use super::block::DataBlock;
use super::block_device;
use super::vfs::meta::*;
use crate::ceil_index;

#[repr(C)]
#[derive(Clone)]
pub struct Ext2Inode {
    /// Type and Permissions (see below)
    pub type_perm: TypePerm,
    /// User ID
    pub uid: u16,
    /// Lower 32 bits of size in bytes
    pub size_low: u32,
    /// Last Access Time (in POSIX time)
    pub atime: u32,
    /// Creation Time (in POSIX time)
    pub ctime: u32,
    /// Last Modification time (in POSIX time)
    pub mtime: u32,
    /// Deletion time (in POSIX time)
    pub dtime: u32,
    /// Group ID
    pub gid: u16,
    /// Count of hard links (directory entries) to this inode. When this
    /// reaches 0, the data blocks are marked as unallocated.
    pub hard_links: u16,
    /// Count of disk sectors (not Ext2 blocks) in use by this inode, not
    /// counting the actual inode structure nor directory entries linking
    /// to the inode.
    pub sectors_count: u32,
    /// Flags
    pub flags: Flags,
    /// Operating System Specific value #1
    pub _os_specific_1: [u8; 4],
    /// Direct block pointers
    pub direct_pointer: [u32; 12],
    /// Singly Indirect Block Pointer (Points to a block that is a list of
    /// block pointers to data)
    pub indirect_pointer: u32,
    /// Doubly Indirect Block Pointer (Points to a block that is a list of
    /// block pointers to Singly Indirect Blocks)
    pub doubly_indirect: u32,
    /// Triply Indirect Block Pointer (Points to a block that is a list of
    /// block pointers to Doubly Indirect Blocks)
    pub triply_indirect: u32,
    /// Generation number (Primarily used for NFS)
    pub gen_number: u32,
    /// In Ext2 version 0, this field is reserved. In version >= 1,
    /// Extended attribute block (File ACL).
    pub ext_attribute_block: u32,
    /// In Ext2 version 0, this field is reserved. In version >= 1, Upper
    /// 32 bits of file size (if feature bit set) if it's a file,
    /// Directory ACL if it's a directory
    pub size_high: u32,
    /// Block address of fragment
    pub frag_block_addr: u32,
    /// Operating System Specific Value #2
    pub _os_specific_2: [u8; 12],
}

type IndirectBlock = [u32; Ext2Inode::INDIRECT_COUNT];

impl Ext2Inode {
    pub const DIRECT_COUNT: usize = 12;
    pub const INDIRECT_COUNT: usize = block::SIZE / 4;
    pub const INDIRECT_BOUND: usize = Self::DIRECT_COUNT + Self::INDIRECT_COUNT;
    pub const DOUBLE_COUNT: usize = Self::INDIRECT_COUNT * Self::INDIRECT_COUNT;
    pub const DOUBLE_BOUND: usize = Self::INDIRECT_BOUND + Self::DOUBLE_COUNT;

    pub fn init(&mut self, filetype: VfsFileType) {
        self.set_filetype(&filetype);
        if filetype.is_symlink() {
            self.set_permissions(&VfsPermissions::all());
        } else {
            self.set_permissions(&VfsPermissions::empty());
        }

        self.uid = 1000;
        self.size_low = 0;
        self.atime = 0;
        self.ctime = 0;
        self.mtime = 0;
        self.dtime = 0;
        self.gid = 100;
        self.hard_links = 1;
        self.sectors_count = 0;
        self.flags = Flags::empty();
        self._os_specific_1 = [0; 4];
        self.direct_pointer = [0; Self::DIRECT_COUNT];
        self.indirect_pointer = 0;
        self.doubly_indirect = 0;
        self.triply_indirect = 0;
        self.gen_number = 0;
        self.ext_attribute_block = 0;
        self.size_high = 0;
        self.frag_block_addr = 0;
        self._os_specific_2 = [0; 12];
    }

    pub fn filetype(&self) -> VfsFileType {
        self.type_perm.filetype()
    }

    pub fn set_filetype(&mut self, filetype: &VfsFileType) {
        self.type_perm.set_filetype(filetype)
    }

    pub fn permissions(&self) -> VfsPermissions {
        self.type_perm.permissions()
    }

    pub fn set_permissions(&mut self, permissions: &VfsPermissions) {
        self.type_perm.set_permissions(permissions);
    }

    pub fn size(&self) -> usize {
        if self.filetype().is_file() {
            assert_eq!(self.size_high, 0);
        }
        self.size_low as usize
    }

    pub fn set_size(&mut self, size: usize) {
        if self.filetype().is_file() {
            assert_eq!(self.size_high, 0);
        }
        self.size_low = size as u32;
    }

    pub fn timestamp(&self) -> VfsTimeStamp {
        VfsTimeStamp::new(
            self.atime as u64,
            self.ctime as u64,
            self.mtime as u64,
            self.dtime as u64,
        )
    }

    pub fn uid(&self) -> u16 {
        self.uid
    }
    pub fn gid(&self) -> u16 {
        self.gid
    }

    pub fn hard_links(&self) -> u16 {
        self.hard_links
    }

    pub fn inc_hard_links(&mut self) {
        self.hard_links += 1;
    }

    pub fn dec_hard_links(&mut self) -> bool {
        self.hard_links -= 1;
        self.hard_links == 0
    }

    fn block_id_for(&self, inner_idx: u32) -> u32 {
        let inner_idx = inner_idx as usize;
        if inner_idx < Self::DIRECT_COUNT {
            self.direct_pointer[inner_idx]
        } else if inner_idx < Self::INDIRECT_BOUND {
            block_device::read(
                self.indirect_pointer as usize,
                0,
                |indirect_block: &IndirectBlock| indirect_block[inner_idx - Self::DIRECT_COUNT],
            )
        } else if inner_idx < Self::DOUBLE_BOUND {
            let last = inner_idx - Self::INDIRECT_BOUND;
            let indirect = block_device::read(
                self.doubly_indirect as usize,
                0,
                |indirect2: &IndirectBlock| indirect2[last / Self::INDIRECT_COUNT],
            );

            block_device::read(indirect as usize, 0, |indirect1: &IndirectBlock| {
                indirect1[last % Self::INDIRECT_COUNT]
            })
        } else {
            panic!("where is the large block from : inner_id = {}", inner_idx);
        }
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let block_size = block::SIZE;
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size());
        if start >= end {
            return 0;
        }
        let mut start_block = start / block_size;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / block_size + 1) * block_size;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];

            block_device::read(
                self.block_id_for(start_block as u32) as usize,
                0,
                |data_block: &DataBlock| {
                    let src = &data_block[start % block_size..start % block_size + block_read_size];
                    dst.copy_from_slice(src);
                },
            );

            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }

    /// 文件长度必须先扩容, 本函数不负责扩容
    pub fn write_at(&mut self, offset: usize, buf: &[u8]) -> usize {
        let block_size = block::SIZE;
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size());
        assert!(start <= end);
        let mut start_block = start / block_size;
        let mut write_size = 0usize;
        loop {
            let mut end_current_block = (start / block_size + 1) * block_size;
            end_current_block = end_current_block.min(end);

            // write and update write size
            let block_write_size = end_current_block - start;
            block_device::modify(
                self.block_id_for(start_block as u32) as usize,
                0,
                |data_block: &mut DataBlock| {
                    let src = &buf[write_size..write_size + block_write_size];
                    let dst =
                        &mut data_block[start % block_size..start % block_size + block_write_size];
                    dst.copy_from_slice(src);
                },
            );
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        write_size
    }

    pub fn data_blocks(size: usize) -> usize {
        ceil_index!(size, block::SIZE)
    }

    // 计算文件包含的总块数, 包含 indirect1/2
    pub fn total_blocks(size: usize) -> usize {
        let data_blocks = Self::data_blocks(size);
        let mut total = data_blocks;

        // 需要一个块充当 indirect1
        if data_blocks > Self::DIRECT_COUNT {
            total += 1;
        }

        // 需要一个块充当 indirect2
        if data_blocks > Self::INDIRECT_BOUND {
            total += 1;
            let double_blocks = data_blocks - Self::INDIRECT_BOUND;
            total += ceil_index!(double_blocks, Self::INDIRECT_COUNT);
        }
        total
    }

    // 在 [start, end) 之间填充 blocks
    fn fill_from_direct(
        &mut self,
        start_block: usize,
        end_block: usize,
        blocks: &mut IntoIter<u32>,
    ) -> usize {
        let mut current = start_block;
        let end = end_block.min(Self::DIRECT_COUNT);
        while current < end {
            self.direct_pointer[current] = blocks.next().unwrap();
            current += 1;
        }
        current
    }

    fn fill_from_indirect(
        &mut self,
        start_block: usize,
        end_block: usize,
        blocks: &mut IntoIter<u32>,
    ) -> usize {
        // 如果不在自己的范围内
        if end_block <= Self::DIRECT_COUNT {
            return start_block;
        }

        let end = (end_block - Self::DIRECT_COUNT).min(Self::INDIRECT_COUNT);
        let mut current = start_block - Self::DIRECT_COUNT;
        if current == 0 {
            self.indirect_pointer = blocks.next().unwrap();
        }
        block_device::modify(
            self.indirect_pointer as usize,
            0,
            |indirect1: &mut IndirectBlock| {
                while current < end {
                    indirect1[current] = blocks.next().unwrap();
                    current += 1;
                }
            },
        );

        current + Self::DIRECT_COUNT
    }

    fn fill_from_double(
        &mut self,
        start_block: usize,
        end_block: usize,
        blocks: &mut IntoIter<u32>,
    ) -> usize {
        if end_block <= Self::INDIRECT_BOUND {
            return start_block;
        }

        let end = (end_block - Self::INDIRECT_BOUND).min(Self::DOUBLE_COUNT);
        let mut current = start_block - Self::INDIRECT_BOUND;
        if current == 0 {
            self.doubly_indirect = blocks.next().unwrap();
        }

        // fill indirect2 from (a0, b0) -> (a1, b1)
        let mut a0 = current / Self::INDIRECT_COUNT;
        let mut b0 = current % Self::INDIRECT_COUNT;
        let a1 = end / Self::INDIRECT_COUNT;
        let b1 = end % Self::INDIRECT_COUNT;

        block_device::modify(
            self.doubly_indirect as usize,
            0,
            |indirect2: &mut IndirectBlock| {
                while (a0 < a1) || (a0 == a1 && b0 < b1) {
                    if b0 == 0 {
                        indirect2[a0] = blocks.next().unwrap();
                    }
                    block_device::modify(
                        indirect2[a0] as usize,
                        0,
                        |indirect1: &mut IndirectBlock| {
                            while (a0 < a1 && b0 < Self::INDIRECT_COUNT) || (a0 == a1 && b0 < b1) {
                                indirect1[b0] = blocks.next().unwrap();
                                b0 += 1;
                                current += 1;
                            }

                            if b0 == Self::INDIRECT_COUNT {
                                b0 = 0;
                                a0 += 1;
                            }
                        },
                    )
                }
            },
        );

        current + Self::INDIRECT_BOUND
    }

    pub fn increase_to(&mut self, new_size: usize, new_blocks: Vec<u32>) {
        assert!(new_size > self.size());
        let mut start_block = Self::data_blocks(self.size());
        self.set_size(new_size);
        let end_block = Self::data_blocks(new_size);

        let mut blocks_iter = new_blocks.into_iter();

        if start_block < Self::DIRECT_COUNT {
            start_block = self.fill_from_direct(start_block, end_block, &mut blocks_iter);
            start_block = self.fill_from_indirect(start_block, end_block, &mut blocks_iter);
            start_block = self.fill_from_double(start_block, end_block, &mut blocks_iter);
        } else if start_block < Self::INDIRECT_BOUND {
            start_block = self.fill_from_indirect(start_block, end_block, &mut blocks_iter);
            start_block = self.fill_from_double(start_block, end_block, &mut blocks_iter);
        } else if start_block < Self::DOUBLE_BOUND {
            start_block = self.fill_from_double(start_block, end_block, &mut blocks_iter);
        } else {
            panic!("where the ultra-big size(={}) from?", new_size);
        }

        assert_eq!(start_block, end_block);
        assert!(blocks_iter.next().is_none());
    }

    fn free_from_direct(
        &mut self,
        start_block: usize,
        end_block: usize,
        blocks: &mut Vec<u32>,
    ) -> usize {
        let mut current = start_block;
        let end = end_block.min(Self::DIRECT_COUNT);
        while current < end {
            blocks.push(self.direct_pointer[current]);
            self.direct_pointer[current] = 0;
            current += 1;
        }
        current
    }

    fn free_from_indirect(
        &mut self,
        start_block: usize,
        end_block: usize,
        blocks: &mut Vec<u32>,
    ) -> usize {
        // 如果不在自己的范围内
        if end_block <= Self::DIRECT_COUNT {
            return start_block;
        }

        let end = (end_block - Self::DIRECT_COUNT).min(Self::INDIRECT_COUNT);
        let mut current = start_block - Self::DIRECT_COUNT;
        let free_indirect = current == 0;

        block_device::modify(
            self.indirect_pointer as usize,
            0,
            |indirect1: &mut IndirectBlock| {
                while current < end {
                    blocks.push(indirect1[current]);
                    current += 1;
                }
            },
        );

        if free_indirect {
            blocks.push(self.indirect_pointer);
            self.indirect_pointer = 0;
        }

        current + Self::DIRECT_COUNT
    }

    fn free_from_double(
        &mut self,
        start_block: usize,
        end_block: usize,
        blocks: &mut Vec<u32>,
    ) -> usize {
        if end_block <= Self::INDIRECT_BOUND {
            return start_block;
        }

        let end = (end_block - Self::INDIRECT_BOUND).min(Self::DOUBLE_COUNT);
        let mut current = start_block - Self::INDIRECT_BOUND;
        let free_double = current == 0;

        // free indirect2 from (a0, b0) -> (a1, b1)
        let mut a0 = current / Self::INDIRECT_COUNT;
        let mut b0 = current % Self::INDIRECT_COUNT;
        let a1 = end / Self::INDIRECT_COUNT;
        let b1 = end % Self::INDIRECT_COUNT;
        block_device::modify(
            self.doubly_indirect as usize,
            0,
            |indirect2: &mut IndirectBlock| {
                while (a0 < a1) || (a0 == a1 && b0 < b1) {
                    if b0 == 0 {
                        blocks.push(indirect2[a0]);
                    }
                    block_device::modify(
                        indirect2[a0] as usize,
                        0,
                        |indirect1: &mut IndirectBlock| {
                            while (a0 < a1 && b0 < Self::INDIRECT_COUNT) || (a0 == a1 && b0 < b1) {
                                blocks.push(indirect1[b0]);
                                b0 += 1;
                                current += 1;
                            }

                            if b0 == Self::INDIRECT_COUNT {
                                b0 = 0;
                                a0 += 1;
                            }
                        },
                    )
                }
            },
        );

        if free_double {
            blocks.push(self.doubly_indirect);
            self.doubly_indirect = 0;
        }

        current + Self::INDIRECT_BOUND
    }

    pub fn decrease_to(&mut self, new_size: usize) -> Vec<u32> {
        assert!(new_size < self.size());
        let end_block = Self::data_blocks(self.size());
        self.set_size(new_size);
        let mut start_block = Self::data_blocks(new_size);

        let mut freed = Vec::new();
        if start_block < Self::DIRECT_COUNT {
            start_block = self.free_from_direct(start_block, end_block, &mut freed);
            start_block = self.free_from_indirect(start_block, end_block, &mut freed);
            start_block = self.free_from_double(start_block, end_block, &mut freed);
        } else if start_block < Self::INDIRECT_BOUND {
            start_block = self.free_from_indirect(start_block, end_block, &mut freed);
            start_block = self.free_from_double(start_block, end_block, &mut freed);
        } else if start_block < Self::DOUBLE_BOUND {
            start_block = self.free_from_double(start_block, end_block, &mut freed);
        } else {
            panic!("where the ultra-big size(={}) from?", new_size);
        }

        assert_eq!(start_block, end_block);
        freed
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct TypePerm: u16 {
        /// FIFO
        const FIFO = 0x1000;
        /// Character device
        const CHAR_DEVICE = 0x2000;
        /// Directory
        const DIRECTORY = 0x4000;
        /// Block device
        const BLOCK_DEVICE = 0x6000;
        /// Regular file
        const FILE = 0x8000;
        /// Symbolic link
        const SYMLINK = 0xA000;
        /// Unix socket
        const SOCKET = 0xC000;
        /// Other—execute permission
        const O_EXEC = 0x001;
        /// Other—write permission
        const O_WRITE = 0x002;
        /// Other—read permission
        const O_READ = 0x004;
        /// Group—execute permission
        const G_EXEC = 0x008;
        /// Group—write permission
        const G_WRITE = 0x010;
        /// Group—read permission
        const G_READ = 0x020;
        /// User—execute permission
        const U_EXEC = 0x040;
        /// User—write permission
        const U_WRITE = 0x080;
        /// User—read permission
        const U_READ = 0x100;
        /// Sticky Bit
        const STICKY = 0x200;
        /// Set group ID
        const SET_GID = 0x400;
        /// Set user ID
        const SET_UID = 0x800;
    }
}

impl TypePerm {
    pub fn filetype(&self) -> VfsFileType {
        match self {
            // 下面的 if 不可以轻易调整顺序, 否则可能发生掩盖问题
            _ if self.contains(Self::SOCKET) => VfsFileType::Socket,
            _ if self.contains(Self::SYMLINK) => VfsFileType::SymbolicLink,
            _ if self.contains(Self::FILE) => VfsFileType::RegularFile,
            _ if self.contains(Self::BLOCK_DEVICE) => VfsFileType::BlockDev,
            _ if self.contains(Self::DIRECTORY) => VfsFileType::Directory,
            _ if self.contains(Self::CHAR_DEVICE) => VfsFileType::CharDev,
            _ if self.contains(Self::FIFO) => VfsFileType::FIFO,
            _ => unreachable!("bits {:X}", self.bits()),
        }
    }

    pub fn set_filetype(&mut self, filetype: &VfsFileType) {
        match filetype {
            VfsFileType::RegularFile => self.insert(Self::FILE),
            VfsFileType::Directory => self.insert(Self::DIRECTORY),
            VfsFileType::CharDev => self.insert(Self::CHAR_DEVICE),
            VfsFileType::BlockDev => self.insert(Self::BLOCK_DEVICE),
            VfsFileType::FIFO => self.insert(Self::FIFO),
            VfsFileType::Socket => self.insert(Self::SOCKET),
            VfsFileType::SymbolicLink => self.insert(Self::SYMLINK),
        };
    }

    pub fn permissions(&self) -> VfsPermissions {
        let mut user = 0u8;
        let mut group = 0u8;
        let mut others = 0u8;
        if self.contains(Self::U_READ) {
            user |= 0b100;
        }
        if self.contains(Self::U_WRITE) {
            user |= 0b010;
        }
        if self.contains(Self::U_EXEC) {
            user |= 0b001;
        }
        if self.contains(Self::G_READ) {
            group |= 0b100;
        }
        if self.contains(Self::G_WRITE) {
            group |= 0b010;
        }
        if self.contains(Self::G_EXEC) {
            group |= 0b001;
        }
        if self.contains(Self::O_READ) {
            others |= 0b100;
        }
        if self.contains(Self::O_WRITE) {
            others |= 0b010;
        }
        if self.contains(Self::O_EXEC) {
            others |= 0b001;
        }
        VfsPermissions::inner_new(user, group, others)
    }

    fn set_permissions(&mut self, permissions: &VfsPermissions) {
        let user = permissions.user();
        let group = permissions.group();
        let others = permissions.others();
        self.set_user(user);
        self.set_group(group);
        self.set_others(others);
    }

    fn set_user(&mut self, user: VfsPermission) {
        self.set(Self::U_READ, user.read());
        self.set(Self::U_WRITE, user.write());
        self.set(Self::U_EXEC, user.execute());
    }

    fn set_group(&mut self, group: VfsPermission) {
        self.set(Self::G_READ, group.read());
        self.set(Self::G_WRITE, group.write());
        self.set(Self::G_EXEC, group.execute());
    }

    fn set_others(&mut self, others: VfsPermission) {
        self.set(Self::O_READ, others.read());
        self.set(Self::O_WRITE, others.write());
        self.set(Self::O_EXEC, others.execute());
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct Flags: u32 {
        /// Secure deletion (not used)
        const SECURE_DEL = 0x00000001;
        /// Keep a copy of data when deleted (not used)
        const KEEP_COPY = 0x00000002;
        /// File compression (not used)
        const COMPRESSION = 0x00000004;
        /// Synchronous updates—new data is written immediately to disk
        const SYNC_UPDATE = 0x00000008;
        /// Immutable file (content cannot be changed)
        const IMMUTABLE = 0x00000010;
        /// Append only
        const APPEND_ONLY = 0x00000020;
        /// File is not included in 'dump' command
        const NODUMP = 0x00000040;
        /// Last accessed time should not updated
        const DONT_ATIME = 0x00000080;
        /// Hash indexed directory
        const HASH_DIR = 0x00010000;
        /// AFS directory
        const AFS_DIR = 0x00020000;
        /// Journal file data
        const JOURNAL_DATA = 0x00040000;
    }
}
