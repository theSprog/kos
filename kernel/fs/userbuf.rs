use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }

    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }

    // 该函数假定不会溢出
    pub fn write(&mut self, content: &[u8]) {
        let mut begin = 0;
        for slice in self.buffers.iter_mut() {
            let slice_len = slice.len();
            let rest_len = content.len() - begin;
            // 1. 如果此 slice 足以容纳剩余 content
            // 2. 如果剩余 content 大于 slice, 则还需要下一次填充
            let this_len = rest_len.min(slice_len);
            let src = &content[begin..begin + this_len];
            let dst = &mut slice[..this_len];
            dst.copy_from_slice(src);
            begin += src.len();
        }
        assert_eq!(begin, content.len());
    }
}

pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    // buffer 所在索引
    buffer_idx: usize,
    // buffer 内数据偏移量
    data_offset: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_idx >= self.buffers.len() {
            None
        } else {
            let cur_buffer = &mut self.buffers[self.buffer_idx];
            let byte = &mut cur_buffer[self.data_offset] as *mut _;
            if self.data_offset + 1 == cur_buffer.len() {
                self.data_offset = 0;
                self.buffer_idx += 1;
            } else {
                self.data_offset += 1;
            }
            Some(byte)
        }
    }
}

impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            buffer_idx: 0,
            data_offset: 0,
        }
    }
}
