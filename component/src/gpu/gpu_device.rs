pub trait GpuDevice: Send + Sync + 'static {
    fn update_cursor(&self);
    fn get_framebuffer(&self) -> &mut [u8];
    fn flush(&self);
}
