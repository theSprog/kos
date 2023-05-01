.altmacro   # .altmacro 才能正常使用 .rept 命令
.macro SAVE_GP n
    sd x\n, \n*8(sp)    # 保存寄存器 xn 到 sp[n] 的宏
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)    # 从 sp[n] 中恢复寄存器 xn 的宏
.endm

    .section .text
    .globl __alltraps
    .globl __restore
    # 4 字节对齐，这是 RISC-V 特权级规范的要求
    .align 2
__alltraps:
    # 先将 sscratch 读入 sp 中，然后将 sp 的原先值读入 sscratch 中. 简而言之就是交换 sp 和 sscratch 的值
    # 在这一行之前:sp 指向用户栈，sscratch 指向内核栈
    csrrw sp, sscratch, sp  
    # 现在: sp 指向内核栈，sscratch 指向用户栈
    # 分配一个 TrapContext 的内容 : 32 个通用寄存器 + sepc + Sstatus
    addi sp, sp, -34*8

    # 保存通用寄存器
    # x0 不用保存，因为它永远是 0
    sd x1, 1*8(sp)
    # 跳过 sp(x2), 之后手动保存, 因为现在 sp 指向的是内核栈，而我们要保存用户栈(sscratch)
    # 我们不能直接将 sscratch 保存至内存中而必须通过通用寄存器中转
    # 而现在通用寄存器正是在被保存的过程中, 所以对 sp 的处理专门放到之后
    sd x3, 3*8(sp)
    # 跳过 tp(x4), 因为它从不被 app 使用
    # 保存 x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # 将 sstatus 和 sepc 读入并且保存到 TrapContext 中
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # 现在处理 sp 的问题，sscratch 由于之前的交换所以指向用户栈
    # 经过 t2 中转并存入 TrapContext 内存区域中
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 至此用户态上下文被完全保存

    # 将 sp 写入 a0, 这因为 trap_handler 的第一个参数要从 a0 中读取
    # 而 sp 现在所指向的正是 TrapContext,
    # 因此我们相当于手动构造了一个 TrapContext 参数并传入 trap_handler 中
    mv a0, sp
    # 需要 Trap 上下文的原因在于：trap_handler 需要知道用户态传过来的某些寄存器的值
    # 例如最基本的 syscall 调用号和对应参数
    call trap_handler

# 从 trap_handler 返回后就会直接顺势从 __restore 继续执行, 我们用此恢复上下文
__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0
    # 目前 sp 指向 kernel-stack, 而 sscratch 指向 user-stack
    # 每恢复一个寄存器，该寄存器就不能再使用，因此需要先恢复 CSR 寄存器
    ld t0, 32*8(sp) # sstatus
    ld t1, 33*8(sp) # sepc
    ld t2, 2*8(sp)  # sscratch
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # 恢复通用寄存器
    ld x1, 1*8(sp)
    # 跳过 sp(x2) 之后处理
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 释放 TrapContext 上下文空间
    addi sp, sp, 34*8
    # 目前 sp 指向内核栈, sscratch 指向用户栈
    # 再次交换 sp 和 sscratch
    csrrw sp, sscratch, sp
    sret
