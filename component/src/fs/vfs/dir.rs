use core::fmt::Debug;

use alloc::boxed::Box;

use super::VfsInode;

pub trait VfsDirEntry: Debug {
    fn name(&self) -> &str;
    fn inode_id(&self) -> usize;

    fn inode(&self) -> Box<dyn VfsInode> {
        unimplemented!()
    }
}
