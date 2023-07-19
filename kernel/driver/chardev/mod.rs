mod ns16550a;

use alloc::sync::Arc;
use lazy_static::*;

use ns16550a::NS16550a;

pub const VIRT_UART: usize = 0x10000000;
pub type CharDeviceImpl = NS16550a<VIRT_UART>;
