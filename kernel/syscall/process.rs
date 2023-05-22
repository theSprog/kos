use logger::*;

use crate::{
    process::{
        processor::{
            self,
            api::{current_pcb, current_pid},
        },
        scheduler,
    },
    timer,
};

/// processor exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!(
        "process-{} exited with code {}",
        processor::api::current_pid(),
        exit_code
    );
    processor::api::exit_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_sched_yield() -> isize {
    // 处理方式就是挂起当前，并且运行下一个
    processor::api::suspend_and_run_next();
    0
}

// 以毫秒的形式返回
pub fn sys_get_time_of_day() -> isize {
    timer::get_time_ms() as isize
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
    let trap_cx = new_pcb.inner().trap_cx();

    // 子进程的返回值为 0
    trap_cx.x[10] = 0;
    scheduler::add_ready(new_pcb);

    // 父进程返回子进程的 pid
    new_pid as isize
}

pub fn sys_exec(prog: *const u8) -> isize {
    todo!()
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    todo!()
}

// pub fn sys_exec(path: *const u8) -> isize {
//     let token = current_user_token();
//     let path = translated_str(token, path);
//     if let Some(data) = get_app_data_by_name(path.as_str()) {
//         let task = current_task().unwrap();
//         task.exec(data);
//         0
//     } else {
//         -1
//     }
// }

// /// If there is not a child process whose pid is same as given, return -1.
// /// Else if there is a child process but it is still running, return -2.
// pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
//     let task = current_task().unwrap();
//     // find a child process

//     // ---- access current TCB exclusively
//     let mut inner = task.inner_exclusive_access();
//     if !inner
//         .children
//         .iter()
//         .any(|p| pid == -1 || pid as usize == p.getpid())
//     {
//         return -1;
//         // ---- release current PCB
//     }
//     let pair = inner.children.iter().enumerate().find(|(_, p)| {
//         // ++++ temporarily access child PCB lock exclusively
//         p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
//         // ++++ release child PCB
//     });
//     if let Some((idx, _)) = pair {
//         let child = inner.children.remove(idx);
//         // confirm that child will be deallocated after removing from children list
//         assert_eq!(Arc::strong_count(&child), 1);
//         let found_pid = child.getpid();
//         // ++++ temporarily access child TCB exclusively
//         let exit_code = child.inner_exclusive_access().exit_code;
//         // ++++ release child PCB
//         *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
//         found_pid as isize
//     } else {
//         -2
//     }
//     // ---- release current PCB lock automatically
// }
