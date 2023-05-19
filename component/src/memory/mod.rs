pub mod bitmap;
pub mod buddy;
pub mod slab;

// 所有的分配器都应该实现这几点
pub trait IAllocator<T> {
    fn empty() -> T;
    fn init(&mut self, start: usize, size: usize);
}
