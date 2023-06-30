use core::fmt::Debug;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use spin::Mutex;

use super::vfs::{
    error::{IOError, IOErrorKind, VfsError, VfsErrorKind, VfsResult},
    meta::VfsFileType,
    VfsDirEntry, VfsInode, VfsPath,
};

use super::{
    allocator::Ext2Allocator, block, disk_inode::Ext2Inode, inode::Inode, layout::Ext2Layout,
};

use crate::{cast, cast_mut, ceil};

#[repr(C)]
#[derive(Clone)]
pub struct Ext2DirEntry {
    inode_id: u32,
    record_len: u16,
    name_len: u8,
    filetype: u8,
    name: u8,
}

impl Ext2DirEntry {
    pub const EXT2_FT_UNKNOWN: u8 = 0;
    pub const EXT2_FT_REG_FILE: u8 = 1;
    pub const EXT2_FT_DIR: u8 = 2;
    pub const EXT2_FT_CHRDEV: u8 = 3;
    pub const EXT2_FT_BLKDEV: u8 = 4;
    pub const EXT2_FT_FIFO: u8 = 5;
    pub const EXT2_FT_SOCK: u8 = 6;
    pub const EXT2_FT_SYMLINK: u8 = 7;

    pub const MAX_FILE_NAME: usize = u8::MAX as usize;
    // 去掉末尾的 name 留下的长度, 有了它就可用从结构体头偏移到 name 起始处
    const BARE_LEN: usize = 8;

    pub fn build_raw<'a>(
        buffer: &'a mut [u8],
        entry_name: &str,
        inode_id: usize,
        filetype: VfsFileType,
    ) -> &'a mut Self {
        let entry = cast_mut!(buffer.as_ptr(), Self);

        entry.inode_id = inode_id as u32;
        entry.name_len = entry_name.len() as u8;
        entry.record_len = ceil!(Self::BARE_LEN + entry.name_len as usize, 4) as u16;
        entry.filetype = match filetype {
            VfsFileType::RegularFile => Self::EXT2_FT_REG_FILE,
            VfsFileType::Directory => Self::EXT2_FT_DIR,
            VfsFileType::CharDev => Self::EXT2_FT_CHRDEV,
            VfsFileType::BlockDev => Self::EXT2_FT_BLKDEV,
            VfsFileType::FIFO => Self::EXT2_FT_FIFO,
            VfsFileType::Socket => Self::EXT2_FT_SOCK,
            VfsFileType::SymbolicLink => Self::EXT2_FT_SYMLINK,
        };

        let name_slice = &mut buffer[Self::BARE_LEN..Self::BARE_LEN + entry_name.len()];
        name_slice.copy_from_slice(entry_name.as_bytes());

        entry
    }

    pub fn is_unused(&self) -> bool {
        self.inode_id == 0
    }

    // record 理论所占空间
    pub fn regular_len(&self) -> usize {
        // 4 字节对齐
        ceil!(Self::BARE_LEN + self.name_len as usize, 4)
    }

    // record 实际所占空间
    pub fn record_len(&self) -> usize {
        assert_eq!(0, self.record_len % 4);
        self.record_len as usize
    }

    pub fn has_free(&self, needed: usize) -> bool {
        // record_len 至少和 regular_len 一样大
        (self.record_len() - self.regular_len()) >= needed
    }

    // 缩小该 record 所占空间, 返回 (期望空间, 释放空间)
    pub fn rec_narrow(&mut self) -> (usize, usize) {
        let old_len = self.record_len();
        self.record_len = self.regular_len() as u16;
        (self.record_len(), old_len - self.record_len())
    }

    pub fn rec_expand(&mut self, new_len: usize) -> usize {
        let old_len = self.record_len();
        assert!(old_len <= new_len);
        self.record_len = new_len as u16;
        old_len
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, self.regular_len() as usize)
        }
    }

    pub fn name_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const _ as *const u8).add(Self::BARE_LEN),
                self.name_len as usize,
            )
        }
    }

    pub fn name_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self as *mut _ as *mut u8).add(Self::BARE_LEN),
                self.name_len as usize,
            )
        }
    }
}

