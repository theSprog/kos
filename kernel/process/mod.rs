pub mod pid;
pub mod processor;
pub mod scheduler;
pub mod signal;

mod fdtable;
mod stack;

use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use component::fs::vfs::VfsPath;
use logger::info;
use sys_interface::{syserr, syssig::*};

use crate::{
    loader::load_app,
    memory::address_space::{AddressSpace, KERNEL_SPACE},
    sync::unicore::UPSafeCell,
    task::{context::TaskContext, TaskStatus, TCB},
    trap::{context::TrapContext, trap_handler},
};

use self::{fdtable::FdTable, pid::Pid, signal::SignalActions, stack::KernelStack};

#[allow(clippy::upper_case_acronyms)]
pub struct PCB {
    // 在初始化之后就不再变化的元数据
    // pid 进程唯一标识符
    pub pid: Pid,
    // KernelStack 只是一个 pid, 目的是 RAII, PCB 析构时自动释放内核栈资源
    pub kernel_stack: KernelStack,
    // pub cmd: String,

    // 在运行过程中可能发生变化的元数据
    inner: UPSafeCell<PCBInner>,
}

impl PCB {
    pub fn ex_inner(&self) -> core::cell::RefMut<'_, PCBInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    /// pid 在该函数内惟一的作用就是决定内核栈的位置
    /// task_cx 需要用到该位置
    /// 注意该函数只应该调用一次, 剩下的进程全都是用 fork 创建出来
    pub fn new_once(elf_data: &[u8], cmd: &str) -> Self {
        // 分配 pid
        let pid = pid::api::pid_alloc();
        // 确定内核栈位置
        let kernel_stack = KernelStack::new(&pid);

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(&elf);
        
        // init 默认优先级是 3
        let tcb = TCB::new(user_sp, entry_point, address_space.trap_cx_ppn(), pid.0, 3);

        // 准备 crt0 栈
        address_space.init_crt0(tcb.trap_cx_ppn.get_mut());

        Self {
            pid,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(PCBInner::new(tcb, address_space, cmd)) },
        }
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // 访问父进程
        let mut parent_inner = self.ex_inner();
        // 拷贝用户空间
        let address_space: AddressSpace = AddressSpace::from_fork(&mut parent_inner.address_space);

        // 分配 pid 和 内核栈
        let pid_handle = pid::api::pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let trap_cx_ppn = address_space.trap_cx_ppn();

        let new_tcb = TCB {
            priority: parent_inner.tcb.priority,
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            trap_cx_ppn,
            base_size: parent_inner.tcb.base_size,
        };

        let pcb_inner = PCBInner {
            tcb: new_tcb,
            address_space,
            fd_table: parent_inner.fd_table.clone(),

            count: 0, // 新建进程所用时间片为 0
            // 父进程是 self, 没有子进程
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            cmd: parent_inner.cmd().to_string(),
            exit_code: 0,
            cwd: parent_inner.cwd().clone(),

            pending_signals: SignalFlags::empty(),
            // inherit the signal_mask and signal_action
            signal_mask: parent_inner.signal_mask,
            handling_sig: -1,
            signal_actions: parent_inner.signal_actions.clone(),
            killed: false,
            frozen: false,
            trap_ctx_backup: None,
        };

        let new_pcb = Arc::new(PCB {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(pcb_inner) },
        });

        // add child
        parent_inner.children.push(new_pcb.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = new_pcb.ex_inner().trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        new_pcb

        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }

    pub fn exec(&self, app_name: &str, args: Vec<String>, envs: Vec<String>) -> isize {
        let elf_data = match load_app(app_name) {
            Some(app) => app,
            None => return syserr::ENOENT,
        };

        let elf = match xmas_elf::ElfFile::new(&elf_data) {
            Ok(elf) => elf,
            Err(err) => {
                info!("Failed to parse elf: {}", err);
                return syserr::ENOEXEC;
            }
        };

        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(&elf);
        let trap_cx_ppn = address_space.trap_cx_ppn();

        let mut inner = self.ex_inner();
        // 替换地址空间, 原来的地址空间全部被回收, 页表也更换了
        inner.address_space = address_space;
        // 更新 trap_cx ppn
        inner.tcb.trap_cx_ppn = trap_cx_ppn;
        // 更新 base_size
        inner.tcb.base_size = user_sp;
        // 更新名称
        inner.cmd = String::from(app_name);

        // 取出进程的 trap_cx 并更新
        let trap_cx = inner.trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(), // 复用子进程自身的 kernel_stack
            trap_handler as usize,
        );

        // 压入 crt0 栈
        inner.address_space.push_crt0(trap_cx, &args, &envs);

        // 注意: 在执行 execve() 后:
        // 子进程的程序映像被替换为新的程序，但是文件描述符表不会受到影响，仍然保持不变。
        // 因此，子进程在 execve() 执行后会继续使用父进程继承的文件描述符。

        // 如果希望子进程在 execve() 执行后关闭或重定向文件描述符，
        // 可以在调用 execve() 之前手动关闭或重定向这些文件描述符
        0
    }
}

