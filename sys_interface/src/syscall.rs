pub const SYSCALL_IO_SETUP: usize = 0x0;
pub const SYSCALL_IO_DESTROY: usize = 0x1;
pub const SYSCALL_IO_SUBMIT: usize = 0x2;
pub const SYSCALL_IO_CANCEL: usize = 0x3;
pub const SYSCALL_IO_GETEVENTS: usize = 0x4;
pub const SYSCALL_SETXATTR: usize = 0x5;
pub const SYSCALL_LSETXATTR: usize = 0x6;
pub const SYSCALL_FSETXATTR: usize = 0x7;
pub const SYSCALL_GETXATTR: usize = 0x8;
pub const SYSCALL_LGETXATTR: usize = 0x9;
pub const SYSCALL_FGETXATTR: usize = 0xa;
pub const SYSCALL_LISTXATTR: usize = 0xb;
pub const SYSCALL_LLISTXATTR: usize = 0xc;
pub const SYSCALL_FLISTXATTR: usize = 0xd;
pub const SYSCALL_REMOVEXATTR: usize = 0xe;
pub const SYSCALL_LREMOVEXATTR: usize = 0xf;
pub const SYSCALL_FREMOVEXATTR: usize = 0x10;
pub const SYSCALL_GETCWD: usize = 0x11;
pub const SYSCALL_LOOKUP_DCOOKIE: usize = 0x12;
pub const SYSCALL_EVENTFD2: usize = 0x13;
pub const SYSCALL_EPOLL_CREATE1: usize = 0x14;
pub const SYSCALL_EPOLL_CTL: usize = 0x15;
pub const SYSCALL_EPOLL_PWAIT: usize = 0x16;
pub const SYSCALL_DUP: usize = 0x17;
pub const SYSCALL_DUP3: usize = 0x18;
pub const SYSCALL_FCNTL: usize = 0x19;
pub const SYSCALL_INOTIFY_INIT1: usize = 0x1a;
pub const SYSCALL_INOTIFY_ADD_WATCH: usize = 0x1b;
pub const SYSCALL_INOTIFY_RM_WATCH: usize = 0x1c;
pub const SYSCALL_IOCTL: usize = 0x1d;
pub const SYSCALL_IOPRIO_SET: usize = 0x1e;
pub const SYSCALL_IOPRIO_GET: usize = 0x1f;
pub const SYSCALL_FLOCK: usize = 0x20;
pub const SYSCALL_MKNODAT: usize = 0x21;
pub const SYSCALL_MKDIRAT: usize = 0x22;
pub const SYSCALL_UNLINKAT: usize = 0x23;
pub const SYSCALL_SYMLINKAT: usize = 0x24;
pub const SYSCALL_LINKAT: usize = 0x25;
pub const SYSCALL_UMOUNT2: usize = 0x27;
pub const SYSCALL_MOUNT: usize = 0x28;
pub const SYSCALL_PIVOT_ROOT: usize = 0x29;
pub const SYSCALL_NFSSERVCTL: usize = 0x2a;
pub const SYSCALL_STATFS: usize = 0x2b;
pub const SYSCALL_FSTATFS: usize = 0x2c;
pub const SYSCALL_TRUNCATE: usize = 0x2d;
pub const SYSCALL_FTRUNCATE: usize = 0x2e;
pub const SYSCALL_FALLOCATE: usize = 0x2f;
pub const SYSCALL_FACCESSAT: usize = 0x30;
pub const SYSCALL_CHDIR: usize = 0x31;
pub const SYSCALL_FCHDIR: usize = 0x32;
pub const SYSCALL_CHROOT: usize = 0x33;
pub const SYSCALL_FCHMOD: usize = 0x34;
pub const SYSCALL_FCHMODAT: usize = 0x35;
pub const SYSCALL_FCHOWNAT: usize = 0x36;
pub const SYSCALL_FCHOWN: usize = 0x37;
pub const SYSCALL_OPENAT: usize = 0x38;
pub const SYSCALL_CLOSE: usize = 0x39;
pub const SYSCALL_VHANGUP: usize = 0x3a;
pub const SYSCALL_PIPE2: usize = 0x3b;
pub const SYSCALL_QUOTACTL: usize = 0x3c;
pub const SYSCALL_GETDENTS64: usize = 0x3d;
pub const SYSCALL_LSEEK: usize = 0x3e;
pub const SYSCALL_READ: usize = 0x3f;
pub const SYSCALL_WRITE: usize = 0x40;
pub const SYSCALL_READV: usize = 0x41;
pub const SYSCALL_WRITEV: usize = 0x42;
pub const SYSCALL_PREAD64: usize = 0x43;
pub const SYSCALL_PWRITE64: usize = 0x44;
pub const SYSCALL_PREADV: usize = 0x45;
pub const SYSCALL_PWRITEV: usize = 0x46;
pub const SYSCALL_SENDFILE: usize = 0x47;
pub const SYSCALL_PSELECT6: usize = 0x48;
pub const SYSCALL_PPOLL: usize = 0x49;
pub const SYSCALL_SIGNALFD4: usize = 0x4a;
pub const SYSCALL_VMSPLICE: usize = 0x4b;
pub const SYSCALL_SPLICE: usize = 0x4c;
pub const SYSCALL_TEE: usize = 0x4d;
pub const SYSCALL_READLINKAT: usize = 0x4e;
pub const SYSCALL_NEWFSTATAT: usize = 0x4f;
pub const SYSCALL_FSTAT: usize = 0x50;
pub const SYSCALL_SYNC: usize = 0x51;
pub const SYSCALL_FSYNC: usize = 0x52;
pub const SYSCALL_FDATASYNC: usize = 0x53;
pub const SYSCALL_SYNC_FILE_RANGE: usize = 0x54;
pub const SYSCALL_TIMERFD_CREATE: usize = 0x55;
pub const SYSCALL_TIMERFD_SETTIME: usize = 0x56;
pub const SYSCALL_TIMERFD_GETTIME: usize = 0x57;
pub const SYSCALL_UTIMENSAT: usize = 0x58;
pub const SYSCALL_ACCT: usize = 0x59;
pub const SYSCALL_CAPGET: usize = 0x5a;
pub const SYSCALL_CAPSET: usize = 0x5b;
pub const SYSCALL_PERSONALITY: usize = 0x5c;
pub const SYSCALL_EXIT: usize = 0x5d;
pub const SYSCALL_EXIT_GROUP: usize = 0x5e;
pub const SYSCALL_WAITID: usize = 0x5f;
pub const SYSCALL_SET_TID_ADDRESS: usize = 0x60;
pub const SYSCALL_UNSHARE: usize = 0x61;
pub const SYSCALL_FUTEX: usize = 0x62;
pub const SYSCALL_SET_ROBUST_LIST: usize = 0x63;
pub const SYSCALL_GET_ROBUST_LIST: usize = 0x64;
pub const SYSCALL_NANOSLEEP: usize = 0x65;
pub const SYSCALL_GETITIMER: usize = 0x66;
pub const SYSCALL_SETITIMER: usize = 0x67;
pub const SYSCALL_KEXEC_LOAD: usize = 0x68;
pub const SYSCALL_INIT_MODULE: usize = 0x69;
pub const SYSCALL_DELETE_MODULE: usize = 0x6a;
pub const SYSCALL_TIMER_CREATE: usize = 0x6b;
pub const SYSCALL_TIMER_GETTIME: usize = 0x6c;
pub const SYSCALL_TIMER_GETOVERRUN: usize = 0x6d;
pub const SYSCALL_TIMER_SETTIME: usize = 0x6e;
pub const SYSCALL_TIMER_DELETE: usize = 0x6f;
pub const SYSCALL_CLOCK_SETTIME: usize = 0x70;
pub const SYSCALL_CLOCK_GETTIME: usize = 0x71;
pub const SYSCALL_CLOCK_GETRES: usize = 0x72;
pub const SYSCALL_CLOCK_NANOSLEEP: usize = 0x73;
pub const SYSCALL_SYSLOG: usize = 0x74;
pub const SYSCALL_PTRACE: usize = 0x75;
pub const SYSCALL_SCHED_SETPARAM: usize = 0x76;
pub const SYSCALL_SCHED_SETSCHEDULER: usize = 0x77;
pub const SYSCALL_SCHED_GETSCHEDULER: usize = 0x78;
pub const SYSCALL_SCHED_GETPARAM: usize = 0x79;
pub const SYSCALL_SCHED_SETAFFINITY: usize = 0x7a;
pub const SYSCALL_SCHED_GETAFFINITY: usize = 0x7b;
pub const SYSCALL_SCHED_YIELD: usize = 0x7c;
pub const SYSCALL_SCHED_GET_PRIORITY_MAX: usize = 0x7d;
pub const SYSCALL_SCHED_GET_PRIORITY_MIN: usize = 0x7e;
pub const SYSCALL_SCHED_RR_GET_INTERVAL: usize = 0x7f;
pub const SYSCALL_RESTART_SYSCALL: usize = 0x80;
pub const SYSCALL_KILL: usize = 0x81;
pub const SYSCALL_TKILL: usize = 0x82;
pub const SYSCALL_TGKILL: usize = 0x83;
pub const SYSCALL_SIGALTSTACK: usize = 0x84;
pub const SYSCALL_RT_SIGSUSPEND: usize = 0x85;
pub const SYSCALL_RT_SIGACTION: usize = 0x86;
pub const SYSCALL_RT_SIGPROCMASK: usize = 0x87;
pub const SYSCALL_RT_SIGPENDING: usize = 0x88;
pub const SYSCALL_RT_SIGTIMEDWAIT: usize = 0x89;
pub const SYSCALL_RT_SIGQUEUEINFO: usize = 0x8a;
pub const SYSCALL_RT_SIGRETURN: usize = 0x8b;
pub const SYSCALL_SETPRIORITY: usize = 0x8c;
pub const SYSCALL_GETPRIORITY: usize = 0x8d;
pub const SYSCALL_REBOOT: usize = 0x8e;
pub const SYSCALL_SETREGID: usize = 0x8f;
pub const SYSCALL_SETGID: usize = 0x90;
pub const SYSCALL_SETREUID: usize = 0x91;
pub const SYSCALL_SETUID: usize = 0x92;
pub const SYSCALL_SETRESUID: usize = 0x93;
pub const SYSCALL_GETRESUID: usize = 0x94;
pub const SYSCALL_SETRESGID: usize = 0x95;
pub const SYSCALL_GETRESGID: usize = 0x96;
pub const SYSCALL_SETFSUID: usize = 0x97;
pub const SYSCALL_SETFSGID: usize = 0x98;
pub const SYSCALL_TIMES: usize = 0x99;
pub const SYSCALL_SETPGID: usize = 0x9a;
pub const SYSCALL_GETPGID: usize = 0x9b;
pub const SYSCALL_GETSID: usize = 0x9c;
pub const SYSCALL_SETSID: usize = 0x9d;
pub const SYSCALL_GETGROUPS: usize = 0x9e;
pub const SYSCALL_SETGROUPS: usize = 0x9f;
pub const SYSCALL_UNAME: usize = 0xa0;
pub const SYSCALL_SETHOSTNAME: usize = 0xa1;
pub const SYSCALL_SETDOMAINNAME: usize = 0xa2;
pub const SYSCALL_GETRLIMIT: usize = 0xa3;
pub const SYSCALL_SETRLIMIT: usize = 0xa4;
pub const SYSCALL_GETRUSAGE: usize = 0xa5;
pub const SYSCALL_UMASK: usize = 0xa6;
pub const SYSCALL_PRCTL: usize = 0xa7;
pub const SYSCALL_GETCPU: usize = 0xa8;
pub const SYSCALL_GETTIMEOFDAY: usize = 0xa9;
pub const SYSCALL_SETTIMEOFDAY: usize = 0xaa;
pub const SYSCALL_ADJTIMEX: usize = 0xab;
pub const SYSCALL_GETPID: usize = 0xac;
pub const SYSCALL_GETPPID: usize = 0xad;
pub const SYSCALL_GETUID: usize = 0xae;
pub const SYSCALL_GETEUID: usize = 0xaf;
pub const SYSCALL_GETGID: usize = 0xb0;
pub const SYSCALL_GETEGID: usize = 0xb1;
pub const SYSCALL_GETTID: usize = 0xb2;
pub const SYSCALL_SYSINFO: usize = 0xb3;
pub const SYSCALL_MQ_OPEN: usize = 0xb4;
pub const SYSCALL_MQ_UNLINK: usize = 0xb5;
pub const SYSCALL_MQ_TIMEDSEND: usize = 0xb6;
pub const SYSCALL_MQ_TIMEDRECEIVE: usize = 0xb7;
pub const SYSCALL_MQ_NOTIFY: usize = 0xb8;
pub const SYSCALL_MQ_GETSETATTR: usize = 0xb9;
pub const SYSCALL_MSGGET: usize = 0xba;
pub const SYSCALL_MSGCTL: usize = 0xbb;
pub const SYSCALL_MSGRCV: usize = 0xbc;
pub const SYSCALL_MSGSND: usize = 0xbd;
pub const SYSCALL_SEMGET: usize = 0xbe;
pub const SYSCALL_SEMCTL: usize = 0xbf;
pub const SYSCALL_SEMTIMEDOP: usize = 0xc0;
pub const SYSCALL_SEMOP: usize = 0xc1;
pub const SYSCALL_SHMGET: usize = 0xc2;
pub const SYSCALL_SHMCTL: usize = 0xc3;
pub const SYSCALL_SHMAT: usize = 0xc4;
pub const SYSCALL_SHMDT: usize = 0xc5;
pub const SYSCALL_SOCKET: usize = 0xc6;
pub const SYSCALL_SOCKETPAIR: usize = 0xc7;
pub const SYSCALL_BIND: usize = 0xc8;
pub const SYSCALL_LISTEN: usize = 0xc9;
pub const SYSCALL_ACCEPT: usize = 0xca;
pub const SYSCALL_CONNECT: usize = 0xcb;
pub const SYSCALL_GETSOCKNAME: usize = 0xcc;
pub const SYSCALL_GETPEERNAME: usize = 0xcd;
pub const SYSCALL_SENDTO: usize = 0xce;
pub const SYSCALL_RECVFROM: usize = 0xcf;
pub const SYSCALL_SETSOCKOPT: usize = 0xd0;
pub const SYSCALL_GETSOCKOPT: usize = 0xd1;
pub const SYSCALL_SHUTDOWN: usize = 0xd2;
pub const SYSCALL_SENDMSG: usize = 0xd3;
pub const SYSCALL_RECVMSG: usize = 0xd4;
pub const SYSCALL_READAHEAD: usize = 0xd5;
pub const SYSCALL_BRK: usize = 0xd6;
pub const SYSCALL_MUNMAP: usize = 0xd7;
pub const SYSCALL_MREMAP: usize = 0xd8;
pub const SYSCALL_ADD_KEY: usize = 0xd9;
pub const SYSCALL_REQUEST_KEY: usize = 0xda;
pub const SYSCALL_KEYCTL: usize = 0xdb;
pub const SYSCALL_CLONE: usize = 0xdc;
pub const SYSCALL_EXECVE: usize = 0xdd;
pub const SYSCALL_MMAP: usize = 0xde;
pub const SYSCALL_FADVISE64: usize = 0xdf;
pub const SYSCALL_SWAPON: usize = 0xe0;
pub const SYSCALL_SWAPOFF: usize = 0xe1;
pub const SYSCALL_MPROTECT: usize = 0xe2;
pub const SYSCALL_MSYNC: usize = 0xe3;
pub const SYSCALL_MLOCK: usize = 0xe4;
pub const SYSCALL_MUNLOCK: usize = 0xe5;
pub const SYSCALL_MLOCKALL: usize = 0xe6;
pub const SYSCALL_MUNLOCKALL: usize = 0xe7;
pub const SYSCALL_MINCORE: usize = 0xe8;
pub const SYSCALL_MADVISE: usize = 0xe9;
pub const SYSCALL_REMAP_FILE_PAGES: usize = 0xea;
pub const SYSCALL_MBIND: usize = 0xeb;
pub const SYSCALL_GET_MEMPOLICY: usize = 0xec;
pub const SYSCALL_SET_MEMPOLICY: usize = 0xed;
pub const SYSCALL_MIGRATE_PAGES: usize = 0xee;
pub const SYSCALL_MOVE_PAGES: usize = 0xef;
pub const SYSCALL_RT_TGSIGQUEUEINFO: usize = 0xf0;
pub const SYSCALL_PERF_EVENT_OPEN: usize = 0xf1;
pub const SYSCALL_ACCEPT4: usize = 0xf2;
pub const SYSCALL_RECVMMSG: usize = 0xf3;
pub const SYSCALL_ARCH_SPECIFIC_SYSCALL: usize = 0xf4;
pub const SYSCALL_WAIT4: usize = 0x104;
pub const SYSCALL_PRLIMIT64: usize = 0x105;
pub const SYSCALL_FANOTIFY_INIT: usize = 0x106;
pub const SYSCALL_FANOTIFY_MARK: usize = 0x107;
pub const SYSCALL_NAME_TO_HANDLE_AT: usize = 0x108;
pub const SYSCALL_OPEN_BY_HANDLE_AT: usize = 0x109;
pub const SYSCALL_CLOCK_ADJTIME: usize = 0x10a;
pub const SYSCALL_SYNCFS: usize = 0x10b;
pub const SYSCALL_SETNS: usize = 0x10c;
pub const SYSCALL_SENDMMSG: usize = 0x10d;
pub const SYSCALL_PROCESS_VM_READV: usize = 0x10e;
pub const SYSCALL_PROCESS_VM_WRITEV: usize = 0x10f;
pub const SYSCALL_KCMP: usize = 0x110;
pub const SYSCALL_FINIT_MODULE: usize = 0x111;
pub const SYSCALL_SCHED_SETATTR: usize = 0x112;
pub const SYSCALL_SCHED_GETATTR: usize = 0x113;
pub const SYSCALL_RENAMEAT2: usize = 0x114;
pub const SYSCALL_SECCOMP: usize = 0x115;
pub const SYSCALL_GETRANDOM: usize = 0x116;
pub const SYSCALL_MEMFD_CREATE: usize = 0x117;
pub const SYSCALL_BPF: usize = 0x118;
pub const SYSCALL_EXECVEAT: usize = 0x119;
pub const SYSCALL_USERFAULTFD: usize = 0x11a;
pub const SYSCALL_MEMBARRIER: usize = 0x11b;
pub const SYSCALL_MLOCK2: usize = 0x11c;
pub const SYSCALL_COPY_FILE_RANGE: usize = 0x11d;
pub const SYSCALL_PREADV2: usize = 0x11e;
pub const SYSCALL_PWRITEV2: usize = 0x11f;
pub const SYSCALL_PKEY_MPROTECT: usize = 0x120;
pub const SYSCALL_PKEY_ALLOC: usize = 0x121;
pub const SYSCALL_PKEY_FREE: usize = 0x122;
pub const SYSCALL_STATX: usize = 0x123;
pub const SYSCALL_IO_PGETEVENTS: usize = 0x124;
pub const SYSCALL_RSEQ: usize = 0x125;
pub const SYSCALL_KEXEC_FILE_LOAD: usize = 0x126;
pub const SYSCALL_PIDFD_SEND_SIGNAL: usize = 0x1a8;
pub const SYSCALL_IO_URING_SETUP: usize = 0x1a9;
pub const SYSCALL_IO_URING_ENTER: usize = 0x1aa;
pub const SYSCALL_IO_URING_REGISTER: usize = 0x1ab;
pub const SYSCALL_OPEN_TREE: usize = 0x1ac;
pub const SYSCALL_MOVE_MOUNT: usize = 0x1ad;
pub const SYSCALL_FSOPEN: usize = 0x1ae;
pub const SYSCALL_FSCONFIG: usize = 0x1af;
pub const SYSCALL_FSMOUNT: usize = 0x1b0;
pub const SYSCALL_FSPICK: usize = 0x1b1;
pub const SYSCALL_PIDFD_OPEN: usize = 0x1b2;
pub const SYSCALL_CLONE3: usize = 0x1b3;
pub const SYSCALL_CLOSE_RANGE: usize = 0x1b4;
pub const SYSCALL_OPENAT2: usize = 0x1b5;
pub const SYSCALL_PIDFD_GETFD: usize = 0x1b6;
pub const SYSCALL_FACCESSAT2: usize = 0x1b7;
pub const SYSCALL_PROCESS_MADVISE: usize = 0x1b8;
pub const SYSCALL_EPOLL_PWAIT2: usize = 0x1b9;
pub const SYSCALL_MOUNT_SETATTR: usize = 0x1ba;
pub const SYSCALL_LANDLOCK_CREATE_RULESET: usize = 0x1bc;
pub const SYSCALL_LANDLOCK_ADD_RULE: usize = 0x1bd;
pub const SYSCALL_LANDLOCK_RESTRICT_SELF: usize = 0x1be;

