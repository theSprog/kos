use super::{
    address::*,
    frame::{frame_alloc, PhysFrame},
    kernel_view::*,
    page_table::{PTEFlags, PageTable, PageTableEntry},
};

use crate::{
    bitflags::bitflags, unicore::UPSafeCell, util::human_size, MEMORY_END, PAGE_SIZE, TRAMPOLINE,
    TRAP_CONTEXT, USER_STACK_SIZE,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use logger::info;

// 内核空间
lazy_static! {
    // 为什么要用 arc?
    // 有多处引用因此要用 Rc, 又由于全局变量所有要用 Arc, Rc 只适合单线程
    pub(crate) static ref KERNEL_SPACE: Arc<UPSafeCell<AddressSpace>> ={
        info!("KERNEL_SPACE initializing...");
        Arc::new(unsafe { UPSafeCell::new(AddressSpace::new_kernel()) })
    };
}

bitflags! {
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
    vpn_range: VPNRange, // 一段连续虚拟内存，表示该逻辑段在地址区间中的位置和长度
    data_frames: BTreeMap<VirtPageNum, PhysFrame>, // 当 MapType 是 Framed 映射时有效
    map_type: MapType,   // 映射类型
    map_perm: MapPermission,
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
    pub fn map(&mut self, address_space_page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(address_space_page_table, vpn);
        }
    }

    pub fn unmap(&mut self, address_space_page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(address_space_page_table, vpn);
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

    /// 对逻辑段中的单个虚拟页面进行映射
    pub fn map_one(&mut self, address_space_page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = match self.map_type {
            MapType::Identical => PhysPageNum(vpn.0),
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                let ret = frame.ppn;
                self.data_frames.insert(vpn, frame);
                ret
            }
        };
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        address_space_page_table.map(vpn, ppn, pte_flags);
    }

    /// 对逻辑段中的单个虚拟页面进行解映射
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unmap(vpn);
    }
}

/// 地址空间
pub struct AddressSpace {
    // 页表自身也需要物理地址
    // 每个应用的地址空间都对应一个不同的多级页表，这也就意味页表根节点的地址是不一样的。
    // 因此 PageTable 要保存它根节点的物理页号 root_ppn 作为页表唯一的区分标志
    page_table: PageTable,

    // 地址空间由许多段组成
    segments: Vec<Segment>,
}

