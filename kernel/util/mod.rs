use alloc::format;
use alloc::string::String;

pub fn human_size(size: usize) -> String {
    const K: usize = 1024;
    const M: usize = K * K;
    const G: usize = M * K;

    if size < K {
        format!("{} B", size)
    } else if size < M {
        format!("{} KB", size / K)
    } else if size < G {
        format!("{} MB", size / M)
    } else {
        format!("{} GB", size / G)
    }
}
