#[macro_export]
macro_rules! ceil_index {
    ($index:expr, $size:expr) => {
        ($index + $size - 1) / $size
    };
}

#[macro_export]
macro_rules! ceil {
    ($index:expr, $bound:expr) => {
        (($index + $bound - 1) / $bound) * $bound
    };
}