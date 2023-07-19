use alloc::sync::Arc;
use component::{chardev::CharDevice, io::InputDevice};
use logger::info;

use crate::driver::{
    chardev::CharDeviceImpl,
    input::{VirtIOKeyBoard, VirtIOMouse},
};

// 输入有两种: 键盘和鼠标
lazy_static::lazy_static!(
    pub static ref KEYBOARD_DEVICE: Arc<dyn InputDevice> = {
        info!("Keyboard initializing ...");
         Arc::new(VirtIOKeyBoard::new())
    };

    pub static ref MOUSE_DEVICE: Arc<dyn InputDevice> = {
        info!("Mouse initializing ...");
        Arc::new(VirtIOMouse::new())
    };
);

lazy_static! {
    pub static ref UART: Arc<CharDeviceImpl> = {
        info!("UART initializing ...");
        Arc::new(CharDeviceImpl::new())
    };
}

pub fn init() {
    UART.init();
    KEYBOARD_DEVICE.as_ref();
    MOUSE_DEVICE.as_ref();
}
