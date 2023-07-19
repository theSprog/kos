use alloc::sync::Arc;
use component::net::NetDevice;
use logger::info;

use crate::driver::net::NetDeviceImpl;

lazy_static! {
    pub static ref NET_DEVICE: Arc<dyn NetDevice> = {
        info!("NET_DEVICE initializing...");
        Arc::new(NetDeviceImpl::new())
    };
}

pub fn init() {
    NET_DEVICE.as_ref();
}