pub struct DirEntry {
    name: String,
    inode_id: usize,
    parent_id: usize,
    layout: Arc<Ext2Layout>,
    allocator: Arc<Mutex<Ext2Allocator>>,
}
impl DirEntry {
    fn new(
        inode_id: usize,
        parent_id: usize,
        name: String,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Self {
        Self {
            name,
            inode_id,
            parent_id,
            layout,
            allocator,
        }
    }

    pub(crate) fn inode(&self) -> Inode {
        self.layout
            .inode_nth(self.inode_id, self.layout.clone(), self.allocator.clone())
            .with_parent(self.parent_id)
    }
}

impl Debug for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{} -> {}", self.name, self.inode_id)
    }
}

impl VfsDirEntry for DirEntry {
    fn inode_id(&self) -> usize {
        self.inode_id
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn inode(&self) -> Box<dyn VfsInode> {
        Box::new(self.inode())
    }
}

pub struct Dir {
    inode_id: usize,
    buffer: Vec<u8>,
    layout: Arc<Ext2Layout>,
    allocator: Arc<Mutex<Ext2Allocator>>,
}

impl Dir {
    pub fn from_inode(
        inode_id: usize,
        ext2_inode: &Ext2Inode,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Self {
        let mut buffer = alloc::vec![0; ext2_inode.size()];
        ext2_inode.read_at(0, &mut buffer);
        Self {
            inode_id,
            buffer,
            layout,
            allocator,
        }
    }

    fn inode_id(&self) -> usize {
        self.inode_id
    }

    pub fn write_to_disk(&self, ext2_inode: &mut Ext2Inode) -> VfsResult<()> {
        if ext2_inode.size() < self.buffer.len() {
            let new_blocks = self.allocator.lock().alloc_data(1)?;
            // 不需要填充 0 因为 buffer 总是和 ext2_inode 所承载空间一样大,
            // 而且 buffer 末尾为 [..., xx, 0, 0, ...] 切片
            ext2_inode.increase_to(self.buffer.len(), new_blocks)
        }
        ext2_inode.write_at(0, &self.buffer);
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.iter().all(|&x| x == 0)
    }

    pub(crate) fn entries(&self) -> Vec<DirEntry> {
        let mut entries = Vec::new();
        for (_, entry) in self.split() {
            let entry_id = entry.inode_id as usize;
            let name = String::from_utf8(entry.name_bytes().to_vec()).unwrap();
            entries.push(DirEntry::new(
                entry_id,
                self.inode_id(),
                name,
                self.layout.clone(),
                self.allocator.clone(),
            ));
        }
        entries
    }

    fn split(&self) -> Vec<(usize, &Ext2DirEntry)> {
        self.split_mut()
            .into_iter()
            .map(|(index, entry)| (index, entry as &Ext2DirEntry))
            .collect()
    }

    fn split_mut(&self) -> Vec<(usize, &mut Ext2DirEntry)> {
        let mut offset = 0;
        let mut slice = Vec::new();
        while offset < self.buffer.len() {
            let entry = cast_mut!(self.buffer.as_ptr().add(offset), Ext2DirEntry);
            let rec_len = entry.record_len as usize;
            slice.push((offset, entry));
            offset += rec_len;
        }
        slice
    }

    fn place_entry(&mut self, offset: usize, entry: &Ext2DirEntry) {
        let dst = &mut self.buffer[offset..offset + entry.regular_len()];
        let src = entry.as_bytes();
        dst.copy_from_slice(src);
    }

    fn insert_entry(&mut self, entry_name: &str, inode_id: usize, filetype: VfsFileType) {
        let mut buffer = [0u8; block::SIZE];
        let new_entry = Ext2DirEntry::build_raw(&mut buffer, entry_name, inode_id, filetype);

        if self.is_empty() {
            new_entry.rec_expand(block::SIZE);
            self.place_entry(0, new_entry);
            return;
        }

        for (offset, entry) in self.split_mut() {
            if entry.has_free(new_entry.regular_len()) {
                let (new_len, freed) = entry.rec_narrow();
                new_entry.rec_expand(freed);
                self.place_entry(offset + new_len, new_entry);
                return;
            }
        }

        // 到此处说明 dir 没有空间可用, 需要扩容
        let old_len = self.buffer.len();
        self.buffer.extend(alloc::vec![0u8; block::SIZE]);
        new_entry.rec_expand(block::SIZE);
        self.place_entry(old_len, new_entry);
    }

    fn remove_entry(&mut self, entry_name: &str) {
        let mut offset = 0;
        while offset < self.buffer.len() {
            let prev_entry = cast_mut!(self.buffer.as_ptr().add(offset), Ext2DirEntry);
            offset += prev_entry.record_len();
            let cur_entry = cast!(self.buffer.as_ptr().add(offset), Ext2DirEntry);

            if cur_entry.name_bytes() == entry_name.as_bytes() {
                if offset % block::SIZE == 0 {
                    self.move_to_prev(offset, offset + cur_entry.record_len());
                } else {
                    let new_len = prev_entry.record_len() + cur_entry.record_len();
                    prev_entry.rec_expand(new_len);
                }
                break;
            }
        }
    }

    /// | prev | current         | other | => | current             | other |
    fn move_to_prev(&mut self, prev_offset: usize, cur_offset: usize) {
        assert_eq!(0, prev_offset % block::SIZE);
        let prev_entry = cast!(self.buffer.as_ptr().add(prev_offset), Ext2DirEntry);
        let cur_entry = cast_mut!(self.buffer.as_ptr().add(cur_offset), Ext2DirEntry);
        if cur_entry.is_unused() {
            return;
        }

        cur_entry.rec_expand(prev_entry.record_len() + cur_entry.record_len());
        self.place_entry(prev_offset, cur_entry);
    }
}

impl Inode {
    // 读当前 inode 下所有目录下, 如果当前 inode 不是目录抛出异常
    pub fn read_dir(&self) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        if !self.is_dir() {
            return Err(IOError::new(IOErrorKind::NotADirectory).into());
        }

