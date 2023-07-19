use super::bus::VirtioHal;

use alloc::{collections::VecDeque, sync::Arc};
use component::{io::InputDevice, HandleIRQ};
use core::ptr::NonNull;
use virtio_drivers::{
    device::input::VirtIOInput,
    transport::mmio::{MmioTransport, VirtIOHeader},
};

use crate::sync::unicore::UPIntrFreeCell;

struct VirtIOInputInner {
    virtio_input: VirtIOInput<VirtioHal, MmioTransport>,
    events: VecDeque<u64>,
}

pub struct VirtIOInputImpl {
    inner: UPIntrFreeCell<VirtIOInputInner>,
    // condvar: Condvar,
}

impl VirtIOInputImpl {
    pub fn new(addr: usize) -> Self {
        let inner = unsafe {
            let header = addr as *mut VirtIOHeader;
            let transport = MmioTransport::new(NonNull::new(header).unwrap()).unwrap();

            VirtIOInputInner {
                virtio_input: VirtIOInput::new(transport).unwrap(),
                events: VecDeque::new(),
            }
        };
        Self {
            inner: unsafe { UPIntrFreeCell::new(inner) },
            // condvar: Condvar::new(),
        }
    }
}

impl HandleIRQ for VirtIOInputImpl {
    fn handle_irq(&self) {
        todo!();
        // let mut count = 0;
        // let mut result = 0;
        // self.inner.exclusive_session(|inner| {
        //     inner.virtio_input.ack_interrupt();
        //     while let Some(event) = inner.virtio_input.pop_pending_event() {
        //         count += 1;
        //         result = (event.event_type as u64) << 48
        //             | (event.code as u64) << 32
        //             | (event.value) as u64;
        //         inner.events.push_back(result);
        //     }
        // });
        // if count > 0 {
        //     self.condvar.signal();
        // };
    }
}

impl InputDevice for VirtIOInputImpl {
    fn is_empty(&self) -> bool {
        self.inner.exclusive_access().events.is_empty()
    }

    fn read_event(&self) -> u64 {
        todo!();
        // loop {
        //     let mut inner = self.inner.exclusive_access();
        //     if let Some(event) = inner.events.pop_front() {
        //         return event;
        //     } else {
        //         let task_cx_ptr = self.condvar.wait_no_sched();
        //         drop(inner);
        //         schedule(task_cx_ptr);
        //     }
        // }
    }
}

const VIRTIO4: usize = 0x10005000;
pub struct VirtIOKeyBoard(VirtIOInputImpl);
impl VirtIOKeyBoard {
    pub fn new() -> Self {
        Self(VirtIOInputImpl::new(VIRTIO4))
    }
}

impl HandleIRQ for VirtIOKeyBoard {
    fn handle_irq(&self) {
        self.0.handle_irq()
    }
}

impl InputDevice for VirtIOKeyBoard {
    fn read_event(&self) -> u64 {
        self.0.read_event()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

const VIRTIO5: usize = 0x10006000;

pub struct VirtIOMouse(VirtIOInputImpl);
impl VirtIOMouse {
    pub fn new() -> Self {
        Self(VirtIOInputImpl::new(VIRTIO5))
    }
}

impl HandleIRQ for VirtIOMouse {
    fn handle_irq(&self) {
        self.0.handle_irq()
    }
}

impl InputDevice for VirtIOMouse {
    fn read_event(&self) -> u64 {
        self.0.read_event()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
