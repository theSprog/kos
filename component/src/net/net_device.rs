pub trait NetDevice: Send + Sync + 'static {
    fn transmit(&self, data: &[u8]);
    fn receive(&self, data: &mut [u8]) -> usize;
}
