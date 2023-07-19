mod ns16550a;

use alloc::sync::Arc;
use lazy_static::*;

use ns16550a::NS16550a;

pub trait CharDevice {
    fn init(&self);
    fn read(&self) -> u8;
    fn write(&self, ch: u8);
    fn handle_irq(&self);
}

pub const VIRT_UART: usize = 0x1000_0000;
type CharDeviceImpl = NS16550a<VIRT_UART>;

lazy_static! {
    pub static ref UART: Arc<CharDeviceImpl> = Arc::new(CharDeviceImpl::new());
}
