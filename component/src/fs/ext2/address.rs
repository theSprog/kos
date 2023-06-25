use core::{
    fmt::{self, Display, LowerHex},
    ops::{Add, Sub},
};

use super::block;

/// Address in a physical sector
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Address {
    block_id: usize,
    offset: usize,
}

impl Address {
    /// # Safety
    pub unsafe fn new_unchecked(block_id: usize, offset: usize) -> Self {
        assert!(offset < block::SIZE, "offset out of block bounds");
        Self { block_id, offset }
    }

    pub fn new(block_id: usize, offset: isize) -> Self {
        let block_size = block::SIZE as isize;

        let new_block_id = block_id as isize + (offset >> 12);
        assert!(
            new_block_id >= 0,
            "block_id={}, offset={}",
            block_id,
            offset
        );

        let new_offset = (offset % block_size + block_size) % block_size;
        assert!(new_offset >= 0 && new_offset < block_size);

        unsafe { Self::new_unchecked(new_block_id as usize, new_offset as usize) }
    }

    pub fn block_id(&self) -> usize {
        self.block_id
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.block_id, self.offset)
    }
}

impl LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}:{:x}", self.block_id, self.offset)
    }
}

impl From<usize> for Address {
    fn from(idx: usize) -> Address {
        let block_id = idx >> block::LOG_SIZE;
        let offset = (idx & block::MASK) as isize;
        Address::new(block_id, offset)
    }
}

impl From<Address> for usize {
    fn from(addr: Address) -> usize {
        (addr.block_id << block::LOG_SIZE) | addr.offset
    }
}

impl Add for Address {
    type Output = Address;
    fn add(self, rhs: Address) -> Address {
        Address::new(
            self.block_id + rhs.block_id,
            (self.offset + rhs.offset) as isize,
        )
    }
}

impl Sub for Address {
    type Output = Address;
    fn sub(self, rhs: Address) -> Address {
        Address::new(
            self.block_id - rhs.block_id,
            self.offset as isize - rhs.offset as isize,
        )
    }
}
