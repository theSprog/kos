const FB_VADDR: usize = 0x10000000;

pub fn sys_framebuffer() -> isize {
    todo!();
    // let fb = GPU_DEVICE.get_framebuffer();
    // let len = fb.len();
    // // println!("[kernel] FrameBuffer: addr 0x{:X}, len {}", fb.as_ptr() as usize , len);
    // let fb_start_pa = PhysAddr::from(fb.as_ptr() as usize);
    // assert!(fb_start_pa.aligned());
    // let fb_start_ppn = fb_start_pa.floor();
    // let fb_start_vpn = VirtAddr::from(FB_VADDR).floor();
    // let pn_offset = fb_start_ppn.0 as isize - fb_start_vpn.0 as isize;

    // let current_process = current_process();
    // let mut inner = current_process.inner_exclusive_access();
    // inner.memory_set.push(
    //     MapArea::new(
    //         (FB_VADDR as usize).into(),
    //         (FB_VADDR + len as usize).into(),
    //         MapType::Linear(pn_offset),
    //         MapPermission::R | MapPermission::W | MapPermission::U,
    //     ),
    //     None,
    // );
    // FB_VADDR as isize
}

pub fn sys_framebuffer_flush() -> isize {
    todo!()
    // GPU_DEVICE.flush();
    // 0
}
