use crate::driver::gpu::VirtIOGPU;
use alloc::sync::Arc;
use component::gpu::gpu_device::GpuDevice;
use logger::info;

lazy_static::lazy_static!(
    pub static ref GPU_DEVICE: Arc<dyn GpuDevice> = {
        info!("GPU_DEVICE initializing...");
        Arc::new(VirtIOGPU::new())
    };
);

pub fn init() {
    GPU_DEVICE.as_ref();
}
