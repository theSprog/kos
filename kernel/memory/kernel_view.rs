use core::ops::Range;

use crate::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};

extern "C" {
    // 内核起始地址
    fn kernel_start();
    fn stext();
    fn strampoline();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn sbss();
    fn ebss();
    // 内核结束地址
    fn kernel_end();
}

pub struct KernelView {
    pub kernel_start: usize,
    pub stext: usize,
    pub strampoline: usize,
    pub etext: usize,
    pub srodata: usize,
    pub erodata: usize,
    pub sdata: usize,
    pub edata: usize,
    pub sbss_with_stack: usize,
    pub sbss: usize,
    pub ebss: usize,
    pub kernel_end: usize,
}

impl KernelView {
    fn new() -> KernelView {
        KernelView {
            kernel_start: kernel_start as usize,
            stext: stext as usize,
            strampoline: strampoline as usize,
            etext: etext as usize,
            srodata: srodata as usize,
            erodata: erodata as usize,
            sdata: sdata as usize,
            edata: edata as usize,
            sbss_with_stack: sbss_with_stack as usize,
            sbss: sbss as usize,
            ebss: ebss as usize,
            kernel_end: kernel_end as usize,
        }
    }

    pub fn kernel_range(&self) -> Range<usize> {
        self.kernel_start..self.kernel_end
    }

    pub fn text_range(&self) -> Range<usize> {
        self.stext..self.etext
    }

    pub fn rodata_range(&self) -> Range<usize> {
        self.srodata..self.erodata
    }

    pub fn data_range(&self) -> Range<usize> {
        self.sdata..self.edata
    }

    pub fn bss_range(&self) -> Range<usize> {
        // 注意起始点不是 sbss_with_stack, 目前不知道为什么
        self.sbss..self.ebss
    }

    // pub fn trampoline_range(&self) -> Range<usize> {
    //     trampoline as usize..trampoline as usize + PAGE_SIZE
    // }

    // 每个应用程序的内核栈地址
    pub fn kernel_stack_range(&self, app_id: usize) -> (usize, usize) {
        let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let bottom = top - KERNEL_STACK_SIZE;
        (bottom, top)
    }
}

pub fn get_kernel_view() -> KernelView {
    KernelView::new()
}
