use super::PCB;
use crate::process::scheduler;
use crate::task::switch::__switch;
use crate::task::TaskStatus;
use crate::{memory::address::*, task::TCB};
use crate::{sync::unicore::UPSafeCell, task::context::TaskContext, trap::context::TrapContext};
use alloc::sync::Arc;
use logger::info;

lazy_static! {
    pub(crate) static ref PROCESSOR: UPSafeCell<Processor> = unsafe {
        info!("PROCESSOR initializing...");
        UPSafeCell::new(Processor::new())
    };
}

pub struct Processor {
    current: Option<Arc<PCB>>,
    idle_task_cx: TaskContext, // idle 进程
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::idle(), // 最初的 unused TaskContext
        }
    }

    // 取出当前正在执行的任务
    pub fn take_current(&mut self) -> Option<Arc<PCB>> {
        self.current.take()
    }

    // 返回当前执行的任务的一份拷贝, 会增加引用计数
    pub fn current(&self) -> Option<Arc<PCB>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }

    fn get_idle_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

pub mod api {

    use alloc::string::String;
    use sys_interface::config::PAGE_SIZE;

    use crate::sbi::shutdown;

    use super::*;

    pub fn take_current_pcb() -> Option<Arc<PCB>> {
        PROCESSOR.exclusive_access().take_current()
    }

    pub fn current_pcb() -> Option<Arc<PCB>> {
        PROCESSOR.exclusive_access().current()
    }

    pub fn current_pcb_name() -> String {
        String::from(current_pcb().unwrap().inner().prog_name())
    }

    pub fn current_tcb() -> &'static mut TCB {
        current_pcb().unwrap().inner().tcb()
    }

    pub fn current_pid() -> usize {
        current_pcb().unwrap().getpid()
    }

    pub fn current_user_token() -> usize {
        current_pcb().unwrap().inner().user_token()
    }

    pub fn current_trap_cx() -> &'static mut TrapContext {
        current_pcb().unwrap().inner().trap_cx()
    }

    pub fn run_app() {
        let mut processor = PROCESSOR.exclusive_access();
        let idle_cx_ptr = processor.get_idle_cx_ptr();
        drop(processor);
        schedule(idle_cx_ptr);
    }

    // 本函数只管对下一个进程设置, 不负责对当前线程进行设置
    pub fn schedule(current_task_cx_ptr: *mut TaskContext) {
        let mut processor = PROCESSOR.exclusive_access();

        // 如果找得到下一个进程
        if let Some(pcb_next) = scheduler::fetch() {
            // 互斥访问下一个 PCB
            let mut pcb_next_inner = pcb_next.inner();
            let pcb_next_cx_ptr = pcb_next_inner.task_cx() as *const TaskContext;
            pcb_next_inner.set_status(TaskStatus::Running);
            pcb_next_inner.inc_count();
            // 停止互斥访问
            drop(pcb_next_inner);
            processor.current = Some(pcb_next);
            drop(processor);

            // 切换任务
            unsafe {
                __switch(current_task_cx_ptr, pcb_next_cx_ptr);
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
        let pcb = current_pcb().unwrap();
        let mut pcb_inner = pcb.inner();
        pcb_inner.set_status(TaskStatus::Ready);
        let task_cx_ptr = pcb_inner.task_cx() as *mut TaskContext;
        drop(pcb_inner);

        // suspend 只是换一个进程调度, 而当前进程仍然是 ready 的
        scheduler::add_ready(pcb.clone());

        schedule(task_cx_ptr);
    }

    pub fn exit_and_run_next() {
        let pcb = current_pcb().unwrap();
        let mut pcb_inner = pcb.inner();

        // 到底是 Zombie 还是 Died ?
        pcb_inner.set_status(TaskStatus::Died);
        let task_cx_ptr = pcb_inner.task_cx() as *mut TaskContext;
        drop(pcb_inner);
        schedule(task_cx_ptr);
    }

    pub(crate) fn sbrk(incrment: usize) -> usize {
        let tcb = current_tcb();
        // 默认最后一个是 heap
        let heap = tcb.address_space.heap();

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
