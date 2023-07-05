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

#[macro_export]
macro_rules! zero {
    ($addr:expr, $T:ty) => {
        unsafe {
            let ptr = $addr as *mut _ as *mut u8;
            let size = core::mem::size_of::<$T>();
            core::ptr::write_bytes(ptr, 0, size);
        }
    };
}