/// 自定义
pub const SYSCALL_CUSTOM_LISTDIR: usize = 0x200;

/// Minimum valid system call number.
pub const SYSCALL_CALL_BASE_INDEX: usize = 0x0;

/// String table of system calls names.
pub static SYSCALL_CALL_NAME: &[&str] = &[
    "io_setup",
    "io_destroy",
    "io_submit",
    "io_cancel",
    "io_getevents",
    "setxattr",
    "lsetxattr",
    "fsetxattr",
    "getxattr",
    "lgetxattr",
    "fgetxattr",
    "listxattr",
    "llistxattr",
    "flistxattr",
    "removexattr",
    "lremovexattr",
    "fremovexattr",
    "getcwd",
    "lookup_dcookie",
    "eventfd2",
    "epoll_create1",
    "epoll_ctl",
    "epoll_pwait",
    "dup",
    "dup3",
    "fcntl",
    "inotify_init1",
    "inotify_add_watch",
    "inotify_rm_watch",
    "ioctl",
    "ioprio_set",
    "ioprio_get",
    "flock",
    "mknodat",
    "mkdirat",
    "unlinkat",
    "symlinkat",
    "linkat",
    "",
    "umount2",
    "mount",
    "pivot_root",
    "nfsservctl",
    "statfs",
    "fstatfs",
    "truncate",
    "ftruncate",
    "fallocate",
    "faccessat",
    "chdir",
    "fchdir",
    "chroot",
    "fchmod",
    "fchmodat",
    "fchownat",
    "fchown",
    "openat",
    "close",
    "vhangup",
    "pipe2",
    "quotactl",
    "getdents64",
    "lseek",
    "read",
    "write",
    "readv",
    "writev",
    "pread64",
    "pwrite64",
    "preadv",
    "pwritev",
    "sendfile",
    "pselect6",
    "ppoll",
    "signalfd4",
    "vmsplice",
    "splice",
    "tee",
    "readlinkat",
    "newfstatat",
    "fstat",
    "sync",
    "fsync",
    "fdatasync",
    "sync_file_range",
    "timerfd_create",
    "timerfd_settime",
    "timerfd_gettime",
    "utimensat",
    "acct",
    "capget",
    "capset",
    "personality",
    "exit",
    "exit_group",
    "waitid",
    "set_tid_address",
    "unshare",
    "futex",
    "set_robust_list",
    "get_robust_list",
    "nanosleep",
    "getitimer",
    "setitimer",
    "kexec_load",
    "init_module",
    "delete_module",
    "timer_create",
    "timer_gettime",
    "timer_getoverrun",
    "timer_settime",
    "timer_delete",
    "clock_settime",
    "clock_gettime",
    "clock_getres",
    "clock_nanosleep",
    "syslog",
    "ptrace",
    "sched_setparam",
    "sched_setscheduler",
    "sched_getscheduler",
    "sched_getparam",
    "sched_setaffinity",
    "sched_getaffinity",
    "sched_yield",
    "sched_get_priority_max",
    "sched_get_priority_min",
    "sched_rr_get_interval",
    "restart_syscall",
    "kill",
    "tkill",
    "tgkill",
    "sigaltstack",
    "rt_sigsuspend",
    "rt_sigaction",
    "rt_sigprocmask",
    "rt_sigpending",
    "rt_sigtimedwait",
    "rt_sigqueueinfo",
    "rt_sigreturn",
    "setpriority",
    "getpriority",
    "reboot",
    "setregid",
    "setgid",
    "setreuid",
    "setuid",
    "setresuid",
    "getresuid",
    "setresgid",
    "getresgid",
    "setfsuid",
    "setfsgid",
    "times",
    "setpgid",
    "getpgid",
    "getsid",
    "setsid",
    "getgroups",
    "setgroups",
    "uname",
    "sethostname",
    "setdomainname",
    "getrlimit",
    "setrlimit",
    "getrusage",
    "umask",
    "prctl",
    "getcpu",
    "gettimeofday",
    "settimeofday",
    "adjtimex",
    "getpid",
    "getppid",
    "getuid",
    "geteuid",
    "getgid",
    "getegid",
    "gettid",
    "sysinfo",
    "mq_open",
    "mq_unlink",
    "mq_timedsend",
    "mq_timedreceive",
    "mq_notify",
    "mq_getsetattr",
    "msgget",
    "msgctl",
    "msgrcv",
    "msgsnd",
    "semget",
    "semctl",
    "semtimedop",
    "semop",
    "shmget",
    "shmctl",
    "shmat",
    "shmdt",
    "socket",
    "socketpair",
    "bind",
    "listen",
    "accept",
    "connect",
    "getsockname",
    "getpeername",
    "sendto",
    "recvfrom",
    "setsockopt",
    "getsockopt",
    "shutdown",
    "sendmsg",
    "recvmsg",
    "readahead",
    "brk",
    "munmap",
    "mremap",
    "add_key",
    "request_key",
    "keyctl",
    "clone",
    "execve",
    "mmap",
    "fadvise64",
    "swapon",
    "swapoff",
    "mprotect",
    "msync",
    "mlock",
    "munlock",
    "mlockall",
    "munlockall",
    "mincore",
    "madvise",
    "remap_file_pages",
    "mbind",
    "get_mempolicy",
    "set_mempolicy",
    "migrate_pages",
    "move_pages",
    "rt_tgsigqueueinfo",
    "perf_event_open",
    "accept4",
    "recvmmsg",
    "arch_specific_syscall",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "wait4",
    "prlimit64",
    "fanotify_init",
    "fanotify_mark",
    "name_to_handle_at",
    "open_by_handle_at",
    "clock_adjtime",
    "syncfs",
    "setns",
    "sendmmsg",
    "process_vm_readv",
    "process_vm_writev",
    "kcmp",
    "finit_module",
    "sched_setattr",
    "sched_getattr",
    "renameat2",
    "seccomp",
    "getrandom",
    "memfd_create",
    "bpf",
    "execveat",
    "userfaultfd",
    "membarrier",
    "mlock2",
    "copy_file_range",
    "preadv2",
    "pwritev2",
    "pkey_mprotect",
    "pkey_alloc",
    "pkey_free",
    "statx",
    "io_pgetevents",
    "rseq",
    "kexec_file_load",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "pidfd_send_signal",
    "io_uring_setup",
    "io_uring_enter",
    "io_uring_register",
    "open_tree",
    "move_mount",
    "fsopen",
    "fsconfig",
    "fsmount",
    "fspick",
    "pidfd_open",
    "clone3",
    "close_range",
    "openat2",
    "pidfd_getfd",
    "faccessat2",
    "process_madvise",
    "epoll_pwait2",
    "mount_setattr",
    "",
    "landlock_create_ruleset",
    "landlock_add_rule",
    "landlock_restrict_self",
];
