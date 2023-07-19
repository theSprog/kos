use alloc::sync::Arc;

use crate::fs::BLOCK_DEVICE;

pub struct PLIC {
    base_addr: usize,
}
impl PLIC {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }
}

#[derive(Copy, Clone)]
pub enum IntrTargetPriority {
    Machine = 0,
    Supervisor = 1,
}

// lazy_static::lazy_static!(
//     pub static ref KEYBOARD_DEVICE: Arc<dyn InputDevice> = Arc::new(VirtIOInputWrapper::new(VIRTIO5));
//     pub static ref MOUSE_DEVICE: Arc<dyn InputDevice> = Arc::new(VirtIOInputWrapper::new(VIRTIO6));
// );

pub const VIRT_PLIC: usize = 0xC00_0000;

pub fn irq_handler() {
    todo!();

    // let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    // // let intr_src_id = plic.claim(0, IntrTargetPriority::Supervisor);
    // match intr_src_id {
    //     8 => BLOCK_DEVICE.handle_irq(),

    //     // 5 => KEYBOARD_DEVICE.handle_irq(),
    //     // 6 => MOUSE_DEVICE.handle_irq(),
    //     // 10 => UART.handle_irq(),
    //     _ => panic!("unsupported IRQ {}", intr_src_id),
    // }
    // plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
}
