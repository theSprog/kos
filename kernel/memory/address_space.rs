use core::{assert_eq, unreachable};

use super::{
    address::*,
    kernel_view::*,
    page_table::{PTEFlags, PageTable, PageTableEntry},
    segment::*,
};

use crate::{
    memory::{heap_alloc, page_table},
    sync::unicore::UPSafeCell,
    trap::context::TrapContext,
    MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE,
};
use alloc::{
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use component::{
    crt0::{Builder, Entry},
    util::human_size::*,
};
use logger::info;
use qemu_config::MMIO;
use sys_interface::config::USER_PROG_PATH;

// 内核空间
lazy_static! {
    // 为什么要用 arc?
    // 有多处引用因此要用 Rc, 又由于全局变量所有要用 Arc, Rc 只适合单线程
    pub(crate) static ref KERNEL_SPACE: Arc<UPSafeCell<AddressSpace>> ={
        info!("KERNEL_SPACE initializing...");
        Arc::new(unsafe { UPSafeCell::new(AddressSpace::new_kernel()) })
    };
}

pub fn kernel_token() -> usize {
    KERNEL_SPACE.exclusive_access().token()
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
    pub fn from_fork(parent_space: &mut Self) -> Self {
        let mut new_space = Self::new_bare();
        // map trampoline
        new_space.map_trampoline();

        // 从 user_space 复制 trap_context,
        // 每一个进程都有自己的 trap_context, 但初始时候都一样
        let trap_seg = Segment::from_trap(&parent_space.segments[0]);
        let trap_content = parent_space.trap_ppn().get_bytes_array();

        // 向新 address_space 添加一个段, 并且放置初始内容
        // 注意这不能够 COW 因为两个进程的 trap 必定不一样(至少返回值不一样)
        new_space.push(trap_seg, Some(trap_content));

        // 复制 segment/user_stack/heap, 跳过 trap_segment
        for seg in parent_space.segments.iter_mut().skip(1) {
            if seg.map_perm.contains(MapPermission::W) {
                // 移除写权限, 父进程和子进程之后都不能写该 Segment
                // 一旦发生写, 那么会触发故障从而修复写故障, 实现 cow
                seg.map_perm.remove(MapPermission::W);
                // 先将所有写权限擦除
                seg.remap(&mut parent_space.page_table);

                // 子进程同样没有写权限
                let mut new_seg = Segment::from_another(seg, &mut new_space.page_table);

                new_seg.map_perm.insert(MapPermission::W);
                new_space.push_lazy(new_seg);

                seg.map_perm.insert(MapPermission::W);
            } else {
                // 没有 W 权限, 直接复制
                let new_seg = Segment::from_another(seg, &mut new_space.page_table);
                new_space.push_lazy(new_seg);
            }
        }

        new_space
    }

    pub fn page_table(&self) -> &PageTable {
        &self.page_table
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    // 把倒数第二个 segement 必须设置为 stack 段
    pub fn stack_mut(&mut self) -> &mut Segment {
        assert!(self.segments.len() >= 2);
        let idx = self.segments.len() - 2;
        let stack_seg = &mut self.segments[idx];
        // 必然是用户态访问
        assert!(stack_seg.map_perm.contains(MapPermission::U));
        stack_seg
    }

    // 把倒数第一个 segement 必须设置为 heap 段
    // heap 是可变的
    pub fn heap_mut(&mut self) -> &mut Segment {
        assert!(self.segments.len() >= 2);
        let idx = self.segments.len() - 1;
        let heap_seg = &mut self.segments[idx];
        // 必然是用户态访问
        assert!(heap_seg.map_perm.contains(MapPermission::U));
        heap_seg
    }

    // 回收所有空间, 同时回收页表
    pub fn release_space(&mut self) {
        self.segments.clear();
        self.page_table.clear();
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

    /// 假设 seg 之间没有两个段占用同一页面
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
            debug_size(kernel_view.text_range().len())
        );
        info!(
            ".rodata  [{:#x}, {:#x}), size: {}",
            kernel_view.srodata,
            kernel_view.erodata,
            debug_size(kernel_view.rodata_range().len())
        );
        info!(
            ".data    [{:#x}, {:#x}), size: {}",
            kernel_view.sdata,
            kernel_view.edata,
            debug_size(kernel_view.data_range().len())
        );
        info!(
            ".bss     [{:#x}, {:#x}), size: {}",
            kernel_view.sbss_with_stack,
            kernel_view.ebss,
            debug_size(kernel_view.bss_range().len())
        );

        info!(
            "free mem [{:#x}, {:#x}), size: {}",
            kernel_view.kernel_end,
            MEMORY_END,
            debug_size(MEMORY_END - kernel_view.kernel_end)
        );

        // 各种驱动映射
        for (start, len) in MMIO {
            let range = *start..(start + len);
            info!(
                "driver   [{:#x}, {:#x}], size: {}",
                range.start,
                range.end,
                debug_size(range.len())
            );
        }

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

        info!("mapping memory-mapped registers");
        for pair in MMIO {
            address_space.push(
                Segment::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }

        info!("Kernel mapping done");
        heap_alloc::api::display_heap_info();

        address_space
    }

    /// 映射 ELF 的 sections 以及 trampoline、TrapContext(用于地址空间切换) 和 user stack,
    /// 返回 user_sp 和 entry point.
    pub fn from_elf(elf_data: &[u8], pid: usize) -> (Self, usize, usize) {
        info!("Creating user ELF file mapping for pid={}", pid);

        // 为应用程序申请一个地址空间
        let mut address_space = Self::new_bare();

        // map trampoline
        address_space.map_trampoline();

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

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();

        let mut max_end_vpn: VirtPageNum = VirtPageNum::empty();
        for ph in elf.program_iter() {
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
                assert_ne!(ph.mem_size(), 0, "zeroed memory?");

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

                address_space.push(
                    segment,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }

        // 用户堆栈都用 push_lazy 方式
        // 先处理用户栈
        // 此前的修改使得 max_end_vpn 已经在最后一个 section 的结尾地址处了
        let max_end_va: VirtAddr = max_end_vpn.into();
        // 用户栈栈底
        let mut user_stack_bottom: usize = max_end_va.into();
        // 跳过 guard page 保护页, 一旦栈溢出触发警告
        user_stack_bottom += PAGE_SIZE;
        // 用户栈栈顶, 从栈底延伸出一个 USER_STACK_SIZE 的空间大小
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        info!("user_stack_top = {:#x} for pid={}", user_stack_top, pid);

        let mut stack_seg = Segment::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        // 由于 crt0 的空间需要提前分配一页
        stack_seg.alloc_one(
            &mut address_space.page_table,
            (stack_seg.vpn_range.get_end().0 - 1).into(),
        );
        address_space.push_lazy(stack_seg);

        // 用户堆内存, 堆向高地址生长, 最初时无内存
        // 加上 PAGE_SIZE 是为了 guard page
        let heap_start_va = (user_stack_top + PAGE_SIZE).into();
        let heap_end_va = heap_start_va;
        address_space.push_lazy(Segment::new(
            heap_start_va,
            heap_end_va,
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ));

        heap_alloc::api::display_heap_info();

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
        // 这是所有进程共享的, 不需要单独分配页面
        self.page_table.link(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(kernel_view.strampoline).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    pub fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    // 找到该地址空间的 trap 的 ppn
    pub fn trap_ppn(&self) -> PhysPageNum {
        let trap = self.translate_vpn(VirtAddr::from(TRAP_CONTEXT).into());
        assert!(
            trap.is_some(),
            "trap should be initialized in address_space"
        );
        trap.unwrap().ppn()
    }

    // page fault 有两种: 写 cow 和懒加载
    pub fn is_page_fault(&self, vaddr: usize, perm: MapPermission) -> bool {
        for segment in &self.segments {
            // 段内地址包含 且 权限正确
            if segment.contains(vaddr) && segment.map_perm.contains(perm | MapPermission::U) {
                return true;
            }
        }
        false
    }

    // segment 自身包含着权限，直接取出用, 所以不需要再在参数中传递权限
    // 此函数会分配物理页面
    pub fn map_phys_page(&mut self, vaddr: usize) -> PhysPageNum {
        let vpn: VirtPageNum = VirtAddr(vaddr).floor();
        let pte = self.translate_vpn(vpn);
        // 此前没有分配过
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
    pub fn fix_page_missing(&mut self, vaddr: usize) {
        // 只有可能是堆栈缺页, 分配物理页即可
        self.map_phys_page(vaddr);
    }

    // 修复 cow 异常
    pub fn fix_cow(&mut self, vaddr: usize) {
        let vpn: VirtPageNum = VirtAddr(vaddr).floor();
        let pte = self.page_table.find_pte(vpn).unwrap();
        assert!(pte.valid());
        // 找到引发故障的源物理页面
        let src_ppn = pte.ppn();

        for segment in &mut self.segments {
            if segment.contains(vaddr) {
                assert!(segment.data_frames.contains_key(&vpn));
                let pf = segment.data_frames.get(&vpn).unwrap();
                assert_eq!(pf.ppn, src_ppn);

                // 只有一个进程引向该页面
                if Arc::strong_count(pf) == 1 {
                    assert!(segment.map_perm.contains(MapPermission::W));
                    let pte_flags = PTEFlags::from_bits(segment.map_perm.bits()).unwrap();
                    // 重新为其赋予可写权限
                    self.page_table.relink(vpn, src_ppn, pte_flags);
                } else {
                    // 有多个进程引用该页面, 重新分配一页
                    // 分配物理页面, 权限包含在该 segment 中
                    // realloc_one 内部会更新 data_frames, 自动将 Arc 引用计数减一
                    // 同时不需要显式 relink 因为 realloc_one 内部会完成
                    let dst_ppn = segment.realloc_one(&mut self.page_table, vpn);
                    // 从源物理页拷贝到目的物理页
                    dst_ppn
                        .get_bytes_array()
                        .copy_from_slice(src_ppn.get_bytes_array());
                }

                // 一旦修复立即返回
                return;
            }
        }

        unreachable!();
    }

    pub fn release_kernel_stack_segment(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, seg)) = self
            .segments
            .iter_mut()
            .enumerate()
            .find(|(_, seg)| seg.vpn_range.get_start() == start_vpn)
        {
            // 将 kernel stack seg 从 page_table 中全部释放
            // 由于内核栈是立即分配的所以才能够将 seg 中的所有 vpn 都 dealloc_one
            seg.unmap(&mut self.page_table);
            // 同时 segments 中也删除对应的 segment
            self.segments.remove(idx);
        }
    }

    pub(crate) fn push_crt0(
        &mut self,
        trap_cx: &mut TrapContext,
        args: &[String],
        envs: &[String],
    ) {
        // 拨动栈指针
        trap_cx.x[2] -= PAGE_SIZE;

        let stack_frame_top =
            page_table::api::translated_one_page(self.token(), trap_cx.x[2] as *const u8);

        let mut builder_arg = Builder::new(stack_frame_top, trap_cx.x[2]);
        for arg in args {
            builder_arg.push(arg).unwrap();
        }

        let mut builder_env = builder_arg.done().unwrap();
        for env in envs {
            builder_env.push(env).unwrap();
        }

        let mut builder_aux = builder_env.done().unwrap();

        let auxv = [
            Entry::Gid(1000),
            Entry::Uid(1001),
            Entry::Platform("RISCV".to_string()),
        ];
        auxv.iter().for_each(|e| builder_aux.push(e).unwrap());
    }

    // 专为 init 进程准备的
    pub(crate) fn init_crt0(&self, trap_cx: &mut TrapContext) {
        // 拨动栈指针
        trap_cx.x[2] -= PAGE_SIZE;

        let stack_frame_top =
            page_table::api::translated_one_page(self.token(), trap_cx.x[2] as *const u8);

        let mut builder_arg = Builder::new(stack_frame_top, trap_cx.x[2]);

        builder_arg.push("init").unwrap();
        let mut builder_env = builder_arg.done().unwrap();

        builder_env
            .push(&format!("HOME={}", USER_PROG_PATH))
            .unwrap();
        let mut builder_aux = builder_env.done().unwrap();

        let auxv = [
            Entry::Gid(1000),
            Entry::Uid(1001),
            Entry::Platform("RISCV".to_string()),
        ];
        auxv.iter().for_each(|e| builder_aux.push(e).unwrap());
    }
}

#[allow(clippy::bool_assert_comparison)]
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
