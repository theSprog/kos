use super::PCB;
use crate::process::scheduler;
use crate::task::switch::__switch;
use crate::task::TaskStatus;
use crate::{memory::address::*, task::TCB};
use crate::{sync::unicore::UPSafeCell, task::context::TaskContext, trap::context::TrapContext};
use alloc::string::String;
use alloc::sync::Arc;
use logger::*;
use sys_interface::config::PAGE_SIZE;

use crate::{sbi::shutdown, task::INITPROC};

lazy_static! {
    pub(crate) static ref PROCESSOR: UPSafeCell<Processor> = unsafe {
        info!("PROCESSOR initializing...");
        UPSafeCell::new(Processor::new())
    };
}

pub struct Processor {
    current: Option<Arc<TCB>>,
    idle_task_cx: TaskContext, // idle 进程
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::idle(), // 最初的 unused TaskContext
        }
    }

    // 取出当前正在执行的任务而不是得到一份拷贝
    // 注意 take 之后 current 就为 None 了, 无法在使用 api 内的许多函数
    pub fn take_current(&mut self) -> Option<Arc<TCB>> {
        self.current.take()
    }

    // 返回当前执行的任务的一份拷贝, 会增加引用计数
    pub fn current(&self) -> Option<Arc<TCB>> {
        self.current.as_ref().map(|pcb| Arc::clone(pcb))
    }

    fn get_idle_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

pub mod api {
    use crate::{
        memory::address_space::AddressSpace,
        process::{fdtable::FdTable, pid::PID_MAP, processor},
    };
    use logger::*;
    use sys_interface::syssig::SignalFlags;

    use super::*;

    pub fn get_idle_cx_ptr() -> *mut TaskContext {
        PROCESSOR.exclusive_access().get_idle_cx_ptr()
    }

    pub fn current_pcb() -> Arc<PCB> {
        PROCESSOR
            .exclusive_access()
            .current()
            .unwrap()
            .pcb()
            .unwrap()
    }

    pub fn current_tcb() -> Option<Arc<TCB>> {
        PROCESSOR.exclusive_access().current()
    }

    pub fn current_kstack_top() -> usize {
        match current_tcb() {
            Some(tcb) => tcb.kstack.get_top(),
            None => loop {}, // before tcb construct, imply inner error, stop immediately
        }
    }

    pub fn take_current_tcb() -> Option<Arc<TCB>> {
        PROCESSOR.exclusive_access().take_current()
    }

    pub fn current_cmd_name() -> String {
        String::from(current_pcb().ex_inner().cmd())
    }