        Ok(self
            .inner_read_dir()
            .into_iter()
            .map(|x| Box::new(x) as Box<dyn VfsDirEntry>)
            .collect())
    }

    fn inner_read_dir(&self) -> Vec<DirEntry> {
        assert!(self.is_dir());

        self.read_disk_inode(|ext2_inode| {
            let dir = Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            dir.entries()
        })
    }

    // 从 path 一直走到终点, 遇到 symlink 也解析并继续走
    pub(crate) fn walk(&self, path: &VfsPath) -> VfsResult<Inode> {
        let last = self.goto_last(path)?;
        if last.is_symlink() {
            let parent_last = last.parent_inode();
            parent_last.walk(&last.symlink_target(path)?)
        } else {
            Ok(last)
        }
    }

    fn goto_last(&self, path: &VfsPath) -> VfsResult<Inode> {
        let mut current_inode = self.clone();
        let mut next_path = VfsPath::empty(path.is_from_root());
        for next in path.iter() {
            next_path.forward(next);

            if current_inode.is_symlink() {
                let parent = current_inode.parent_inode();
                let symlink_path = current_inode.symlink_target(path)?;
                if symlink_path.is_from_root() {
                    let root = self.layout().root_inode(self.layout(), self.allocator());
                    current_inode = root.walk(&symlink_path)?;
                } else {
                    current_inode = parent.walk(&symlink_path)?;
                }
            }

            if !current_inode.is_dir() {
                return Err(IOError::new(IOErrorKind::NotADirectory)
                    .with_path(&next_path)
                    .into());
            }

            current_inode = current_inode
                .select_child(next)
                .map_err(|err| err.with_path(&next_path))?;
        }
        Ok(current_inode)
    }

