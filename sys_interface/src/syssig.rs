use bitflags::bitflags;

pub const MAX_SIG: usize = 31;

pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGILL: i32 = 4;
pub const SIGTRAP: i32 = 5;
pub const SIGABRT: i32 = 6;
pub const SIGBUS: i32 = 7;
pub const SIGFPE: i32 = 8;
pub const SIGKILL: i32 = 9;
pub const SIGUSR1: i32 = 10;
pub const SIGSEGV: i32 = 11;
pub const SIGUSR2: i32 = 12;
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
pub const SIGTERM: i32 = 15;
pub const SIGSTKFLT: i32 = 16;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;
pub const SIGSTOP: i32 = 19;
pub const SIGTSTP: i32 = 20;
pub const SIGTTIN: i32 = 21;
pub const SIGTTOU: i32 = 22;
pub const SIGURG: i32 = 23;
pub const SIGXCPU: i32 = 24;
pub const SIGXFSZ: i32 = 25;
pub const SIGVTALRM: i32 = 26;
pub const SIGPROF: i32 = 27;
pub const SIGWINCH: i32 = 28;
pub const SIGIO: i32 = 29;
pub const SIGPWR: i32 = 30;
pub const SIGSYS: i32 = 31;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SignalFlags: u32 {
        const SIGDEF = 1 << 0; // Default signal handling
        const SIGHUP = 1 << 1;
        const SIGINT = 1 << 2;
        const SIGQUIT = 1 << 3;
        const SIGILL = 1 << 4;
        const SIGTRAP = 1 << 5;
        const SIGABRT = 1 << 6;
        const SIGBUS = 1 << 7;
        const SIGFPE = 1 << 8;
        const SIGKILL = 1 << 9;
        const SIGUSR1 = 1 << 10;    // 用户自定义信号 1
        const SIGSEGV = 1 << 11;
        const SIGUSR2 = 1 << 12;    // 用户自定义信号 2
        const SIGPIPE = 1 << 13;
        const SIGALRM = 1 << 14;
        const SIGTERM = 1 << 15;
        const SIGSTKFLT = 1 << 16;
        const SIGCHLD = 1 << 17;
        const SIGCONT = 1 << 18;
        const SIGSTOP = 1 << 19;
        const SIGTSTP = 1 << 20;
        const SIGTTIN = 1 << 21;
        const SIGTTOU = 1 << 22;
        const SIGURG = 1 << 23;
        const SIGXCPU = 1 << 24;
        const SIGXFSZ = 1 << 25;
        const SIGVTALRM = 1 << 26;
        const SIGPROF = 1 << 27;
        const SIGWINCH = 1 << 28;
        const SIGIO = 1 << 29;
        const SIGPWR = 1 << 30;
        const SIGSYS = 1 << 31;
    }
}

impl SignalFlags {
    pub fn check_error(&self) -> Option<(i32, &'static str)> {
        match self {
            _ if self.contains(Self::SIGINT) => Some((-2, "Killed, SIGINT=2")),
            _ if self.contains(Self::SIGILL) => Some((-4, "Illegal Instruction, SIGILL=4")),
            _ if self.contains(Self::SIGABRT) => Some((-6, "Aborted, SIGABRT=6")),
            _ if self.contains(Self::SIGFPE) => Some((-8, "Error Arithmetic Operation, SIGFPE=8")),
            _ if self.contains(Self::SIGKILL) => Some((-9, "Killed, SIGKILL=9")),
            _ if self.contains(Self::SIGSEGV) => Some((-11, "Segmentation Fault, SIGSEGV=11")),
            _ => None,
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct SignalAction {
    // 信号处理函数的地址
    pub handler: usize,
    // 处理信号时屏蔽所信号
    pub mask: SignalFlags,
}

impl Default for SignalAction {
    fn default() -> Self {
        Self {
            // 实际上可以注册 ctrl + C/D 退出进程的函数作为默认, 此处为了简化没有注册
            handler: 0,
            // SIGQUIT | SIGTRAP, 掩盖 trap 因为我们拒绝信号嵌套
            mask: SignalFlags::from_bits(40).unwrap(),
        }
    }
}
