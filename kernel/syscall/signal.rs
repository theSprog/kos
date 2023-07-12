use crate::{memory::page_table, process::processor};
use logger::*;
use sys_interface::{syserr, syssig::*};

fn check_sigaction_error(signal: SignalFlags, action: usize, old_action: usize) -> bool {
    // 如果传入的 action 或者 old_action 为空指针则视为错误。
    if action == 0
        || old_action == 0
        // 另一种错误则是信号类型为 SIGKILL 或者 SIGSTOP
        // 参考 Linux 内核规定不允许进程对这两种信号设置信号处理例程，而只能由内核对它们进行处理
        || signal == SignalFlags::SIGKILL
        || signal == SignalFlags::SIGSTOP
    {
        true
    } else {
        false
    }
}

//为当前进程设置某种信号的处理函数，同时保存设置之前的处理函数
pub fn sys_sigaction(
    signal: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    let token = processor::api::current_user_token();
    let pcb = processor::api::current_pcb();
    let mut inner = pcb.ex_inner();
    if signal as usize > MAX_SIG {
        return syserr::EINVAL;
    }
    if let Some(flag) = SignalFlags::from_bits(1 << signal) {
        if check_sigaction_error(flag, action as usize, old_action as usize) {
            return syserr::EINVAL;
        }

        // 将当前 action 保存至 old_action 中
        let prev_action = inner.signal_actions().table[signal as usize];
        let old_action_slot = page_table::api::translated_refmut(token, old_action);
        *old_action_slot = prev_action;

        // 设置好新处理函数
        inner.signal_actions_mut().table[signal as usize] =
            *page_table::api::translated_ref(token, action);

        0
    } else {
        syserr::EINVAL
    }
}

// 进程可以通过 sigprocmask 系统调用直接设置自身的全局信号掩码
pub fn sys_sigprocmask(mask: u32) -> isize {
    let pcb = processor::api::current_pcb();
    let mut inner = pcb.ex_inner();
    let old_mask = inner.signal_mask().bits();
    if let Some(flag) = SignalFlags::from_bits(mask) {
        // 设置信号掩码
        inner.set_signal_mask(flag);
        old_mask as isize
    } else {
        // 非法参数
        syserr::EINVAL
    }
}

pub fn sys_kill(pid: usize, signal: i32) -> isize {
    // 获取目标 pcb
    if let Some(pcb) = processor::api::pid2pcb(pid) {
        if let Some(flag) = SignalFlags::from_bits(1 << signal) {
            let mut inner = pcb.ex_inner();
            // 如果目标进程还有该信号未处理
            if inner.pending_signals().contains(flag) {
                return syserr::EINVAL;
            }
            // 否则插入该信号
            inner.pending_signals_mut().insert(flag);

            0
        } else {
            syserr::EINVAL
        }
    } else {
        // 目标进程不存在
        syserr::ESRCH
    }
}

pub fn sys_sigreturn() -> isize {
    let pcb = processor::api::current_pcb();
    let mut inner = pcb.ex_inner();
    inner.set_handling_sig(-1);
    // restore the trap context
    info!("restore up trap");
    let trap_ctx = processor::api::current_trap_ctx();
    *trap_ctx = inner.trap_ctx_backup().unwrap();
    trap_ctx.x[10] as isize
}
