//!  本 mod 存放地址相关的数据结构，提供各个地址的转换规则以及辅助函数
use super::page_table::PageTableEntry;
use crate::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::{fmt::Debug, ops::Add};

/// 物理地址
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

/// SV39 对应 56 位物理地址
const PA_WIDTH_SV39: usize = 56;
/// SV39 对应的 39 位虚拟地址
const VA_WIDTH_SV39: usize = 39;
/// 虚拟页号宽度
/// |       PPN        |  offset  |
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;

/// 允许从一个 usize 转换为物理地址, 只取低 56 位
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}
impl From<PhysAddr> for usize {
    fn from(val: PhysAddr) -> Self {
        val.0
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl PhysAddr {
    /// 取出页偏移
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    /// 向下、向上取出物理页号
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }

    pub fn get<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
}

/// 物理页页号
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PhysPageNum(pub usize);

/// 从一个 usize 转为 PPN, 同样只需要一定的范围
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}
impl From<PhysPageNum> for usize {
    fn from(val: PhysPageNum) -> Self {
        val.0
    }
}
// 从物理地址中取出页号
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        // self 是一个引用, 因此不能够转为 PhysAddr
        let pa: PhysAddr = (*self).into();

        unsafe {
            // from_raw_parts_mut 从一个原始指针和长度创建一个可变的切片
            // 下面的函数创建一个 512 长度的 PageTableEntry 切片
            // 为什么是 512? 因为每一级页目录数组长度为 512 (9位)。一页 4096 B, 一个项 8 B, 所以一页正好包含 4096/8 = 512 个
            core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 1 << 9)
        }
    }

    // 取出一个 page_size 的数据
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            // 取出一页数据
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE)
        }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

/// 虚拟地址
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VirtAddr(pub usize);

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

/// 虚拟页页号
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl VirtPageNum {
    pub fn empty() -> Self {
        VirtPageNum(0)
    }

    // 一个 VirtPageNum 分为三个部分, 分别指向下一级别页目录号
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idxs = [0usize; 3];

        // 高位是高层页表, 低位是低层页表, 因此需要逆转
        // |  0级  |  1级  |  2级   |
        for i in (0..3).rev() {
            // 取出 9 位
            idxs[i] = vpn & 0b1_1111_1111;
            vpn >>= 9;
        }
        idxs
    }
}

impl core::ops::Sub for VirtPageNum {
    type Output = usize;

    fn sub(self, other: VirtPageNum) -> Self::Output {
        self.0 - other.0
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl Add for VirtPageNum {
    type Output = VirtPageNum;

    fn add(self, rhs: Self) -> Self::Output {
        VirtPageNum::from(self.0 + rhs.0)
    }
}

pub type VPNRange = SimpleRange<VirtPageNum>;

pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}
impl StepByOne for PhysPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

// [l, r)
#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }

    // special for heap
    pub fn set_end(&mut self, end: T) {
        self.r = end;
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
/// iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            // [l, r) if l == r
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
