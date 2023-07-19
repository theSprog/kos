use alloc::collections::VecDeque;

use crate::sync::unicore::UPIntrFreeCell;

pub struct NS16550aRaw {
    base_addr: usize,
}

impl NS16550aRaw {
    pub fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }
}

struct NS16550aInner {
    ns16550a: NS16550aRaw,
    read_buffer: VecDeque<u8>,
}

pub struct NS16550a<const BASE_ADDR: usize> {
    inner: UPIntrFreeCell<NS16550aInner>,
    // condvar: Condvar,
}

impl<const BASE_ADDR: usize> NS16550a<BASE_ADDR> {
    pub fn new() -> Self {
        let inner = NS16550aInner {
            ns16550a: NS16550aRaw::new(BASE_ADDR),
            read_buffer: VecDeque::new(),
        };
        //inner.ns16550a.init();
        Self {
            inner: unsafe { UPIntrFreeCell::new(inner) },
            // condvar: Condvar::new(),
        }
    }

    pub fn read_buffer_is_empty(&self) -> bool {
        self.inner
            .exclusive_session(|inner| inner.read_buffer.is_empty())
    }
}