    pub fn current_ex_address_space() -> &'static mut AddressSpace {
        current_pcb().ex_address_space()
    }

    pub fn current_ex_fdtable() -> &'static mut FdTable {
        current_pcb().ex_fd_table()
    }

    pub fn current_pid() -> usize {
        current_pcb().pid()
    }

    pub fn current_tid() -> usize {
        current_tcb().unwrap().ex_inner().tid()
    }

    pub fn current_user_token() -> usize {
        current_pcb().ex_inner().user_token()
    }

    pub fn current_trap_ctx() -> &'static mut TrapContext {
        current_tcb().unwrap().ex_inner().trap_ctx()
    }

    // 此处返回的是用户态所看到的 trap 虚拟地址
    pub fn current_trap_ctx_uptr() -> usize {
        current_tcb().unwrap().ex_inner().resource().trap_ctx_uptr()
    }

    pub fn pid2pcb(pid: usize) -> Option<Arc<PCB>> {
        let map = PID_MAP.exclusive_access();
        map.get(&pid).map(Arc::clone)
    }

    pub fn current_add_signal(signal: SignalFlags) {
        let pcb = processor::api::current_pcb();
        let mut inner = pcb.ex_inner();
        inner.pending_signals |= signal;
        info!("current task sigflag {:?}", inner.pending_signals());
    }

    pub fn run_app() {
        info!("start running app");
        let mut processor = PROCESSOR.exclusive_access();
        let idle_cx_ptr = processor.get_idle_cx_ptr();
        drop(processor);
        schedule(idle_cx_ptr);
        unreachable!()
    }

    // 本函数只管对下一个进程设置, 不负责对当前线程进行设置
    pub fn schedule(current_task_cx_ptr: *mut TaskContext) {
        let mut processor = PROCESSOR.exclusive_access();

        // 如果找得到下一个进程
        if let Some(tcb_next) = scheduler::fetch() {
            let mut tcb_next_inner = tcb_next.ex_inner();

            // 互斥访问下一个 TCB
            tcb_next_inner.set_status(TaskStatus::Running);
            tcb_next_inner.inc_count();
            let pcb_next_ctx_ptr: *const TaskContext =
                tcb_next_inner.task_ctx() as *const TaskContext;

            drop(tcb_next_inner);
            processor.current = Some(tcb_next);
            drop(processor);

            // 切换任务
            unsafe {
                __switch(current_task_cx_ptr, pcb_next_ctx_ptr);
            }
        } else {
            info!("All applications completed!");
            info!("TODO: this is incomplete because maybe some process in blocking");
            shutdown();
            // 否则, 没有准备好的进程, 回到 idle 进程
            // let idle_cx_ptr = processor.get_idle_cx_ptr();
            // processor.current = None;
            // drop(processor);

            // unsafe {
            //     __switch(current_task_cx_ptr, idle_cx_ptr);
            // }
        }
    }

    pub fn suspend_and_run_next() {
        let tcb = current_tcb().unwrap();
        let mut tcb_inner = tcb.ex_inner();
        tcb_inner.set_status(TaskStatus::Ready);
        let task_ctx_ptr = tcb_inner.task_ctx() as *mut TaskContext;
        drop(tcb_inner);
        // suspend 只是换一个进程调度, 而当前进程仍然是 ready 的
        scheduler::add_ready(tcb);

        schedule(task_ctx_ptr);
    }

    fn exit_main(pcb: Arc<PCB>, exit_code: i32, main_tcb: Arc<TCB>) {
        let pid = pcb.pid();
        // 主线程的 exit_code 就是进程的退出码
        debug!(
            "process-'{}'(pid={}) exited with code {}",
            pcb.ex_inner().cmd(),
            pid,
            exit_code
        );

        let mut pcb_inner = pcb.ex_inner();
        // 设置为 zombie 孩子被 init 收养
        pcb_inner.set_zombie();
        pcb_inner.set_exit_code(exit_code);

        {
            // 访问 init 进程, 所有进程死后它的孩子都归 init 抚养
            let mut initproc_inner = INITPROC.ex_inner();
            for child in pcb_inner.children.iter() {
                child.ex_inner().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // 将剩余所有的线程一并释放
        drop(pcb_inner);
        drop(main_tcb);
        // 有部分线程尚在 scheduler 里面, 找到所有属于本进程的线程
        drop(scheduler::filter(|tcb| tcb.pid() == pid));

        // 逐个 take 取出释放
        let slots = pcb.ex_inner().tcb_slots_len();
        for tid in 0..slots {
            if let Some(tcb) = pcb.ex_drop_tcb(tid) {
                assert_eq!(Arc::strong_count(&tcb), 1, "tid {} not single", tid);
            }
        }

        // 主线程退出后还有其余资源需要释放
        {
            let mut pcb_inner = pcb.ex_inner();
            // 释放对于孩子的所有权
            pcb_inner.children.clear();
            // 释放地址空间, 同时释放页表
            pcb_inner.address_space.release_space();
            // 释放文件描述符
            pcb_inner.fd_table.clear();
        }
    }

    // 从线程退出时仅仅设置标志位（退出码）
    fn exit_slave(pcb: Arc<PCB>, exit_code: i32, slave_tcb: Arc<TCB>) {
        let pid = pcb.pid();
        let tid = slave_tcb.ex_inner().tid();
        debug!("tid={}(pid={}) exited with code {}", tid, pid, exit_code);
        slave_tcb.ex_inner().set_exit_code(exit_code);
    }

    // 退出当前线程, 但不保证退出进程
    pub fn exit_and_run_next(exit_code: i32) -> ! {
        // 已经把 pcb 取出, current 为 None
        let tcb = take_current_tcb().unwrap();
        let pcb = tcb.pcb().unwrap();

        // 如果是主线程, 那么进程也应该退出
        let tid = tcb.ex_inner().tid();
        if tid == 0 {
            exit_main(pcb, exit_code, tcb);
        } else {
            exit_slave(pcb, exit_code, tcb);
        }

        schedule(get_idle_cx_ptr());
        // 不可能到达此处
        unreachable!();
    }

    // TODO: 错误的实现, 权宜之计
    pub(crate) fn sbrk(incrment: usize) -> usize {
        let address_space = current_ex_address_space();
        // 默认最后一个是 heap
        let heap = address_space.heap_mut();

        if incrment == 0 {
            // 获取当前堆顶
            return VirtAddr::from(heap.vpn_range.get_end()).into();
        }

        let inc_vaddr = {
            if incrment % PAGE_SIZE == 0 {
                incrment // 不需要上取，已经是 PAGE_SIZE 的倍数
            } else {
                ((incrment / PAGE_SIZE) + 1) * PAGE_SIZE // 上取到下一个 PAGE_SIZE 的倍数
            }
        };
        assert_eq!(0, inc_vaddr % PAGE_SIZE);

        heap.vpn_range
            .set_end(heap.vpn_range.get_end() + VirtAddr(inc_vaddr).into());

        VirtAddr::from(heap.vpn_range.get_end()).into()
    }
}
