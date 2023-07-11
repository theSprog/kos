#![no_std]
#![no_main]
#![feature(naked_functions)]
//#![feature(asm)]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use core::arch::asm;

use alloc::vec::Vec;
use user_lib::{exit, sleep};

// 默认栈大小, 不可过小
const DEFAULT_STACK_SIZE: usize = 4096;
const MAX_TASKS: usize = 5;
static mut RUNTIME: usize = 0;

pub struct Runtime {
    workers: Vec<Thread>,
    current: usize,
}

#[derive(PartialEq, Eq, Debug)]
enum ThreadState {
    Init,    // 初始态, 初始时只分配了内存, 尚未指定执行内容
    Running, // 运行态
    Ready,   // 就绪态
}

// 线程控制块
#[allow(dead_code)]
struct Thread {
    id: usize,          // 线程ID
    stack: Vec<u8>,     // 线程运行所需要的栈
    ctx: TaskContext,   // 线程被换下时的上下文
    state: ThreadState, // 线程状态
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct TaskContext {
    // 15 u64
    x1: u64,  //ra: return addres
    x2: u64,  //sp
    x8: u64,  //s0: fp
    x9: u64,  //s1
    x18: u64, //x18-27: s2-11
    x19: u64,
    x20: u64,
    x21: u64,
    x22: u64,
    x23: u64,
    x24: u64,
    x25: u64,
    x26: u64,
    x27: u64,
    nx1: u64, //new return addres
}

impl Thread {
    fn new(id: usize) -> Self {
        // We initialize each task here and allocate the stack. This is not neccesary,
        // we can allocate memory for it later, but it keeps complexity down and lets us focus on more interesting parts
        // to do it here. The important part is that once allocated it MUST NOT move in memory.
        Thread {
            id,
            stack: alloc::vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: TaskContext::default(),
            state: ThreadState::Init,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        // This will be our base task, which will be initialized in the `running` state
        let base_task = Thread {
            id: 0,
            stack: alloc::vec![0u8; DEFAULT_STACK_SIZE],
            ctx: TaskContext::default(),
            state: ThreadState::Running,
        };

        // We initialize the rest of our tasks.
        let mut tasks = alloc::vec![base_task];
        let mut available_tasks: Vec<Thread> = (1..MAX_TASKS).map(|i| Thread::new(i)).collect();
        tasks.append(&mut available_tasks);

        let runtime = Runtime {
            workers: tasks,
            current: 0,
        };
        runtime.init();

        runtime
    }

    pub fn init(&self) {
        // 为全局变量赋值,这样所有进程都能够访问到它
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    /// This is where we start running our runtime. If it is our base task, we call yield until
    /// it returns false (which means that there are no tasks scheduled) and we are done.
    pub fn run(&mut self) {
        while self.t_yield() {}
        println!("All tasks finished!");
    }

    /// 销毁当前 worker 任务, 使得 worker 可复用
    fn t_return(&mut self) {
        // 只有非 0 worker 才是可 return (销毁并可复用) 的
        if self.current != 0 {
            self.workers[self.current].state = ThreadState::Init;
            // 调度下一个任务
            self.t_yield();
        }
    }

    /// This is the heart of our runtime. Here we go through all tasks and see if anyone is in the `Ready` state.
    /// If no task is `Ready` we're all done. This is an extremely simple scheduler using only a round-robin algorithm.
    ///
    /// If we find a task that's ready to be run we change the state of the current task from `Running` to `Ready`.
    /// Then we call switch which will save the current context (the old context) and load the new context
    /// into the CPU which then resumes based on the context it was just passed.
    ///
    /// NOITCE: if we comment below `#[inline(never)]`, we can not get the corrent running result
    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut cur = self.current;

        // 选择下一个可用线程,如果不存在则返回 false
        while self.workers[cur].state != ThreadState::Ready {
            cur += 1;
            if cur == self.workers.len() {
                cur = 0;
            }
            if cur == self.current {
                return false;
            }
        }

        // 把当前运行线程状态改为 Ready 就绪态
        if self.workers[self.current].state != ThreadState::Init {
            self.workers[self.current].state = ThreadState::Ready;
        }

        self.workers[cur].state = ThreadState::Running;
        let old_pos = self.current;
        self.current = cur;

        unsafe {
            switch(&mut self.workers[old_pos].ctx, &self.workers[cur].ctx);
        }
    }

    pub fn spawn(&mut self, f: fn()) {
        // 查看是否还有可用的任务槽位
        let worker = self
            .workers
            .iter_mut()
            .find(|t| t.state == ThreadState::Init)
            .expect("no available worker."); // 为了简便起见, 没有空闲 worker 直接报错

        let size = worker.stack.len();
        unsafe {
            let s_ptr = worker.stack.as_mut_ptr().offset(size as isize);

            // 8 字节低位对齐, 即 5 -> 0, 15 -> 8. 因为低内存区域是合法的
            let s_ptr = (s_ptr as usize & !7) as *mut u8;

            // old return address
            worker.ctx.x1 = guard as u64;
            // new return address
            worker.ctx.nx1 = f as u64;
            // sp
            worker.ctx.x2 = s_ptr.offset(-32) as u64;
        }
        worker.state = ThreadState::Ready;
    }
}

/// This is our guard function that we place on top of the stack. All this function does is set the
/// state of our current task and then `yield` which will then schedule a new task to be run.
fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_return();
    };
}

