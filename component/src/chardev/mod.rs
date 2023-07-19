use crate::HandleIRQ;

pub trait CharDevice: Send + Sync + HandleIRQ + 'static {
    fn init(&self);
    fn read(&self) -> u8;
    fn write(&self, ch: u8);
}
