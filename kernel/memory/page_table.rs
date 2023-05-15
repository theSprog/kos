use alloc::vec::Vec;
use bitflags::*;

use super::{
    address::*,
    frame::{frame_alloc, PhysFrame},
};

/// 页表
pub struct PageTable {
    // 页表起始页号
    root_ppn: PhysPageNum,

    // 已经被分配的物理页
    frames: Vec<PhysFrame>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: alloc::vec![frame],
        }
    }

    // 建立虚拟页对物理页的映射, flags 是权限设置
    // 从某种意义上说，也是在向虚拟空间申请虚拟内存
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        // 此处如果找不到页表项(物理页耗尽)则会返回 None, 所以我们 unwarp 会 panic
        // 目前的实现方式并不打算对物理页帧耗尽的情形做任何处理而是直接 panic 退出
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.valid(), "vpn {:?} is mapped before mapping", vpn);
        // 使用 PTEFlags::V 标记在虚拟空间中已分配
        // 在所找到的页表项上写上物理地址，从而完成 map
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.valid(), "vpn {:?} is invalid before unmapping", vpn);
        // 置空
        *pte = PageTableEntry::empty();
    }

    // 给定虚拟页号，找到页表项，找不到就建立
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        // 分解页号
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            // 仅负责从VPN查到页表项，但是并不要求这个页表项必须合法，
            // 这个检查工作应该由 find_pte_create 的调用者完成
            if i == 2 {
                // 已经是最后一级页表了
                result = Some(pte);
                break;
            }
            // 无效页面, 需要置为有效
            if !pte.valid() {
                let frame = frame_alloc().unwrap();
                // PTEFlags::V 标记被分配
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            // 进入下一级页表
            ppn = pte.ppn();
        }
        result
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    // 临时创建一个专用来手动查页表的 PageTable
    // 它仅有一个从传入的 satp token 中得到的多级页表根节点的物理页号
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    // 从一个虚拟页号手动查询页表, 拿到最后的页表项
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }

    /// 用于设定 satp csr 寄存器
    /// 当 MODE 设置为 0 的时候，代表所有访存都被视为物理地址；
    /// 而设置为 8 的时候，SV39 分页机制被启用，所有 S/U 特权级的访存被视为一个 39 位的虚拟地址
    /// 它们于是需要经过 MMU 的地址转换流程
    pub fn token(&self) -> usize {
        let mode = 0b1000 as usize;
        // << 优先级高于 |
        mode << 60 | self.root_ppn.0
    }
}

bitflags! {
    /// 定义一个结构体, bit 位分别如下所示
    #[derive(PartialEq)]
    pub struct PTEFlags: u8 {
        const V = 1 << 0;   // 是否在虚拟内存中被分配
        const R = 1 << 1;   // R 可读
        const W = 1 << 2;   // W 可写
        const X = 1 << 3;   // X 可执行
        const U = 1 << 4;   // 用户态可访问
        const G = 1 << 5;   //
        const A = 1 << 6;   // 从该位被清零之后，页表项的对应虚拟页面是否被访问过
        const D = 1 << 7;   // 从该位被清零之后，页表项的对应虚拟页面是否被修改过(是否是脏页)
    }
}

/// 页表项数据结构
/// 只有当 V 为1 且 R/W/X 均为 0 时，表示是一个合法的"页目录"表项，其包含的指针会指向下一级的页表；
/// 当 V 为1 且 R/W/X 不全为 0 时，表示是一个合法的"页表项"，其包含了虚地址对应的物理页号
/// 一个页表项 8 bytes, 一页可以容纳 512 个页表项
/// 一级页表的每个页表项中的物理页号可描述一个二级页表；
/// 二级页表的每个页表项中的物理页号可描述一个三级页表；
/// 三级页表中的页表项内容则是正常页表项，其内容包含物理页号，即描述一个要映射到的物理页
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            // 物理页的低十位是 flags
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    /// 初始时 V 标志为 0, 代表该页没有被虚拟内存分配
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    /// 取出 PPN
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    /// 取出 PTEFlags
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// 查询给定 token 的地址空间页表从而访问数据, 一般而言是在内核访问用户空间数据的时候
/// translated_byte_buffer 将用户应用地址空间中一个缓冲区转化为在内核空间中能够直接访问的形式
/// 之所以用 vec 是因为数据有可能跨页，一旦跨页数据就会被拆开，因此以 Vec 的形式返回
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut ret = Vec::new();

    // start 和 end 可能在不同的物理页, 因此逐个处理
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();

        //先设定为下一页页首
        let mut end_va: VirtAddr = vpn.into();
        // 比较 end 和 下一页页首的大小，以此判断数据是否跨页
        end_va = end_va.min(VirtAddr::from(end));

        if end_va.page_offset() == 0 {
            // 如果跨页
            ret.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            // 没有跨页
            ret.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    ret
}
