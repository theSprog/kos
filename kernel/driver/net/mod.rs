mod virtio_net;

use component::net::net_device;

pub type NetDeviceImpl = virtio_net::VirtIONetwork;
