use crate::HandleIRQ;

pub trait InputDevice: Send + Sync + HandleIRQ + 'static {
    fn read_event(&self) -> u64;
    fn is_empty(&self) -> bool;
}
