use crate::sbi::set_timer;
use alloc::sync::Arc;
use component::device_tree::Node;
use logger::info;
use qemu_config::*;

// 每秒执行多少次中断
const INTERRUPT_PER_SEC: usize = 100;

// 时间片长度, 每秒 100 次中断, 每个时间片大概 10 ms
const TIME_INTERVAL: usize = CLOCK_FREQ / INTERRUPT_PER_SEC;

#[repr(C)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
    pub msec: usize,
}

// 取得当前 mtime 计数器
// mtime 是一个64位技术器, 用来统计处理器自上电以来经过了多少个内置时钟的时钟周期
pub fn get_cycle() -> usize {
    riscv::register::time::read()
}

// 以微秒形式获取时间
pub fn get_time_us() -> usize {
    get_cycle() / MICRO_UNIT
}

// 以毫秒形式获取时间
pub fn get_time_ms() -> usize {
    get_cycle() / MILLI_UNIT
}

// 以秒形式获取时间
pub fn get_time_s() -> usize {
    get_cycle() / SECOND_UNIT
}

pub fn set_next_trigger() {
    // 每秒时钟中断次数
    set_timer(get_cycle() + TIME_INTERVAL);
}

/// 如果中断的特权级低于 CPU 当前的特权级，则该中断会被屏蔽，不会被处理；
/// 例如如果当前 CPU 处于 S 态, 则 U 态的时钟中断会被忽略
///
/// 如果中断的特权级高于或等于与 CPU 当前的特权级，则需要通过相应的 CSR 判断该中断是否会被屏蔽。
/// sstatus.sie 管理所有中断屏蔽与否,
/// ssie/stie/seie 分别控制 S 特权级的软件中断、时钟中断和外部中断的屏蔽与否
///
/// 默认情况下，当中断产生并进入某个特权级之后，在中断处理的过程中同特权级的中断都会被屏蔽。
/// 换句话说同特权级中断没有中断嵌套
pub(crate) fn init() {
    unsafe {
        info!("Time-Sharing mechanism initalizing");
        // 此前 S 态由于关时钟中断所以不会响应时钟
        // 开启 S 态时钟中断,
        riscv::register::sie::set_stimer();
        // 此后 S 态开始响应时钟中断
        set_next_trigger();
    }
}

// 时钟端口
// const TIMER_TIME_LOW: usize = 0x00;
// const TIMER_TIME_HIGH: usize = 0x04;
// pub struct RtcGoldfish {
//     base: usize,
// }

// impl RtcGoldfish {
//     // read seconds since 1970-01-01
//     fn read_epoch(&self) -> u64 {
//         let ns = unsafe {
//             let low: u32 = core::ptr::read_volatile((self.base + TIMER_TIME_LOW) as *const _);
//             let high: u32 = core::ptr::read_volatile((self.base + TIMER_TIME_HIGH) as *const _);
//             ((high as u64) << 32) | (low as u64)
//         };

//         ns / 1_000_000_000u64
//     }
// }

// pub const PHYSICAL_MEMORY_OFFSET: usize = 0xFFFF_FFFF_4000_0000;
// fn init_dt(dt: &Node) {
//     let addr = dt.prop_u64("reg").unwrap() as usize;
//     RTC_DRIVERS.write().push(Arc::new(RtcGoldfish {
//         base: PHYSICAL_MEMORY_OFFSET + paddr,
//     }));
// }

// pub fn driver_init() {
//     DEVICE_TREE_REGISTRY
//         .write()
//         .insert("google,goldfish-rtc", init_dt);
// }
