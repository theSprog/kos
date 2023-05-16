use core::{assert_eq, todo, unreachable};

use super::{
    address::*,
    frame::PhysFrame,
    kernel_view::*,
    page_table::{PTEFlags, PageTable, PageTableEntry},
    segment::*,
};

use crate::{
    bitflags::bitflags, task, unicore::UPSafeCell, util::human_size, MEMORY_END, PAGE_SIZE,
    TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use logger::{debug, info};

// 内核空间
lazy_static! {
    // 为什么要用 arc?
    // 有多处引用因此要用 Rc, 又由于全局变量所有要用 Arc, Rc 只适合单线程
    pub(crate) static ref KERNEL_SPACE: Arc<UPSafeCell<AddressSpace>> ={
        info!("KERNEL_SPACE initializing...");
        Arc::new(unsafe { UPSafeCell::new(AddressSpace::new_kernel()) })
    };
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

    // 开启内核内存空间
    pub fn enable_paging(&self) {
        let satp = self.page_table.token();

        info!("Enabling paging mechanism");
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

    /// 向地址空间中添加一个逻辑段 (segment)
    /// 如果它是以 Framed 方式映射到物理内存，
    /// 还可以可选地在那些被映射到的物理页帧上写入一些初始化数据 data
    fn push(&mut self, mut segment: Segment, data: Option<&[u8]>) {
        segment.map(&mut self.page_table);
        if let Some(data) = data {
            segment.copy_data(&mut self.page_table, data);
        }
        self.segments.push(segment);
    }

    /// 以 lazy 的方式添加一个逻辑段, 只有访问该页的时候才会实现物理页分配与映射
    fn push_lazy(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// 填充虚拟内存, 可能会用到 elf 文件的对应内容。
    /// 也可能不会, 例如填充栈段就不用 elf 指定
    pub fn fill_one_page(&mut self, vpn: VirtPageNum) {
        let elf_data = crate::loader::load_app(task::api::current_tid());
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();

        // vpn 转为虚拟地址, 它必然是页对齐的
        let vaddr = VirtAddr::from(vpn).0;
        assert_eq!(0, vaddr % PAGE_SIZE);

        // 防御性检验, 该 vaddr 一定是在某一个 segment 中
        let seg = self.select_seg_by_vaddr(vaddr);
        assert!(seg.is_some());

        let ph = elf
            .program_iter()
            .filter(|phdr| {
                (phdr.virtual_addr() <= vaddr as u64)
                    && (vaddr as u64 <= phdr.virtual_addr() + phdr.mem_size())
            })
            .collect::<Vec<_>>();
        //该 vaddr 有可能来自 elf, 也有可能是其他非 elf 的段(例如用户栈就是 kernel 所设定的)
        assert!(ph.len() <= 1);

        if ph.len() == 1 {
            self.fill_one_page_from_elf(ph[0], &elf, vpn);
        }

        // 如果是来自其他段, 由于之前已经申请过页面了, 所以直接用就行了
    }

    /// 将 elf 中的某个程序段加载一页到虚拟内存(对应的物理内存)中
    fn fill_one_page_from_elf(
        &mut self,
        ph: xmas_elf::program::ProgramHeader,
        elf: &xmas_elf::ElfFile,
        vpn: VirtPageNum,
    ) {
        let vaddr = VirtAddr::from(vpn).0;
        assert_eq!(0, ph.virtual_addr() as usize % PAGE_SIZE);

        //先把需要加载的数据框定
        let data = &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
        // 需要加载的内容的起点
        let start = vaddr - ph.virtual_addr() as usize;

        // 需要加载的内容的大小, 最大一个页面
        let size = PAGE_SIZE.min(ph.file_size() as usize - start);

        // 划定源数据
        let src = &data[start..start + size];
        let pte = self.translate(vpn).unwrap();
        assert!(pte.valid());
        let dst = &mut pte.ppn().get_bytes_array()[..src.len()];
        dst.copy_from_slice(src);
        unsafe {
            core::arch::asm!("fence.i");
        }
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

        // 为应用程序申请一个地址空间
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

        // 计数有多少 program header
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn: VirtPageNum = VirtPageNum::empty();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            // 如果需要 load
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                // 计算出起始和结束地址
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                // 起始地址应该要页边界对齐
                assert_eq!(
                    0,
                    start_va.0 % PAGE_SIZE,
                    "ELF program start_vaddr({:#x}) should aligned with 4K",
                    start_va.0
                );

                // file_size 表示该段在文件中的大小
                // mem_size 表示该段在内存中的大小
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

                // 使用 lazy 方式, 只有当触发缺页时才分配物理页面
                address_space.push_lazy(segment);
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

        // 同样以 lazy 方式
        address_space.push_lazy(Segment::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ));

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

        // 返回值
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
        self.page_table.link(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(kernel_view.strampoline).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    pub fn is_page_fault(&self, vaddr: usize, perm: MapPermission) -> bool {
        for segment in &self.segments {
            // 段内地址包含且权限正确
            if segment.contains(vaddr) && segment.map_perm.contains(perm | MapPermission::U) {
                return true;
            }
        }
        false
    }

    pub fn select_seg_by_vaddr(&self, vaddr: usize) -> Option<&Segment> {
        let segs = self
            .segments
            .iter()
            .filter(|seg| seg.contains(vaddr as usize))
            .collect::<Vec<_>>();
        assert!(segs.len() <= 1);

        if segs.len() == 1 {
            Some(segs[0])
        } else {
            None
        }
    }

    // segment 自身包含着权限，直接取出用, 所以不需要再在参数中传递权限
    pub fn map_phys_page(&mut self, vaddr: usize) -> PhysPageNum {
        let vpn: VirtPageNum = VirtAddr(vaddr).floor();
        let pte = self.translate(vpn);
        assert!(pte.is_none() || !pte.unwrap().valid());
        for segment in &mut self.segments {
            if segment.contains(vaddr) {
                // 建立起映射
                return segment.alloc_one(&mut self.page_table, vpn);
            }
        }
        unreachable!("use valid_addr() before alloc_page !");
    }

    // 修复缺页异常
    pub fn fix_page_fault(&mut self, vaddr: usize) {
        // 分配物理页
        let ppn = self.map_phys_page(vaddr);
        // 找到虚拟页页号
        let vpn = VirtAddr(vaddr).floor();

        // 有可能需要将 ELF 文件对应虚拟地址中的数据搬迁到此处
        self.fill_one_page(vpn);
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
