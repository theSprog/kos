.altmacro

.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm

    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # 根据调用规范, a0 保存 current_task_cx_ptr, a1 保存 next_task_cx_ptr

    # 保存当前任务的现场
    # ra -> [a0 + 0] (即 current_task_cx_ptr.ra)
    sd ra, 0(a0)
    # sp -> [a0 + 8] (即 current_task_cx_ptr.sp)
    sd sp, 8(a0)
    # s0~s11 -> [a0 + n] ( 即 current_task_cx_ptr.s)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr


    # 恢复即将要执行的任务的现场
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # [a1 + 8] -> sp
    ld sp, 8(a1)
    # [a1 + 0] -> ra
    ld ra, 0(a1)

    # 返回到即将要执行的任务
    ret