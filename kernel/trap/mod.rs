use core::arch::global_asm;

use logger::{info, warn};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
    utvec::TrapMode,
};

use crate::{
    println,
    syscall::{self, syscall},
    timer::{get_time_ms, set_next_trigger},
};

use self::context::TrapContext;

pub mod context;

global_asm!(include_str!("trap.S"));

// 外部符号 __alltraps, 发生 tarp 后跳转到其中
extern "C" {
    pub fn __alltraps();
    pub fn __restore(ctx_addr: usize);

}

// 设置发生 trap 时的模式和地址, 自此以后我们就有用户态与内核态的区分了
pub fn init() {
    info!("Trap initalizing");

    unsafe {
        // 将 stvec 设置为 Direct 模式, 一旦发生 trap 总是陷入 __alltraps 地址
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// 处理中断或者系统调用
/// x10 是返回值
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        // 如果是来自用户态的 Syscall 调用(使用 ecall 指令)
        Trap::Exception(Exception::UserEnvCall) => {
            // 因为我们希望从 ecall 的下一条指令返回
            cx.sepc += 4;
            // x17: syscallID; x10-x12: 参数
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        // 如果是来自内存访问错误，包括低特权级访问高特权级寄存器
        Trap::Exception(Exception::StoreFault) => {
            warn!("PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            crate::task::exit_and_run_next();
        }
        Trap::Exception(Exception::StorePageFault) => {
            warn!("PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            crate::task::exit_and_run_next();
        }
        // 如果是来自非法指令, 例如用户态下 sret
        Trap::Exception(Exception::IllegalInstruction) => {
            warn!("IllegalInstruction in application, kernel killed it.");
            crate::task::exit_and_run_next();
        }

        // 处理 S 态的时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            crate::task::suspend_and_run_next();
        }

        _ => {
            panic!(
                "Temporarily unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}
