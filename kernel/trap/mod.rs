use core::arch::{asm, global_asm};

use logger::*;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
    utvec::TrapMode, sepc, sscratch,
};
use sys_interface::{syscall::SYSCALL_EXECVE, syssig::SignalFlags, syserr};

use crate::{
    memory::{address::*,  segment},
    process::{processor::{
        self,
        api::{current_cmd_name, current_pid},
    }, signal},
    syscall::syscall,
    clock::set_next_trigger,
    TRAMPOLINE,
};

pub mod context;
mod plic;

global_asm!(include_str!("trap.S"));

// 外部符号 __alltraps, 发生 tarp 后跳转到其中
extern "C" {
    pub fn __alltraps();
    pub fn __restore();
}

/// 设置发生 trap 时的模式和地址, 自此以后我们就有用户态与内核态的区分了
pub fn init() {
    info!("Trap initalizing");
    set_kernel_trap_entry();
}

/// 弱化 S态 –> S态的 Trap 处理过程
#[no_mangle]
#[repr(align(4096))]
pub fn trap_from_kernel() -> ! {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            // crate::board::irq_handler();
            todo!()
        },
        // Trap::Interrupt(Interrupt::SupervisorTimer) => {
        //     set_next_trigger();
        //     check_timer();
        //     // do not schedule now
        // }

        _ => {
            error!("stval = {:#x}, sepc = {:#x}", stval::read(), sepc::read());
            panic!("An unhandle trap from kernel! How could be it ?");
        }
    }
}

