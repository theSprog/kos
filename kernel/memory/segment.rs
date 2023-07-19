use alloc::{collections::BTreeMap, sync::Arc};
use logger::*;

use crate::{bitflags::bitflags, process::processor, PAGE_SIZE};

use super::{
    address::*,
    frame::{self, PhysFrame},
    page_table::{PTEFlags, PageTable},
};

bitflags! {
    // 注意这里没有 V 有效位, 他是需要手动设置 PTEFlags 的
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
    pub data_frames: BTreeMap<VirtPageNum, Arc<PhysFrame>>, // 当 MapType 是 Framed 映射时有效
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
        // 结束点上浮到页边界, 作为末尾页号(界限)
        let end_vpn: VirtPageNum = end_va.ceil();

        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    /// 从另一个进程拷贝过来, 同时更新页表
    pub fn from_another(another: &Self, new_page_table: &mut PageTable) -> Self {
        // 另一个一定是用户态进程
        assert!(another.map_perm.contains(MapPermission::U));

        // 复制物理页映射, 新进程一定包含这些页映射。
        // 同时更新新进程的页表
        let mut data_frames = BTreeMap::new();
        for (vpn, pf) in &another.data_frames {
            // Arc 复制引用计数
            data_frames.insert(*vpn, pf.clone());

            let ppn = pf.ppn;
            let pte_flags = PTEFlags::from_bits(another.map_perm.bits()).unwrap();
            // link 过程中会创建不存在的页目录项和页表项
            new_page_table.link(*vpn, ppn, pte_flags)
        }

        Self {
            vpn_range: VPNRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
            data_frames,
            map_type: another.map_type,
            map_perm: another.map_perm,
        }
    }

    // trap 不会复制父进程的页映射, 因为父子进程几乎必定不同, cow 没有意义
    pub fn from_trap(trap_seg: &Segment) -> Self {
        assert!(trap_seg
            .map_perm
            .contains(MapPermission::R | MapPermission::W));

        // trap 必然只有一页, 否则出错
        assert_eq!(
            1,
            trap_seg.vpn_range.get_end() - trap_seg.vpn_range.get_start()
        );
        Self {
            vpn_range: VPNRange::new(trap_seg.vpn_range.get_start(), trap_seg.vpn_range.get_end()),
            data_frames: BTreeMap::new(),
            map_type: trap_seg.map_type,
            map_perm: trap_seg.map_perm,
        }
    }

    /// map unmap 将当前逻辑段到物理内存的映射
    /// 从(传入的)该逻辑段所属的地址空间(AddressSpace)的多级页表中加入或删除
    /// 需要注意的是 map 会申请内存, 他是给 vpn 申请一个物理页面
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.alloc_one(page_table, vpn);
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.dealloc_one(page_table, vpn);
        }
    }

    // 将自身 (segment) 权限重新 map 一次
    pub fn remap(&mut self, page_table: &mut PageTable) {
        // 遍历所有已分配物理内存的 linked 的映射
        for (vpn, frame) in self.data_frames.iter() {
            let ppn = frame.ppn;
            let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
            page_table.relink(*vpn, ppn, pte_flags)
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
                .get_one_page()[..src.len()];
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
                // 分配物理页面, 必须将其保存至一个容器中, 否则生命周期只限于本作用域
                let frame = frame::api::frame_alloc().unwrap();
                let ret = frame.ppn;
                assert!(!self.data_frames.contains_key(&vpn));
                self.data_frames.insert(vpn, Arc::new(frame));
                ret
            }
        };
        // segment 中包含 self.map_perm 字段, 用于设置该页的权限
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        page_table.link(vpn, ppn, pte_flags);
        ppn
    }

    // 由于 cow 重新分配一个物理页面
    pub fn realloc_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) -> PhysPageNum {
        trace!(
            "realloc_one alloc new page for pid={}",
            processor::api::current_pid()
        );
        let ppn = match self.map_type {
            MapType::Identical => unreachable!("This is just for user cow!"),
            MapType::Framed => {
                // 分配物理页面, 必须将其保存至一个容器中, 否则生命周期只限于本作用域
                let frame = frame::api::frame_alloc().unwrap();
                let ret = frame.ppn;
                assert!(self.data_frames.contains_key(&vpn));
                self.data_frames.insert(vpn, Arc::new(frame));
                ret
            }
        };
        // segment 中包含 self.map_perm 字段, 用于设置该页的权限
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        assert!(pte_flags.contains(PTEFlags::W));
        page_table.relink(vpn, ppn, pte_flags);
        ppn
    }

    /// 对逻辑段中的单个虚拟页面进行解映射, 同时物理资源也被释放
    pub fn dealloc_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                // 这里有意思的是, 我们没有显式调用 frame_dealloc,
                // 当离开作用域后, 由于 RAII, frame_dealloc 会被自动调用
                // 因为 remove 会移出所有权, 如果引用计数归零则说明没有页表项指向该页面
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
