use core::arch::{asm, global_asm};

use logger::{info, warn};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
    utvec::TrapMode,
};

use crate::{
    syscall::syscall,
    task::{current_trap_cx, current_user_token},
    timer::set_next_trigger,
    TRAMPOLINE, TRAP_CONTEXT,
};

pub mod context;

global_asm!(include_str!("trap.S"));

// 外部符号 __alltraps, 发生 tarp 后跳转到其中
extern "C" {
    pub fn __alltraps();
    pub fn __restore();
}

// 设置发生 trap 时的模式和地址, 自此以后我们就有用户态与内核态的区分了
pub fn init() {
    info!("Trap initalizing");
    set_kernel_trap_entry();
}

#[no_mangle]
#[repr(align(4096))]
pub fn trap_from_kernel() -> ! {
    panic!("A trap from kernel! How could be it ?");
}

/// 一旦进入内核后再次触发到 S态 Trap，
/// 则硬件在设置一些 CSR 寄存器之后，
/// 会跳过对通用寄存器的保存过程，
/// 直接跳转到 trap_from_kernel 函数，在这里直接 panic 退出
fn set_kernel_trap_entry() {
    unsafe {
        // 将 stvec 设置为 Direct 模式, 一旦发生 trap 总是陷入 __alltraps 地址
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_return() -> ! {
    // 一旦返回用户态，trap 就可以通过 TRAMPOLINE 返回
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();

    // 最后我们需要跳转到 __restore ，
    // 以执行：切换到应用地址空间、从 Trap 上下文中恢复通用寄存器、 sret 继续执行应用
    // 由于 __alltraps 是对齐到地址空间跳板页面的起始地址 TRAMPOLINE 上的，
    // 则 __restore 的虚拟地址只需在 TRAMPOLINE 基础上加上 __restore 相对于 __alltraps 的偏移量即可
    let restore_va = TRAMPOLINE + __restore as usize - __alltraps as usize;
    unsafe {
        asm!(
            // 清空指令缓存 i-cache
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

/// 处理中断或者系统调用
/// x10 是返回值
#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    // 调用 current_trap_cx 来获取当前应用的 Trap 上下文的可变引用而不是像之前那样作为参数传入 trap_handler
    let cx = current_trap_cx();
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

    trap_return();
}