/// 一旦进入内核后再次触发到 S态 Trap，
/// 则硬件在设置一些 CSR 寄存器之后，
/// 会跳过对通用寄存器的保存过程，
/// 直接跳转到 trap_from_kernel 函数，在那里直接 panic 退出
fn set_kernel_trap_entry() {
    // extern "C" {
    //     fn __alltraps();
    //     fn __alltraps_k();
    // }
    // let __kernel_trap = TRAMPOLINE + (__alltraps_k as usize - __alltraps as usize);
    // unsafe {
    //     stvec::write(__kernel_trap, TrapMode::Direct);
    //     sscratch::write(trap_from_kernel as usize);
    // }

    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        // 将 stvec 设置为 Direct 模式, 一旦发生 trap 总是陷入 TRAMPOLINE 地址
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_return() -> ! {
    // 一旦返回用户态，trap 就可以通过 TRAMPOLINE 陷入内核
    set_user_trap_entry();
    let trap_cx_ptr = processor::api::current_trap_ctx_uptr();
    // 拿回用户页表
    let user_satp = processor::api::current_user_token();

    // 最后我们需要跳转到 __restore ，以执行：
    // 切换到应用地址空间、从 Trap 上下文中恢复通用寄存器、 sret 继续执行应用
    // 由于 __alltraps 是对齐到地址空间跳板页面的起始地址 TRAMPOLINE 上的，
    // 则 __restore 的虚拟地址只需在 TRAMPOLINE 基础上加上 __restore 相对于 __alltraps 的偏移量即可
    let restore_va = TRAMPOLINE + (__restore as usize - __alltraps as usize);
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
/// trap handler 只有在内核地址空间中才能访问
#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    // 调用 current_trap_cx 来获取当前应用的 Trap 上下文的可变引用而不是像之前那样作为参数传入 trap_handler
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    let cx = processor::api::current_trap_ctx();
    match scause.cause() {
        // 如果是来自用户态的 Syscall 调用(使用 ecall 指令)
        Trap::Exception(Exception::UserEnvCall) => {
            // 因为我们希望从 ecall 的下一条指令返回
            cx.sepc += 4;

            // x17: syscallID; 
            // x10-x12: 参数
            if cx.x[17] == SYSCALL_EXECVE {
                // exec 会将当前地址空间替换为新的地址空间
                // 这其中也包括 trap 的 ppn 也被替换，因此在 syscall 前后 trap ppn 不同
                let old_cx = cx;

                let ret = syscall(old_cx.x[17], [old_cx.x[10], old_cx.x[11], old_cx.x[12]]);

                let now_cx = processor::api::current_trap_ctx();
                now_cx.x[10] = ret as usize;
            }else {
                cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            }
        }

        // 处理 S 态的时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 设置好下一次时钟中断
            set_next_trigger();
            // 切换任务
            processor::api::suspend_and_run_next();
        }

        // 内存访问错误，类似写入只读区域, 包括低特权级访问高特权级寄存器
        Trap::Exception(Exception::LoadFault) => {
            warn!("LoadFault in application-'{}'(pid={}), bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", current_cmd_name(),current_pid(), stval, cx.sepc);
            // processor::api::exit_and_run_next(-2);
            processor::api::current_add_signal(SignalFlags::SIGSEGV);
        }

        Trap::Exception(Exception::StoreFault) => {
            warn!("StoreFault in application-'{}'(pid={}), bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", current_cmd_name(), current_pid(), stval, cx.sepc);
            // processor::api::exit_and_run_next(-2);
            processor::api::current_add_signal(SignalFlags::SIGSEGV);
        }

        // 如果是来自非法指令, 例如用户态下 sret
        Trap::Exception(Exception::IllegalInstruction) => {
            warn!(
                "IllegalInstruction in application-'{}'(pid={}), stval:{}, cx.sepc: {:#X?}. kernel killed it.",
                current_cmd_name(),current_pid(),
                stval,
                cx.sepc
            );
            // processor::api::exit_and_run_next(-3);
            processor::api::current_add_signal(SignalFlags::SIGILL);
        }

        // 以下是三个缺页异常
        // 写数据缺页
        Trap::Exception(Exception::StorePageFault) => {
            trace!(
                "StorePageFault from application-'{}'(pid={})",
                current_cmd_name(),
                current_pid()
            );
            let address_space = processor::api::current_ex_address_space();
            // 先判断是不是缺页, 还是说真的是 page_fault
            if address_space
                .is_page_fault(stval, segment::MapPermission::W)
            {
                // 是否是 copy on write 
                let vpn = VirtAddr::from(stval).floor();
                let user_page_table = address_space.page_table();
                let pte = user_page_table.translate(vpn);

                // 页存在且有物理页映射, 但是不可写
                if pte.is_some() && pte.as_ref().unwrap().valid() && !pte.as_ref().unwrap().writable() {
                    // segment 可写, 但是 pte 表示不可写, 说明是 cow
                    trace!("Fixing COW for this StorePageFault");
                    address_space.fix_cow(stval);
                } else {
                    // 物理页不存在, 缺页只有可能发生在堆栈段
                    assert!(pte.is_none() || !pte.unwrap().valid());
                    address_space.fix_page_missing(stval);
                }
            } else {
                warn!("PageFault in application: bad 'store' addr = {:#x} for instruction (addr = {:#x})", stval, cx.sepc);
                // processor::api::exit_and_run_next(-2);
                processor::api::current_add_signal(SignalFlags::SIGSEGV);
            }
        }

        // 读数据缺页, 例如 bss 可能会直接 load
        Trap::Exception(Exception::LoadPageFault) => {
            trace!(
                "LoadPageFault from application-'{}'(pid={})",
                current_cmd_name(),
                current_pid()
            );

            let address_space = processor::api::current_ex_address_space();
            if address_space
                .is_page_fault(stval, segment::MapPermission::R)
            {
                address_space.fix_page_missing(stval);
            } else {
                warn!("PageFault in application: bad 'read' addr = {:#x} for instruction (addr= {:#x})", stval, cx.sepc);
                // processor::api::exit_and_run_next(-2);
                processor::api::current_add_signal(SignalFlags::SIGSEGV);
            }
        }

        // 执行指令缺页, 由于所有可执行代码段全部加载, 所以不可能出现缺页
        Trap::Exception(Exception::InstructionPageFault) => {
            trace!(
                "InstructionPageFault from app-'{}'(pid={})",
                current_cmd_name(),
                current_pid()
            );

            warn!("PageFault in app-'{}'(pid={}): bad 'execute' instruction = {:#x}(addr= {:#x}) for there is unexecutable. kernel killed it.", 
            current_cmd_name(), 
            current_pid(),
            stval, cx.sepc);
            // processor::api::exit_and_run_next(-3);
            processor::api::current_add_signal(SignalFlags::SIGSEGV);
        }

        _ => {
            panic!(
                "Temporarily unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }

    // 返回时处理信号
    signal::api::handle_signals();

    // 检查是否有错, 若有错(例如段错误)则退出
    if let Some((exit_code, msg)) = signal::api::check_signals_error() {
        info!("tid: {}, msg: {}, exit_code: {}", processor::api::current_tid(), msg,  exit_code);
        processor::api::exit_and_run_next(syserr::EINTR as i32);
    }

    // 返回用户地址空间
    trap_return();
}
