use sys_interface::syssig::{SignalAction, MAX_SIG};

#[derive(Clone)]
pub struct SignalActions {
    pub table: [SignalAction; MAX_SIG + 1],
}

impl Default for SignalActions {
    fn default() -> Self {
        Self {
            table: [SignalAction::default(); MAX_SIG + 1],
        }
    }
}

pub mod api {
    use super::*;
    use crate::process::processor::{self};
    use logger::info;
    use sys_interface::syssig::{SignalFlags, MAX_SIG};

    // 处理尚未处理的信号, 此处才是真正的信号处理逻辑
    fn handle_pending_signals() {
        // 遍历所有信号
        for sig in 0..MAX_SIG {
            let pcb = processor::api::current_pcb().unwrap();
            let inner = pcb.ex_inner();
            let signal = SignalFlags::from_bits(1 << sig).unwrap();

            // 如果该信号未处理并且也未被掩盖
            if inner.pending_signals().contains(signal) && (!inner.signal_mask().contains(signal)) {
                let mut masked = true;
                let handling_sig = inner.handling_sig();

                // 有可能一个信号被另一个信号处理函数屏蔽
                if handling_sig == -1 {
                    masked = false;
                } else {
                    let handling_sig = handling_sig as usize;
                    if !inner.signal_actions().table[handling_sig]
                        .mask
                        .contains(signal)
                    {
                        masked = false;
                    }
                }

                if !masked {
                    drop(inner);
                    drop(pcb);
                    match signal {
                        SignalFlags::SIGKILL
                        | SignalFlags::SIGSTOP
                        | SignalFlags::SIGCONT
                        | SignalFlags::SIGDEF => {
                            // 内核信号
                            kernel_signal_handler(signal);
                        }
                        _ => {
                            // 用户信号
                            user_signal_handler(sig, signal);
                            return;
                        }
                    }
                }
            }
        }
    }

    fn kernel_signal_handler(signal: SignalFlags) {
        let pcb = processor::api::current_pcb().unwrap();
        let mut inner = pcb.ex_inner();
        match signal {
            SignalFlags::SIGSTOP => {
                inner.frozen = true;
                inner.pending_signals ^= SignalFlags::SIGSTOP;
            }
            SignalFlags::SIGCONT => {
                if inner.pending_signals.contains(SignalFlags::SIGCONT) {
                    inner.pending_signals ^= SignalFlags::SIGCONT;
                    inner.frozen = false;
                }
            }
            _ => {
                info!(
                    "kernel_signal_handler:: current task sigflag {:?}",
                    inner.pending_signals
                );
                inner.killed = true;
            }
        }
    }

    fn user_signal_handler(sig: usize, signal: SignalFlags) {
        let pcb = processor::api::current_pcb().unwrap();
        let mut inner = pcb.ex_inner();
        // 找到信号处理函数
        let handler = inner.signal_actions.table[sig].handler;

        // 如果用户设置了 handler, 则调用用户的 handler
        if handler != 0 {
            // 设置正在进行的信号处理函数
            inner.handling_sig = sig as isize;
            // 在未处理信号中移除该信号
            inner.pending_signals ^= signal;

            // 备份 trap
            let trap_ctx = inner.trap_cx();
            inner.trap_ctx_backup = Some(trap_ctx.clone());

            // 将返回地址设置为信号处理函数地址
            // 我们并没有修改 Trap 上下文中的 sp, 这意味着例程还会在原先的用户栈上执行
            // 在 Linux 的实现中，内核会为每次例程的执行重新分配一个用户栈
            trap_ctx.sepc = handler;

            // 放置参数, 使得信号类型能够作为参数被例程接收(a0 即第一个参数)
            trap_ctx.x[10] = sig;
        } else {
            // 否则未设置信号处理函数, 仅仅打印日志记录
            info!("user_signal_handler: default action: ignore it or kill process");
        }
    }

    pub fn handle_signals() {
        // 这个循环的意义在于：
        // 只要进程还处于暂停且未被杀死的状态就会停留在循环中等待 SIGCONT 信号的到来
        loop {
            handle_pending_signals();
            // 检查是否被 STOP 或者 KILL
            let (frozen, killed) = {
                let pcb = processor::api::current_pcb().unwrap();
                let inner = pcb.ex_inner();
                (inner.frozen(), inner.killed())
            };
            if !frozen || killed {
                break;
            }
            processor::api::suspend_and_run_next();
        }
    }

    pub fn check_signals_error() -> Option<(i32, &'static str)> {
        let pcb = processor::api::current_pcb().unwrap();
        let inner = pcb.ex_inner();
        inner.pending_signals().check_error()
    }
}
