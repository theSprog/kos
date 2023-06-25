#[macro_export]
macro_rules! cast {
    ($addr:expr, $T:ty) => {
        unsafe { &*($addr as *const $T) }
    };
}

#[macro_export]
macro_rules! cast_mut {
    ($addr:expr, $T:ty) => {
        unsafe { &mut *($addr as *mut $T) }
    };
}
