use alloc::format;
use alloc::string::String;
use logger::error;

pub fn human_size(size: usize) -> String {
    const K: usize = 1024;
    const M: usize = K * K;
    const G: usize = M * K;

    if size < K {
        format!("{}B", size)
    } else if size < M {
        let kbs = size / K;
        let rest = size % K;
        if rest == 0 {
            format!("{}KiB", kbs)
        } else {
            format!("{}KiB+{}", kbs, human_size(rest))
        }
    } else if size < G {
        let mbs = size / M;
        let rest = size % M;
        if rest == 0 {
            format!("{}MiB", mbs)
        } else {
            format!("{}MiB+{}", mbs, human_size(rest))
        }
    } else {
        panic!("Too large size for {} Bytes", size)
    }
}
