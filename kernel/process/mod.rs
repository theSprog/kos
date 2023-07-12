pub mod kstack;
pub mod pid;
pub mod processor;
pub mod scheduler;
pub mod signal;

mod fdtable;

use core::cell::UnsafeCell;

use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use component::fs::vfs::VfsPath;
use logger::info;
use spin::Mutex;
use sys_interface::{syserr, syssig::*};

use crate::{
    loader::load_app,
    memory::address_space::{AddressSpace, KERNEL_SPACE},
    sync::unicore::UPSafeCell,
    task::{context::TaskContext, TCBInner, TaskStatus, TCB},
    trap::{context::TrapContext, trap_handler},
};

use self::{
    fdtable::FdTable,
    kstack::KernelStack,
    pid::{Pid, RecycleAllocator},
    signal::SignalActions,
};

#[allow(clippy::upper_case_acronyms)]
pub struct PCB {
    // 在初始化之后就不再变化的元数据
    // pid 进程唯一标识符
    pub pid: Pid,

    // 在运行过程中可能发生变化的元数据
    inner: UPSafeCell<PCBInner>,
}

impl PCB {
    /// task_ctx 需要用到该位置
    /// 注意该函数只应该调用一次, 剩下的进程全都是用 fork 创建出来
    pub fn new_once(elf_data: &[u8], cmd: &str) -> Arc<Self> {
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let (address_space, ustack_base, entry_point) = AddressSpace::from_elf(&elf);

        let pcb = Arc::new(Self {
            pid: Pid::alloc(), // 分配 pid
            inner: unsafe { UPSafeCell::new(PCBInner::new(address_space, cmd)) },
        });

        // init 默认优先级是 3
        let tcb = Arc::new(TCB::new(&pcb, ustack_base, false));

        let tcb_inner = tcb.ex_inner();
        let trap_ctx = tcb_inner.trap_ctx();
        let (ustack_top, kstack_top) = (tcb_inner.ustack_top(), tcb.kstack.get_top());
        drop(tcb_inner);

        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            ustack_top,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        );

        // 为 main 线程准备 crt0 栈
        pcb.ex_address_space().init_crt0(tcb.ex_inner().trap_ctx());

        // 进程的第一个可运行线程
        pcb.ex_add_tcb(tcb.clone());