pub struct PCBInner {
    tcb: TCB,

    // 应用程序的地址空间
    address_space: AddressSpace,
    fd_table: FdTable,

    // 当前所执行的命令
    cmd: String,

    // 当前进程所在目录
    cwd: VfsPath,

    // 进程运行的时间段
    count: usize,

    // 树形结构, 父子进程, 父进程有多个子进程指向它
    // weak 智能指针将不会影响父进程的引用计数
    parent: Option<Weak<PCB>>,
    children: Vec<Arc<PCB>>,

    // 退出码
    exit_code: i32,

    // 记录对应进程目前已经收到了哪些信号尚未处理
    pending_signals: SignalFlags,

    signal_mask: SignalFlags,
    signal_actions: SignalActions,

    handling_sig: isize,

    // 是否被 kill
    killed: bool,
    // 是否被 frozen
    frozen: bool,
    trap_ctx_backup: Option<TrapContext>,
}

impl PCBInner {
    pub fn new(tcb: TCB, address_space: AddressSpace, cmd: &str) -> Self {
        Self {
            tcb,
            address_space,
            fd_table: FdTable::default(),

            count: 0,
            parent: None,
            children: Vec::new(),
            cmd: String::from(cmd),
            exit_code: 0,
            cwd: VfsPath::empty(true),

            pending_signals: SignalFlags::empty(),
            signal_mask: SignalFlags::empty(),
            handling_sig: -1,
            signal_actions: SignalActions::default(),
            killed: false,
            frozen: false,
            trap_ctx_backup: None,
        }
    }

    pub fn tcb(&mut self) -> &'static mut TCB {
        let tcb = &mut self.tcb as *mut TCB;
        unsafe { tcb.as_mut().unwrap() }
    }

    pub fn address_space(&mut self) -> &'static mut AddressSpace {
        let address_space = &mut self.address_space as *mut AddressSpace;
        unsafe { address_space.as_mut().unwrap() }
    }

    pub fn fd_table(&mut self) -> &'static mut FdTable {
        let fd_table = &mut self.fd_table as *mut FdTable;
        unsafe { fd_table.as_mut().unwrap() }
    }

    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn cwd(&self) -> &VfsPath {
        &self.cwd
    }

    pub fn cwd_mut(&mut self) -> &mut VfsPath {
        &mut self.cwd
    }

    pub fn parent(&self) -> Option<Weak<PCB>> {
        self.parent.clone()
    }

    pub fn status(&self) -> TaskStatus {
        self.tcb.task_status
    }
    pub fn set_status(&mut self, status: TaskStatus) {
        self.tcb.task_status = status;
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
    pub fn inc_count(&mut self) {
        // 60 是 1-5 的最小公倍数
        self.count += 60;
    }
    pub fn is_zombie(&self) -> bool {
        self.status() == TaskStatus::Zombie
    }

    pub fn user_token(&self) -> usize {
        self.address_space.token()
    }
    pub fn trap_cx(&self) -> &'static mut TrapContext {
        self.tcb.trap_cx_ppn.get_mut()
    }
    pub fn task_cx(&mut self) -> &'static mut TaskContext {
        let ctx = &mut self.tcb.task_cx as *mut TaskContext;
        unsafe { ctx.as_mut().unwrap() }
    }

    pub fn children_mut(&mut self) -> &mut Vec<Arc<PCB>> {
        &mut self.children
    }

    pub fn children(&self) -> &Vec<Arc<PCB>> {
        &self.children
    }

    pub fn pending_signals(&self) -> &SignalFlags {
        &self.pending_signals
    }

    pub fn pending_signals_mut(&mut self) -> &mut SignalFlags {
        &mut self.pending_signals
    }

    pub fn signal_mask(&self) -> &SignalFlags {
        &self.signal_mask
    }

    pub fn set_signal_mask(&mut self, flag: SignalFlags) {
        self.signal_mask = flag;
    }

    pub fn signal_actions(&self) -> &SignalActions {
        &self.signal_actions
    }

    pub fn signal_actions_mut(&mut self) -> &mut SignalActions {
        &mut self.signal_actions
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }
    pub fn killed(&self) -> bool {
        self.killed
    }

    pub fn handling_sig(&self) -> isize {
        self.handling_sig
    }

    pub fn set_handling_sig(&mut self, handling_sig: isize) {
        self.handling_sig = handling_sig;
    }

    pub fn trap_ctx_backup(&self) -> Option<TrapContext> {
        self.trap_ctx_backup.clone()
    }
}
