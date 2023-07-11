pub mod pid;
pub mod processor;
pub mod scheduler;
pub mod signal;
pub mod stack;

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

use self::{fdtable::FdTable, pid::Pid, signal::SignalActions, stack::KernelStack};

#[allow(clippy::upper_case_acronyms)]
pub struct PCB {
    // 在初始化之后就不再变化的元数据
    // pid 进程唯一标识符
    pub pid: Pid,

    // 在运行过程中可能发生变化的元数据
    inner: UPSafeCell<PCBInner>,
}

impl PCB {
    pub fn ex_inner(&self) -> core::cell::RefMut<'_, PCBInner> {
        self.inner.exclusive_access()
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }

    pub fn address_space(&self) -> &mut AddressSpace {
        self.ex_inner().address_space()
    }

    /// pid 在该函数内惟一的作用就是决定内核栈的位置
    /// task_cx 需要用到该位置
    /// 注意该函数只应该调用一次, 剩下的进程全都是用 fork 创建出来
    pub fn new_once(elf_data: &[u8], cmd: &str) -> Arc<Self> {
        // 分配 pid
        let pid = Pid::alloc();

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(&elf);

        let pcb = Arc::new(Self {
            pid,
            inner: unsafe { UPSafeCell::new(PCBInner::new(address_space, cmd)) },
        });

        // init 默认优先级是 3
        let tcb = Arc::new(TCB::new_once(&pcb, user_sp, entry_point, 3));

        // 准备 crt0 栈
        pcb.address_space().init_crt0(tcb.trap_ctx_ppn());

        // 进程的第一个可运行线程
        pcb.ex_inner().tcbs.push(Some(tcb));

        pcb
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // 访问父进程
        let mut parent_pcb_inner = self.ex_inner();
        assert_eq!(parent_pcb_inner.tcb_count(), 1);

        // 拷贝父进程用户态地址空间(内核态空间不能拷贝)
        let child_address_space = AddressSpace::from_fork(parent_pcb_inner.address_space());

        // 由于 address_space 即将 move, 所以先保存子进程自己的 trap
        let trap_ctx_ppn = child_address_space.trap_ctx_ppn();

        // 先构建 pcb
        let child_pcb = Arc::new(PCB {
            pid: Pid::alloc(), // 分配 pid
            inner: unsafe {
                UPSafeCell::new(PCBInner {
                    tcbs: Vec::new(), // 暂时还未放置线程
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
                    // task_res_allocator: RecycleAllocator::new(),
                    // mutex_list: Vec::new(),
                    // semaphore_list: Vec::new(),
                    // condvar_list: Vec::new(),
                })
            },
        });

        // 父子关系
        parent_pcb_inner.children.push(child_pcb.clone());

        // pcb 构建完毕, 开始构建 tcb
        let parent_tcb = parent_pcb_inner.get_tcb(0);
        drop(parent_pcb_inner);

        let kstack = KernelStack::alloc();
        let kstack_top = kstack.get_top();

        let child_tcb: Arc<TCB> = Arc::new(TCB {
            pcb: Arc::downgrade(&child_pcb),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TCBInner {
                    priority: parent_tcb.priority(),
                    task_status: TaskStatus::Ready,
                    count: 0,
                    task_ctx: TaskContext::goto_trap_return(kstack_top),
                    trap_ctx_ppn,
                    base_size: parent_tcb.base_size(),
                })
            },
        });

        // 设置该 tcb 的内核栈
        let mut tcb_inner = child_tcb.ex_inner();
        let trap_ctx = tcb_inner.trap_ctx();
        trap_ctx.kernel_sp = kstack_top;
        drop(tcb_inner);

        // 将构建好的 tcb 放入其中
        let mut child_pcb_inner = child_pcb.ex_inner();
        child_pcb_inner.tcbs.push(Some(child_tcb));
        drop(child_pcb_inner);

        child_pcb
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
        let trap_cx_ppn = address_space.trap_ctx_ppn();

        let mut pcb_inner = self.ex_inner();
        // 替换地址空间, 原来的地址空间全部被回收, 页表也更换了
        pcb_inner.address_space = address_space;
        // 更新名称
        pcb_inner.cmd = String::from(app_name);

        let tcb = pcb_inner.get_tcb(0);
        // 更新 trap_cx ppn
        tcb.set_trap_ctx_ppn(trap_cx_ppn);
        // 更新 base_size
        tcb.set_base_size(user_sp);

        // 取出进程的 trap_cx 并更新
        let trap_cx = tcb.trap_ctx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            tcb.kstack.get_top(), // 复用子进程自身的 kernel_stack
            trap_handler as usize,
        );

        // 压入 crt0 栈
        pcb_inner.address_space.push_crt0(trap_cx, &args, &envs);

        // 注意: 在执行 execve() 后:
        // 子进程的程序映像被替换为新的程序，但是文件描述符表不会受到影响，仍然保持不变。
        // 因此，子进程在 execve() 执行后会继续使用父进程继承的文件描述符。

        // 如果希望子进程在 execve() 执行后关闭或重定向文件描述符，
        // 可以在调用 execve() 之前手动关闭或重定向这些文件描述符
        0
    }
}

pub struct PCBInner {
    tcbs: Vec<Option<Arc<TCB>>>,

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

    pub fn get_tcb(&mut self, tid: usize) -> Arc<TCB> {
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

    pub fn status(&self) -> TaskStatus {
        todo!()
    }
    pub fn set_zombie(&mut self) {
        self.is_zombie = true;
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn is_zombie(&self) -> bool {
        self.is_zombie
    }

    pub fn user_token(&self) -> usize {
        self.address_space.token()
    }

    // pub fn trap_cx(&self) -> &'static mut TrapContext {
    //     self.tcb.trap_cx_ppn.get_mut()
    // }
    // pub fn task_cx(&mut self) -> &'static mut TaskContext {
    //     let ctx = &mut self.tcb.task_cx as *mut TaskContext;
    //     unsafe { ctx.as_mut().unwrap() }
    // }

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