impl AddressSpace {
    // 申请一块裸空间
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            segments: Vec::new(),
        }
    }

    /// 向地址空间中压入一个逻辑段 (segment)
    /// 如果它是以 Framed 方式映射到物理内存，
    /// 还可以可选地在那些被映射到的物理页帧上写入一些初始化数据 data
    fn push(&mut self, mut segment: Segment, data: Option<&[u8]>) {
        segment.map(&mut self.page_table);
        if let Some(data) = data {
            segment.copy_data(&mut self.page_table, data);
        }
        self.segments.push(segment);
    }

    /// Assume that no conflicts.
    /// 插入一个以 framed 方式为映射的逻辑段, 供 User 调用
    pub fn insert_framed_segment(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            Segment::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    // 准备好内核虚实地址的关联，对内核进行恒等映射
    pub fn new_kernel() -> Self {
        info!("Creating kernel mapping");
        let kernel_view = get_kernel_view();
        let mut address_space = Self::new_bare();

        // 映射 trampoline
        address_space.map_trampoline();

        // 映射内核 sections
        info!(
            ".text    [{:#x}, {:#x}), size: {}",
            kernel_view.stext,
            kernel_view.etext,
            human_size(kernel_view.text_range().len())
        );
        info!(
            ".rodata  [{:#x}, {:#x}), size: {}",
            kernel_view.srodata,
            kernel_view.erodata,
            human_size(kernel_view.rodata_range().len())
        );
        info!(
            ".data    [{:#x}, {:#x}), size: {}",
            kernel_view.sdata,
            kernel_view.edata,
            human_size(kernel_view.data_range().len())
        );
        info!(
            ".bss     [{:#x}, {:#x}), size: {}",
            kernel_view.sbss_with_stack,
            kernel_view.ebss,
            human_size(kernel_view.bss_range().len())
        );

        info!(
            "free mem [{:#x}, {:#x}), size: {}",
            kernel_view.kernel_end,
            MEMORY_END,
            human_size(MEMORY_END - kernel_view.kernel_end)
        );

        // 所有逻辑段的 U 标志位均未被设置，
        // 使得 CPU 只能在处于 S 特权级（或以上）时访问它们
        // text 可读可执行
        info!("Mapping .text section");
        address_space.push(
            Segment::new(
                (kernel_view.stext).into(),
                (kernel_view.etext).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );

        // rodata 可读
        info!("Mapping .rodata section");
        address_space.push(
            Segment::new(
                (kernel_view.srodata).into(),
                (kernel_view.erodata).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );

        // data 可读可写
        info!("Mapping .data section");
        address_space.push(
            Segment::new(
                (kernel_view.sdata).into(),
                (kernel_view.edata).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        // bss 可读可写
        info!("Mapping .bss section");
        address_space.push(
            Segment::new(
                (kernel_view.sbss_with_stack).into(),
                (kernel_view.ebss).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        // 内核可以访问所有页面
        info!("Mapping physical memory");
        address_space.push(
            Segment::new(
                (kernel_view.kernel_end).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        info!("Kernel mapping done");
        address_space
    }

    /// 映射 ELF 的 sections 以及 trampoline、TrapContext(用于地址空间切换) 和 user stack,
    /// 返回 user_sp 和 entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        info!("Creating user ELF file mapping");
        let mut address_space = Self::new_bare();
        address_space.map_trampoline();

        // 用 U flag 映射用户程序
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(
            magic,
            [0x7f, b'E', b'L', b'F'],
            "invalid ELF!, magic number : {:#?}",
            magic
        );

        // 数清楚有多少 program header
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn: VirtPageNum = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            // 如果需要 load
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                // 计算出起始和结束地址
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();

                // 计算权限
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }

                let segment = Segment::new(start_va, end_va, MapType::Framed, map_perm);

                // max_end_vpn 此处被修改, 一直被修改到最后一个 section 的结束
                // PT_LOAD类型的代码段是根据 p_vaddr 来排布的，这就使得 max_end_vpn 可以严格递增
                max_end_vpn = segment.vpn_range.get_end();
                address_space.push(
                    segment,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }

        // map user stack with U flags
        // 此前的修改使得 max_end_vpn 已经在最后一个 section 的结尾地址处了
        let max_end_va: VirtAddr = max_end_vpn.into();
        // 用户栈栈底
        let mut user_stack_bottom: usize = max_end_va.into();
        // 跳过 guard page 保护页, 一旦栈溢出触发警告
        user_stack_bottom += PAGE_SIZE;
        // 用户栈栈顶, 从栈底延伸出一个 USER_STACK_SIZE 的空间大小
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        address_space.push(
            Segment::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        // map TrapContext
        address_space.push(
            Segment::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        // 返回
        (
            address_space,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    // trampoline 不是普通的用户态可以执行的, 而是内核态执行
    fn map_trampoline(&mut self) {
        let kernel_view = get_kernel_view();
        // 将虚拟空间中的 TRAMPOLINE 与物理空间中的 strampoline 联系起来
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(kernel_view.strampoline).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    // 开启内核内存空间
    pub fn enable_paging(&self) {
        let satp = self.page_table.token();

        info!("Activating paging mechanism");
        unsafe {
            // satp : Supervisor Address Translation and Protection
            // 写入页表基地址(物理地址), 开启分页
            // 切换任务的时候， satp 也必须被同时切换
            riscv::register::satp::write(satp);
            // 使用 sfence.vma 指令刷新清空整个 TLB
            // sfence.vma 可以使得所有发生在它后面的地址转换都能够看到所有排在它前面的写入操作
            // 相当于是个内存屏障
            core::arch::asm!("sfence.vma");
        }
        info!("Paging mechanism enabled");
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
}

pub fn remap_test() {
    let kernel_view = get_kernel_view();
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((kernel_view.stext + kernel_view.etext) / 2).into();
    let mid_rodata: VirtAddr = ((kernel_view.srodata + kernel_view.erodata) / 2).into();
    let mid_data: VirtAddr = ((kernel_view.sdata + kernel_view.edata) / 2).into();

    // text 不可写
    let mid_text_pte = kernel_space.page_table.translate(mid_text.floor()).unwrap();
    assert_eq!(mid_text_pte.valid(), true);
    assert_eq!(mid_text_pte.writable(), false);
    assert_eq!(mid_text_pte.executable(), true);
    assert_eq!(mid_text_pte.readable(), true);

    // rodata 不可写不可执行
    let mid_rodata_pte = kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap();
    assert_eq!(mid_rodata_pte.valid(), true);
    assert_eq!(mid_rodata_pte.writable(), false);
    assert_eq!(mid_rodata_pte.executable(), false);
    assert_eq!(mid_rodata_pte.readable(), true);

    // data 不可执行
    let mid_data_pte = kernel_space.page_table.translate(mid_data.floor()).unwrap();
    assert_eq!(mid_data_pte.valid(), true);
    assert_eq!(mid_data_pte.writable(), true);
    assert_eq!(mid_data_pte.executable(), false);
    assert_eq!(mid_data_pte.readable(), true);

    info!("Remap test passed, good luck!");
}
