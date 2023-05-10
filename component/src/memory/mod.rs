pub mod buddy;

// 所有的分配器都应该实现这两点
pub trait IAllocator<T> {
    fn empty() -> T;
    fn init(&mut self, start: usize, size: usize);
}
