use core::ops::{Index, IndexMut};

use alloc::{sync::Arc, vec::Vec};

use crate::fs::{
    stdio::{Stderr, Stdin, Stdout},
    File,
};

#[derive(Clone)]
pub struct FdTable(Vec<Option<Arc<dyn File>>>);

impl FdTable {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn alloc_fd(&mut self) -> usize {
        // 寻找最小的可用 fd
        if let Some(fd) = (0..self.len()).find(|&fd| self[fd].is_none()) {
            fd
        } else {
            // 若没有的话新建一个
            self.0.push(None);
            self.0.len() - 1
        }
    }
}

impl Default for FdTable {
    fn default() -> Self {
        Self(alloc::vec![
            // 0 -> stdin
            Some(Arc::new(Stdin)),
            // 1 -> stdout
            Some(Arc::new(Stdout)),
            // 2 -> stderr
            Some(Arc::new(Stderr)),
        ])
    }
}

impl Index<usize> for FdTable {
    type Output = Option<Arc<dyn File>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for FdTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
