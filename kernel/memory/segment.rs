use alloc::collections::BTreeMap;

use crate::{bitflags::bitflags, PAGE_SIZE};

use super::{
    address::*,
    frame::{self, PhysFrame},
    page_table::{PTEFlags, PageTable},
};

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical, // 恒等映射(虚拟地址 = 物理地址, 主要用于内核)
    Framed,    // 每个虚拟页面都有一个新分配的物理页帧与之对应
}

/// 以逻辑段 MapArea 为单位描述一段地址连续的虚拟内存
/// 例如代码段, 数据段, 只读数据段等
pub struct Segment {
    pub vpn_range: VPNRange, // 一段连续虚拟内存，表示该逻辑段在地址区间中的位置和长度
    pub data_frames: BTreeMap<VirtPageNum, PhysFrame>, // 当 MapType 是 Framed 映射时有效
    pub map_type: MapType,   // 映射类型
    pub map_perm: MapPermission,
}

impl Segment {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        // 通过这两个操作扩充了虚拟页面范围, 扩充虚拟地址范围会产生冲突么 ?
        // 起始点下沉到页边界
        let start_vpn: VirtPageNum = start_va.floor();
        // 结束点上浮到页边界
        let end_vpn: VirtPageNum = end_va.ceil();

        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    /// map unmap 将当前逻辑段到物理内存的映射
    /// 从(传入的)该逻辑段所属的地址空间(AddressSpace)的多级页表中加入或删除
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.alloc_one(page_table, vpn);
        }
    }

    pub fn unmap(&mut self, address_space_page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.dealloc_one(address_space_page_table, vpn);
        }
    }

    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            // 逐页逐页地拷贝
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }

    /// 对逻辑段中的单个虚拟页面进行映射, 不需要指定物理页号, 该函数会自己分配一个页面
    /// 返回分配的页面的物理页号
    pub fn alloc_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) -> PhysPageNum {
        let ppn = match self.map_type {
            MapType::Identical => PhysPageNum(vpn.0),
            MapType::Framed => {
                // 分配物理页面
                let frame = frame::api::frame_alloc().unwrap();
                let ret = frame.ppn;
                self.data_frames.insert(vpn, frame);
                ret
            }
        };
        // segment 中包含 self.map_perm 字段, 用于设置该页的权限
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        page_table.link(vpn, ppn, pte_flags);
        ppn
    }

    /// 对逻辑段中的单个虚拟页面进行解映射
    pub fn dealloc_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unlink(vpn);
    }

    pub fn contains(&self, vaddr: usize) -> bool {
        let start_addr: VirtAddr = self.vpn_range.get_start().into();
        let end_addr: VirtAddr = self.vpn_range.get_end().into();
        start_addr.0 <= vaddr && vaddr < end_addr.0
    }
}
