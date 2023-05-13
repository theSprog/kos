#![no_main]
#![no_std]

/// 时钟频率, 机器每秒执行 CLOCK_FREQ 这么多 cycle
/// 因此 CLOCK_FREQ 可以理解为一秒
pub const CLOCK_FREQ: usize = 10000000;

// 微秒单位
pub const MICRO_UNIT: usize = CLOCK_FREQ / 1_000_000;
// 毫秒单位
pub const MILLI_UNIT: usize = CLOCK_FREQ / 1_000;
// 秒单位
pub const SECOND_UNIT: usize = CLOCK_FREQ;
