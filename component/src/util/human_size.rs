use alloc::format;
use alloc::string::String;

/// 分配内存版本, 在全局内存分配器前不可用
/// 使用 human_size_n 代替
pub fn human_size(size: usize) -> String {
    if size < KB {
        format!("{}B", size)
    } else if size < MB {
        let kbs = size / KB;
        let rest = size % KB;
        if rest == 0 {
            format!("{}KB", kbs)
        } else {
            format!("{}KB+{}", kbs, human_size(rest))
        }
    } else {
        let mbs = size / MB;
        let rest = size % MB;
        if rest == 0 {
            format!("{}MB", mbs)
        } else {
            format!("{}MB+{}", mbs, human_size(rest))
        }
    }
}

// 不分配内存版本
use core::sync::atomic::{AtomicUsize, Ordering};
use sys_interface::config::{KB, MB};

/// 之前的版本使用单一 SLOT: [u8; 32], 但这有个坏处:
/// 多次调用 human_size 会改变前面的返回值的内容, 因此才需要使用 SLOTS 将之前的内容缓存起来
/// 在 info!("{}", human_size(1)); 不会有影响
/// 但是在 info!("{} {}", human_size(1), human_size(2)); 后面的结果会把前面的结果覆盖
/// 这是因为之前的版本共用同一个 SLOT 的缘故
static mut SLOTS: [[u8; 32]; SLOT_COUNT] = [[0; 32]; SLOT_COUNT];

/// 由于多线程操作可能取到同一个 slot, 因此使用原子操作
static SLOT_IDX: AtomicUsize = AtomicUsize::new(0);

/// 严格来说, 当有多个线程使用 SLOTS 而 SLOT_COUNT 又不够大时
/// 仍然有可能出现数据竞争, 多个线程同时由于 SLOT_IDX 回绕从而操作同一个 SLOT
/// 甚至于当格式化中使用 `{}` 同时调用 human_size 过多时也会出现这个情况
const SLOT_COUNT: usize = 16;

/// 这个函数很拧巴, 它为了不分配内存使用了大量丑陋的操作
/// 但特点也很明显, 该函数不分配内存, 有点类似于一个 C 函数了
#[allow(non_snake_case)]
pub fn human_size_n(size: usize) -> &'static str {
    const SUFFIXES: [&str; 3] = ["MB", "KB", "B"];
    const BOUNDS: [usize; 3] = [MB, KB, 1];

    let cur_slot_idx = SLOT_IDX.load(Ordering::SeqCst);
    // 这是一个静态变量(引用 SLOTS)，故大写
    let mut BUFFER = unsafe { &mut SLOTS[cur_slot_idx] };
    clear_buffer(&mut BUFFER);

    let mut cur_size = size;
    let mut suffix_index = 0;
    let mut cur_idx = 0;

    for bound in BOUNDS {
        let aligned = cur_size - cur_size % bound;
        let content = aligned / bound;
        cur_size = cur_size - aligned;

        // content == 0 没有必要写, 除非是传入的参数本就为 0, 即 0 B
        if content != 0 || suffix_index == SUFFIXES.len() - 1 {
            cur_idx = write_size(&mut BUFFER, cur_idx, content, SUFFIXES[suffix_index]);

            if cur_size == 0 {
                break;
            }

            BUFFER[cur_idx] = b'+';
            cur_idx += 1;
        }

        suffix_index += 1;
    }

    unsafe {
        // 到下一个槽位
        SLOT_IDX.fetch_add(1, Ordering::SeqCst); // 先自增
        let new_slot_idx = SLOT_IDX.load(Ordering::SeqCst) % SLOT_COUNT;
        SLOT_IDX.store(new_slot_idx, Ordering::SeqCst);
        core::str::from_utf8_unchecked_mut(BUFFER)
    }
}

fn clear_buffer(buffer: &mut [u8; 32]) {
    for i in 0..buffer.len() {
        buffer[i] = 0;
    }
}

fn write_size(buffer: &mut [u8; 32], mut from: usize, content: usize, suffix: &str) -> usize {
    let mut temp = [0u8; 10];
    let mut idx = 0;

    // 逐个将个位取出, 放入 temp 中
    // 这里断言 content 内容不会太大, temp 足以容纳
    if content == 0 {
        temp[idx] = b'0';
        idx += 1;
    } else {
        let mut cur_content = content;
        while cur_content != 0 {
            temp[idx] = (cur_content % 10) as u8 + b'0';

            cur_content /= 10;
            idx += 1;
        }
    }

    for i in (0..idx).rev() {
        buffer[from] = temp[i];
        from += 1;
    }

    for c in suffix.bytes() {
        buffer[from] = c;
        from += 1;
    }

    from
}
