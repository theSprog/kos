use crate::HandleIRQ;

pub trait NetDevice: Send + Sync + HandleIRQ + 'static {
    fn transmit(&self, data: &[u8]);
    fn receive(&self, data: &mut [u8]) -> usize;
}
