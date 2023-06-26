use crate::{
    clock,
    memory::page_table,
    process::{
        processor::{
            self,
            api::{current_pcb, current_pid},
        },
        scheduler,
    },
};
use alloc::{sync::Arc, vec::Vec};
use logger::*;

/// processor exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!(
        "process-{} exited with code {}",
        processor::api::current_pid(),
        exit_code
    );
    processor::api::exit_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_sched_yield() -> isize {
    // 处理方式就是挂起当前，并且运行下一个
    processor::api::suspend_and_run_next();
    0
}

// 以毫秒的形式返回
pub fn sys_get_time_of_day() -> isize {
    clock::get_time_ms() as isize
}

pub fn sys_sbrk(incrment: usize) -> isize {
    processor::api::sbrk(incrment) as isize
}

pub fn sys_getpid() -> isize {
    current_pid() as isize
}

pub fn sys_fork() -> isize {
    let pcb = current_pcb().unwrap();
    let new_pcb = pcb.fork();
    let new_pid = new_pcb.getpid();
    let trap_cx = new_pcb.ex_inner().trap_cx();

    // 子进程的返回值为 0
    trap_cx.x[10] = 0;
    scheduler::add_ready(new_pcb);

    // 父进程返回子进程的 pid
    new_pid as isize
}

pub fn sys_execve(filename: *const u8, args: *const *const u8, envs: *const *const u8) -> isize {
    let pid = processor::api::current_pid();
    let token = processor::api::current_user_token();
    let app_name = page_table::api::translated_user_cstr(token, filename);
    let args = {
        let mut vec = Vec::new();
        let mut args_ptr = args;
        loop {
            let ptr = page_table::api::translated_ref(token, args_ptr);
            if ptr.is_null() {
                // 到达末尾
                break;
            }
            let arg = page_table::api::translated_user_cstr(token, *ptr);
            vec.push(arg);
            args_ptr = unsafe { args_ptr.add(1) };
        }
        vec
    };
    let envs = {
        let mut vec = Vec::new();
        let mut envs_ptr = envs;
        loop {
            let ptr = page_table::api::translated_ref(token, envs_ptr);
            if ptr.is_null() {
                // 到达末尾
                break;
            }
            let env = page_table::api::translated_user_cstr(token, *ptr);
            vec.push(env);
            envs_ptr = unsafe { envs_ptr.add(1) };
        }
        vec
    };

    debug!("pid = {}, args: {:?}", pid, args);
    debug!("pid = {}, envs: {:?}", pid, envs);
    let pcb = current_pcb().unwrap();
    pcb.exec(&app_name, args, envs)
}

/// 如果当前的进程不存在一个进程 ID 为 pid（pid==-1 或 pid > 0）的子进程，则返回 -1；
/// 如果存在一个进程 ID 为 pid 的僵尸子进程，
/// 则正常回收并返回子进程的 pid，并更新系统调用的退出码参数为 exit_code
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let pcb = current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    if !inner
        .children()
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        // 孩子中不存在所指定 pid 的进程
        return -1;
    }

    // 确实存在要等待的进程(也有可能参数 pid == -1 从而等待任意一个进程)
    // 注意一次只回收一个进程, enumerate 逐个枚举
    let pair = inner.children().iter().enumerate().find(|(_, p)| {
        // 僵尸进程，并且是指定 pid 的进程(pid == -1 表示任意一个进程)
        p.ex_inner().is_zombie() && (pid == -1 || pid as usize == p.getpid())
    });

    if let Some((idx, _)) = pair {
        // remove 会获取所有权
        let child = inner.children_mut().remove(idx);
        // 确保它的引用计数只有 1
        assert_eq!(Arc::strong_count(&child), 1);

        let found_pid = child.getpid();
        let exit_code = child.ex_inner().exit_code();

        // 以可变引用的方式取得用户空间 exit_code_ptr 对应的的地址
        // 并在该处写上 exit_code
        let user_exit_code_ptr =
            page_table::api::translated_refmut(inner.tcb().address_space.token(), exit_code_ptr);
        // 设定好 exit_code
        *user_exit_code_ptr = exit_code;
        found_pid as isize
    } else {
        // 不是僵尸进程(进程未结束), 或者指定 pid 的进程不存在
        -2
    }
}
