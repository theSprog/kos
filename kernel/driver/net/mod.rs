use core::ptr::NonNull;

use crate::driver::bus::VirtioHal;
use crate::sync::unicore::UPIntrFreeCell;
use alloc::string::String;
use alloc::vec::Vec;
use component::net::NetDevice;
use component::HandleIRQ;
use logger::*;
use virtio_drivers::device::net::{TxBuffer, VirtIONet};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::Transport;

pub type NetDeviceImpl = VirtIONetwork;

const VIRTIO3: usize = 0x10004000;
const NET_BUF_LEN: usize = 8192;
const NET_QUEUE_SIZE: usize = 16;

pub struct VirtIONetwork {
    virtio_net: UPIntrFreeCell<VirtIONet<VirtioHal, MmioTransport, NET_QUEUE_SIZE>>,
}

impl VirtIONetwork {
    pub fn new() -> Self {
        unsafe {
            let header = VIRTIO3 as *mut VirtIOHeader;
            let mut transport: MmioTransport =
                MmioTransport::new(NonNull::new(header).unwrap()).unwrap();

            debug!("net max send_queue size: {}", transport.max_queue_size(0));
            debug!("net max recv_queue size: {}", transport.max_queue_size(1));

            let virtio_network = Self {
                virtio_net: UPIntrFreeCell::new(VirtIONet::new(transport, NET_BUF_LEN).unwrap()),
            };

            // net_test(&virtio_network);

            virtio_network
        }
    }
}

fn net_test(net: &VirtIONetwork) {
    let mut net = net.virtio_net.exclusive_access();
    let mac = net.mac_address();
    let mac = mac
        .iter()
        .map(|byte| alloc::format!("{:02X}", byte))
        .collect::<Vec<String>>()
        .join(":");
    info!("MAC address: {}", mac);

    loop {
        match net.receive() {
            Ok(buf) => {
                info!("RECV {} bytes: {:02x?}", buf.packet_len(), buf.packet());
                let tx_buf = TxBuffer::from(buf.packet());
                net.send(tx_buf).expect("failed to send");
                net.recycle_rx_buffer(buf).unwrap();
                break;
            }
            // 没有数据
            Err(virtio_drivers::Error::NotReady) => continue,
            // 其他错误
            Err(err) => panic!("failed to recv: {:?}", err),
        }
    }
    info!("virtio-net test finished");
}

impl HandleIRQ for VirtIONetwork {
    fn handle_irq(&self) {
        todo!()
    }
}

impl NetDevice for VirtIONetwork {
    fn transmit(&self, data: &[u8]) {
        self.virtio_net
            .exclusive_access()
            .send(TxBuffer::from(data))
            .expect("can't send data")
    }

    fn receive(&self, data: &mut [u8]) -> usize {
        let rx_buf = self
            .virtio_net
            .exclusive_access()
            .receive()
            .expect("can't receive data");
        let len = rx_buf.packet_len();
        let slice = &mut data[..len];
        slice.copy_from_slice(rx_buf.packet());
        len
    }
}
