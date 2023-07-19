use crate::driver::bus::VirtioHal;
use crate::sync::unicore::UPIntrFreeCell;
use alloc::{sync::Arc, vec::Vec};
use component::gpu::gpu_device::GpuDevice;
use core::{any::Any, ptr::NonNull};
use embedded_graphics::pixelcolor::Rgb888;
use tinybmp::Bmp;
use virtio_drivers::{
    device::gpu::VirtIOGpu,
    transport::mmio::{MmioTransport, VirtIOHeader},
};

const VIRTIO6: usize = 0x10007000;

pub struct VirtIOGPU {
    gpu: UPIntrFreeCell<VirtIOGpu<VirtioHal, MmioTransport>>,
    fb: &'static [u8],
}

static BMP_DATA: &[u8] = include_bytes!("../../bmp/folder.bmp");

impl VirtIOGPU {
    pub fn new() -> Self {
        unsafe {
            let header = VIRTIO6 as *mut VirtIOHeader;
            let transport = MmioTransport::new(NonNull::new(header).unwrap()).unwrap();

            let mut virtio = VirtIOGpu::<VirtioHal, MmioTransport>::new(transport).unwrap();

            let fbuffer = virtio.setup_framebuffer().unwrap();
            let len = fbuffer.len();
            let ptr = fbuffer.as_mut_ptr();
            let fb = core::slice::from_raw_parts_mut(ptr, len);

            let bmp = Bmp::<Rgb888>::from_slice(BMP_DATA).unwrap();
            let raw = bmp.as_raw();
            let mut b = Vec::new();
            for i in raw.image_data().chunks(3) {
                let mut v = i.to_vec();
                b.append(&mut v);
                if i == [255, 255, 255] {
                    b.push(0x0)
                } else {
                    b.push(0xff)
                }
            }
            virtio.setup_cursor(b.as_slice(), 50, 50, 50, 50).unwrap();

            Self {
                gpu: UPIntrFreeCell::new(virtio),
                fb,
            }
        }
    }
}

impl GpuDevice for VirtIOGPU {
    fn flush(&self) {
        self.gpu.exclusive_access().flush().unwrap();
    }
    fn get_framebuffer(&self) -> &mut [u8] {
        unsafe {
            let ptr = self.fb.as_ptr() as *const _ as *mut u8;
            core::slice::from_raw_parts_mut(ptr, self.fb.len())
        }
    }
    fn update_cursor(&self) {}
}