        pcb
    }

    pub fn ex_inner(&self) -> core::cell::RefMut<'_, PCBInner> {
        self.inner.exclusive_access()
    }
    pub fn pid(&self) -> usize {
        self.pid.0
    }

    pub fn ex_address_space(&self) -> &'static mut AddressSpace {
        self.ex_inner().address_space()
    }

    pub fn ex_fd_table(&self) -> &'static mut FdTable {
        self.ex_inner().fd_table()
    }

    pub fn ex_add_tcb(&self, tcb: Arc<TCB>) {
        self.ex_inner().tcbs.push(Some(tcb));
    }

    pub fn ex_ustack_base(&self) -> usize {
        self.ex_inner().ustack_base()
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // 访问父进程
        let mut parent_pcb_inner = self.ex_inner();
        assert_eq!(parent_pcb_inner.tcb_count(), 1);

        // 拷贝父进程用户态地址空间(内核态空间不能拷贝)
        let parent_tcb = parent_pcb_inner.main_tcb();
        let child_address_space =
            AddressSpace::from_fork(parent_pcb_inner.address_space(), parent_tcb.trap_ctx_ppn());

        // 先构建 pcb
        let child_pcb = Arc::new(PCB {
            pid: Pid::alloc(), // 分配 pid
            inner: unsafe {
                UPSafeCell::new(PCBInner {
                    tcbs: Vec::new(), // 暂时还未放置线程
                    tid_allocator: RecycleAllocator::new(),
                    address_space: child_address_space,
                    fd_table: parent_pcb_inner.fd_table.clone(),

                    is_zombie: false,
                    // 父进程是 self, 暂时没有子进程
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    cmd: parent_pcb_inner.cmd().to_string(),
                    cwd: parent_pcb_inner.cwd().clone(),
                    pending_signals: SignalFlags::empty(),

                    signal_mask: parent_pcb_inner.signal_mask,
                    handling_sig: -1,
                    signal_actions: parent_pcb_inner.signal_actions.clone(),
                    killed: false,
                    frozen: false,
                    trap_ctx_backup: None,
                    // mutex_list: Vec::new(),
                    // semaphore_list: Vec::new(),
                    // condvar_list: Vec::new(),
                })
            },
        });
        // 父子关系
        parent_pcb_inner.children.push(child_pcb.clone());

        // pcb 构建完毕, 开始构建 tcb
        let ustack_base = parent_pcb_inner.ustack_base();
        drop(parent_pcb_inner);

        let child_tcb = Arc::new(TCB::new(&child_pcb, ustack_base, false));

        // 设置该 tcb 的内核栈
        let tcb_inner = child_tcb.ex_inner();
        let trap_ctx = tcb_inner.trap_ctx();
        trap_ctx.kernel_sp = child_tcb.kstack.get_top();
        drop(tcb_inner);

        // 将构建好的 tcb 放入其中
        child_pcb.ex_add_tcb(child_tcb);

        child_pcb
    }

    /// Only support processes with a single thread.
    pub fn exec(&self, app_name: &str, args: Vec<String>, envs: Vec<String>) -> isize {
        assert_eq!(self.ex_inner().tcb_count(), 1);

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

        let (address_space, ustack_base, entry_point) = AddressSpace::from_elf(&elf);

        let mut pcb_inner = self.ex_inner();
        // 替换地址空间, 原来的地址空间全部被回收, 页表也更换了
        pcb_inner.address_space = address_space;
        // 更新名称
        pcb_inner.cmd = String::from(app_name);

        let tcb = pcb_inner.main_tcb();
        drop(pcb_inner);

        let mut tcb_inner = tcb.ex_inner();
        tcb_inner.set_ustack_base(ustack_base);

        let trap_ctx_ppn = tcb_inner.resource().trap_ctx_ppn_ex();
        // 原先的 trap_ctx 物理页已被回收, 现在需要新换上物理页
        tcb_inner.set_trap_ctx_ppn(trap_ctx_ppn);

        let trap_ctx = tcb_inner.trap_ctx();
        let (ustack_top, kstack_top) = (tcb_inner.ustack_top(), tcb.kstack.get_top());

        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            ustack_top,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        );

        // // 压入 crt0 栈
        let mut pcb_inner = self.ex_inner();
        pcb_inner.address_space.update_crt0(trap_ctx, &args, &envs);
        drop(pcb_inner);

        // // 注意: 在执行 execve() 后:
        // // 子进程的程序映像被替换为新的程序，但是文件描述符表不会受到影响，仍然保持不变。
        // // 因此，子进程在 execve() 执行后会继续使用从父进程继承的文件描述符。

        // // 如果希望子进程在 execve() 执行后关闭或重定向文件描述符，
        // // 可以在调用 execve() 之前手动关闭或重定向这些文件描述符
        0
    }
}

pub struct PCBInner {
    tcbs: Vec<Option<Arc<TCB>>>,
    tid_allocator: RecycleAllocator,

    // 进程的地址空间
    address_space: AddressSpace,
    // 进程的文件描述符集合
    fd_table: FdTable,

    // 当前进程是否是 zombie
    is_zombie: bool,

    // 当前所执行的命令
    cmd: String,

    // 当前进程所在目录
    cwd: VfsPath,

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

    // 上下文备份, 信号处理函数进入时保存上下文, 返回时要恢复上下文
    trap_ctx_backup: Option<TrapContext>,
    // mutex_list: Vec<Option<Arc<dyn KMutex>>>, // 互斥锁列表
    // semaphore_list: Vec<Option<Arc<KSemaphore>>>, // 信号量列表
    // condvar_list: Vec<Option<Arc<KCondvar>>>, // 条件变量列表
}

impl PCBInner {
    pub fn new(address_space: AddressSpace, cmd: &str) -> Self {
        Self {
            // 进程初创时没有线程
            tcbs: Vec::new(),
            tid_allocator: RecycleAllocator::new(),
            address_space,
            fd_table: FdTable::default(),

            is_zombie: false,
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

    pub fn alloc_tid(&mut self) -> usize {
        self.tid_allocator.alloc()
    }

    pub fn dealloc_tid(&mut self, tid: usize) {
        self.tid_allocator.dealloc(tid)
    }

    pub fn main_tcb(&self) -> Arc<TCB> {
        self.get_tcb(0)
    }

    pub fn get_tcb(&self, tid: usize) -> Arc<TCB> {
        self.tcbs[tid].as_ref().unwrap().clone()
    }

    pub fn tcb_count(&self) -> usize {
        self.tcbs.len()
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

    pub fn is_zombie(&self) -> bool {
        self.is_zombie
    }

    pub fn set_zombie(&mut self) {
        self.is_zombie = true;
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = exit_code;
    }

    pub fn user_token(&self) -> usize {
        self.address_space.token()
    }

    pub fn ustack_base(&self) -> usize {
        self.main_tcb().ustack_base()
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