    fn child_inode(&self, entries: &[DirEntry], entry_name: &str) -> VfsResult<Inode> {
        let chosen = Self::find_single(entries, entry_name);
        if chosen.is_none() {
            return Err(IOError::new(IOErrorKind::NotFound).into());
        }
        let child_id = chosen.unwrap().inode_id();
        Ok(self
            .layout()
            .inode_nth(child_id, self.layout(), self.allocator())
            .with_parent(self.inode_id()))
    }

    pub(crate) fn select_child(&self, entry_name: &str) -> VfsResult<Inode> {
        assert!(self.is_dir());
        let entries = self.inner_read_dir();
        self.child_inode(&entries, entry_name)
    }

    fn find_single<'a>(entries: &'a [DirEntry], entry_name: &str) -> Option<&'a DirEntry> {
        let mut found_entry = None;

        for entry in entries {
            if entry.name() == entry_name {
                if found_entry.is_some() {
                    panic!(
                        "Multiple entries found with filename: {}, entries: {:#?}",
                        entry_name, entries
                    );
                }
                found_entry = Some(entry);
            }
        }

        found_entry
    }

    fn check_valid_insert(&self, path: &VfsPath) -> VfsResult<()> {
        if !self.is_dir() {
            return Err(IOError::new(IOErrorKind::NotADirectory)
                .with_path(path)
                .into());
        }

        let filename = path.last();
        if filename.is_none() {
            return Err(VfsErrorKind::InvalidPath(path.to_string()).into());
        }

        let filename = filename.unwrap();
        let entries = self.inner_read_dir();
        let chosen = Self::find_single(&entries, filename);
        if chosen.is_some() {
            return Err(IOError::new(IOErrorKind::AlreadyExists)
                .with_path(path)
                .into());
        }

        if filename.len() > Ext2DirEntry::MAX_FILE_NAME {
            return Err(IOError::new(IOErrorKind::TooLongFileName)
                .with_path(path)
                .into());
        }

        Ok(())
    }

    fn check_valid_remove(&self, path: &VfsPath) -> VfsResult<()> {
        if !self.is_dir() {
            return Err(IOError::new(IOErrorKind::NotADirectory)
                .with_path(path)
                .into());
        }
        let filename = path.last();
        if filename.is_none() {
            return Err(VfsErrorKind::InvalidPath(path.to_string()).into());
        }

        let filename = filename.unwrap();
        let entries = self.inner_read_dir();
        let chosen = Self::find_single(&entries, filename);

        // 如果没有该 entry
        if chosen.is_none() {
            return Err(IOError::new(IOErrorKind::NotFound).with_path(path).into());
        }

        Ok(())
    }

    // 该函数不会设置权限
    pub fn insert_entry(
        &mut self,
        path: &VfsPath,
        filetype: VfsFileType,
    ) -> VfsResult<Box<dyn VfsInode>> {
        self.check_valid_insert(path)?;
        let entry_name = path.last().unwrap();
        if entry_name.len() > u8::MAX as usize {
            return Err(IOError::new(IOErrorKind::TooLongFileName)
                .with_path(path)
                .into());
        }

        match filetype {
            VfsFileType::RegularFile => self.insert_file_entry(entry_name),
            VfsFileType::Directory => self.insert_dir_entry(entry_name),
            _ => todo!("why got {}", filetype),
        }
    }

    /// 1. 申请一个 Inode
    /// 2. 在目录中创建一个目录项
    fn insert_file_entry(&mut self, filename: &str) -> VfsResult<Box<dyn VfsInode>> {
        let inode_id = self.allocator().lock().alloc_inode(false)? as usize;
        let inode = self.layout().new_inode_nth(
            inode_id,
            VfsFileType::RegularFile,
            self.layout(),
            self.allocator(),
        );

        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 filename -> inode_id 的映射关系
            dir.insert_entry(filename, inode_id, VfsFileType::RegularFile);
            // dir 仅仅是内存中的数据结构, 因此需要写回磁盘
            dir.write_to_disk(ext2_inode)
        })?;

        Ok(Box::new(inode))
    }

    /// 1. 申请一个 Inode
    /// 2. 在 dirname 下新建两个目录项, 分别是 . 和 .., 注意硬链接变化
    /// 3. 在目录中创建一个目录项
    fn insert_dir_entry(&mut self, dirname: &str) -> VfsResult<Box<dyn VfsInode>> {
        let inode_id = self.allocator().lock().alloc_inode(true)? as usize;
        let mut dir_inode = self.layout().new_inode_nth(
            inode_id,
            VfsFileType::Directory,
            self.layout(),
            self.allocator(),
        );

        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 entry_name -> inode_id 的映射关系
            dir.insert_entry(dirname, inode_id, VfsFileType::Directory);
            // 写回磁盘
            dir.write_to_disk(ext2_inode)
        })?;

        dir_inode.increase_to(block::SIZE)?;
        dir_inode.modify_disk_inode(|ext2_inode| {
            let mut dir = Dir::from_inode(inode_id, ext2_inode, self.layout(), self.allocator());
            // 建立 . -> inode_id 的映射关系
            dir.insert_entry(".", inode_id, VfsFileType::Directory);

            // 建立 .. -> inode_id 的映射关系
            dir.insert_entry("..", self.inode_id(), VfsFileType::Directory);

            // 一齐写回磁盘
            dir.write_to_disk(ext2_inode)
        })?;

        dir_inode.modify_disk_inode(|ext2_inode| {
            ext2_inode.inc_hard_links();
        });
        self.modify_disk_inode(|ext2_inode| {
            ext2_inode.inc_hard_links();
        });

        Ok(Box::new(dir_inode))
    }

    // hardlink 相比于其他 entry 区别: 不会申请 inode
    pub fn insert_hardlink(
        &mut self,
        path_from: &VfsPath,
        path_to: &VfsPath,
        target_inode: &Inode,
    ) -> VfsResult<()> {
        self.check_valid_insert(path_from)?;

        // 除了通用检查外, 硬链接只针对 file
        if !target_inode.is_file() {
            return Err(IOError::new(IOErrorKind::NotAFile)
                .with_path(path_to)
                .into());
        }

        let filename = path_from.last().unwrap();
        self.insert_hardlink_entry(filename, target_inode)
    }

    /// to 可能会不存在, 因此不能返回 to 的 inode,
    /// 另外也不能返回 Symlink 的 Inode, 因为这对用户没有意义
    pub fn insert_symlink(&mut self, path_from: &VfsPath, path_to: &VfsPath) -> VfsResult<()> {
        self.check_valid_insert(path_from)?;
        let filename = path_from.last().unwrap();
        let inode_id = self.allocator().lock().alloc_inode(false)? as usize;
        let mut inode = self.layout().new_inode_nth(
            inode_id,
            VfsFileType::SymbolicLink,
            self.layout(),
            self.allocator(),
        );
        inode.write_symlink(path_to)?;

        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 filename -> inode_id 的映射关系
            dir.insert_entry(filename, inode_id, VfsFileType::SymbolicLink);
            // dir 仅仅是内存中的数据结构, 因此需要写回磁盘
            dir.write_to_disk(ext2_inode)
        })?;

        Ok(())
    }

    fn insert_hardlink_entry(&mut self, filename: &str, target_inode: &Inode) -> VfsResult<()> {
        // 目录下插入新目录项
        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 filename -> inode_id 的映射关系
            dir.insert_entry(filename, target_inode.inode_id(), target_inode.filetype());
            // dir 仅仅是内存中的数据结构, 因此需要写回磁盘
            dir.write_to_disk(ext2_inode)
        })?;

        // 目标 inode 硬链接增加
        target_inode.modify_disk_inode(|ext2_inode| {
            ext2_inode.inc_hard_links();
        });
        Ok(())
    }

    pub fn remove_entry(&mut self, path: &VfsPath) -> VfsResult<()> {
        self.check_valid_remove(path)?;
        let entry_name = path.last().unwrap();
        let mut target_inode = self.select_child(entry_name)?;

        match target_inode.filetype() {
            VfsFileType::RegularFile => self.remove_file_entry(entry_name, &mut target_inode),
            VfsFileType::SymbolicLink => self.remove_symlink_entry(entry_name, &mut target_inode),
            VfsFileType::Directory => {
                if entry_name == "." || entry_name == ".." {
                    return Err(VfsError::new(
                        path,
                        VfsErrorKind::InvalidPath(path.to_string()),
                        "Forbidden to remove '.' or '..'".to_string(),
                    ));
                }
                self.remove_dir_entry(entry_name, &mut target_inode)
            }
            filetype => todo!("why got {}", filetype),
        }
    }

    /// 扣除 hardlink, 到 0 则释放
    fn remove_file_entry(&mut self, filename: &str, target_inode: &mut Inode) -> VfsResult<()> {
        let should_remove = self.unlink(filename, target_inode);
        if should_remove {
            // 释放目标文件的存储空间
            target_inode.set_len(0)?;
            // 释放目标文件对应的 inode, 在 bitmap 上清除位后, 对应的 inode 即不可用
            self.free_inode(target_inode.inode_id(), false)?;
        };
        Ok(())
    }

    fn remove_symlink_entry(&mut self, filename: &str, target_inode: &mut Inode) -> VfsResult<()> {
        // symlink 只需要删除目录项 和 inode 即可
        let should_remove = self.unlink(filename, &target_inode);
        if should_remove {
            self.free_inode(target_inode.inode_id(), false)?;
        }
        Ok(())
    }

    fn remove_dir_entry(&mut self, dirname: &str, target_inode: &mut Inode) -> VfsResult<()> {
        let dir_entries = target_inode.inner_read_dir();
        // 将目标目录下的所有目录项都删除
        for entry in &dir_entries {
            if entry.name() == "." || entry.name() == ".." {
                continue;
            }

            let mut sub_target_inode = entry.inode();
            let sub_target_filetype = sub_target_inode.filetype();
            if sub_target_filetype.is_dir() {
                target_inode.remove_dir_entry(entry.name(), &mut sub_target_inode)?;
            } else if sub_target_filetype.is_symlink() {
                target_inode.remove_symlink_entry(entry.name(), &mut sub_target_inode)?;
            } else {
                target_inode.remove_file_entry(entry.name(), &mut sub_target_inode)?;
            }
        }

        // remove .
        target_inode.modify_disk_inode(|ext2_inode| {
            ext2_inode.dec_hard_links();
        });
        // remove ..
        self.modify_disk_inode(|ext2_inode| {
            ext2_inode.dec_hard_links();
        });

        let should_remove = self.unlink(dirname, target_inode);
        assert!(should_remove);

        // 释放目录
        target_inode.set_len(0)?;
        // 释放目标文件对应的 inode, 在 bitmap 上清除位后, 对应的 inode 即不可用
        self.free_inode(target_inode.inode_id(), true)?;

        Ok(())
    }

    // 在当前 dir 下删除 entry -> target_inode 这一 entry 目录项, 该方法会递减 hardlinks
    fn unlink(&mut self, entry_name: &str, target_inode: &Inode) -> bool {
        assert!(self.is_dir());
        // 删除目录项
        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 filename -> inode_id 的映射关系
            dir.remove_entry(entry_name);
            // dir 仅仅是内存中的数据结构, 因此需要写回磁盘
            // remove entry 不可能扩容, 因此可以直接 unwarp
            dir.write_to_disk(ext2_inode).unwrap()
        });
        // 硬链接减1
        target_inode.modify_disk_inode(|ext2_inode| ext2_inode.dec_hard_links())
    }

    fn free_inode(&self, inode_id: usize, is_dir: bool) -> VfsResult<()> {
        self.allocator()
            .lock()
            .dealloc_inode(inode_id as u32, is_dir)
    }
}