/// 回到 runtime, 调用 yield
pub fn yield_task() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    };
}

#[naked]
#[no_mangle]
unsafe extern "C" fn switch(old: *mut TaskContext, new: *const TaskContext) -> ! {
    // a0 是第一个参数, a1 是第二个参数
    // a0: _old, a1: _new
    asm!(
        "
        sd x1, 0x00(a0)
        sd x2, 0x08(a0)
        sd x8, 0x10(a0)
        sd x9, 0x18(a0)
        sd x18, 0x20(a0)
        sd x19, 0x28(a0)
        sd x20, 0x30(a0)
        sd x21, 0x38(a0)
        sd x22, 0x40(a0)
        sd x23, 0x48(a0)
        sd x24, 0x50(a0)
        sd x25, 0x58(a0)
        sd x26, 0x60(a0)
        sd x27, 0x68(a0)
        sd x1, 0x70(a0)

        ld x1, 0x00(a1)
        ld x2, 0x08(a1)
        ld x8, 0x10(a1)
        ld x9, 0x18(a1)
        ld x18, 0x20(a1)
        ld x19, 0x28(a1)
        ld x20, 0x30(a1)
        ld x21, 0x38(a1)
        ld x22, 0x40(a1)
        ld x23, 0x48(a1)
        ld x24, 0x50(a1)
        ld x25, 0x58(a1)
        ld x26, 0x60(a1)
        ld x27, 0x68(a1)
        ld t0, 0x70(a1)

        jr t0
    ",
        options(noreturn)
    );
}

#[no_mangle]
pub fn main() {
    println!("stackful_coroutine begin...");
    println!("TASK  0(Runtime) STARTING");
    let mut runtime = Runtime::new();
    runtime.spawn(|| {
        println!("TASK  1 STARTING");
        let id = 1;
        for i in 0..4 {
            println!("task: {} counter: {}", id, i);
            sleep(100);
            yield_task();
        }
        println!("TASK 1 FINISHED");
    });
    runtime.spawn(|| {
        println!("TASK 2 STARTING");
        let id = 2;
        for i in 0..8 {
            println!("task: {} counter: {}", id, i);
            sleep(100);
            yield_task();
        }
        println!("TASK 2 FINISHED");
    });
    runtime.spawn(|| {
        println!("TASK 3 STARTING");
        let id = 3;
        for i in 0..12 {
            println!("task: {} counter: {}", id, i);
            sleep(100);
            yield_task();
        }
        println!("TASK 3 FINISHED");
    });
    runtime.spawn(|| {
        println!("TASK 4 STARTING");
        let id = 4;
        for i in 0..16 {
            println!("task: {} counter: {}", id, i);
            sleep(100);
            yield_task();
        }
        println!("TASK 4 FINISHED");
    });
    runtime.run();
    println!("stackful_coroutine PASSED");
    exit(0);
}
