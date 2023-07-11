use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

// 目前我们内核仅运行在单核上，因此无需在意任何多核引发的数据竞争/同步问题
// 因此我们向编译器保证 UPSafeCell 是 sync 的
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// 用户需要负责该变量只能在单线程内使用
    pub(crate) unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// 以可变借用形式访问
    /// 由于是 borrow_mut 所以相比原生的 RefCell 它不再允许多个读操作同时存在
    pub(crate) fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
