.altmacro   # .altmacro 才能正常使用 .rept 命令
.macro SAVE_GP n
    sd x\n, \n*8(sp)    # 保存寄存器 xn 到 sp[n] 的宏
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)    # 从 sp[n] 中恢复寄存器 xn 的宏
.endm

    # 将 trap.S 中的整段汇编代码放置在 .text.trampoline 段
    # 在链接时此处的首地址便是 strampoline
    # 这样，这段汇编代码放在一个物理页帧中，
    # 且 __alltraps 恰好位于这个物理页帧的开头，其物理地址被外部符号 strampoline 标记
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    # 4(2^2) 字节对齐，这是 RISC-V 特权级规范的要求
    .align 2

    # trap 作用: 保存和恢复 Trap 上下文, 切换地址空间(应用地址空间到内核地址空间)
__alltraps:
    # 先将 sscratch 读入 sp 中，然后将 sp 的原先值读入 sscratch 中. 简而言之就是交换 sp 和 sscratch 的值
    # 在这一行之前: sp 指向用户栈，sscratch 指向应用空间的 TrapContext
    csrrw sp, sscratch, sp  
    # 现在: sp 指向应用空间的 TrapContext，sscratch 指向用户栈

    # 保存通用寄存器
    # x0 不用保存，因为它永远是 0
    sd x1, 1*8(sp)
    # 跳过 sp(x2), 之后手动保存, 因为现在 sp 指向应用空间的 TrapContext
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

    # 现在能够使用 t0/t1/t2 因为他们已经被保存起来了
    # 将 sstatus 和 sepc 读入并且保存到 TrapContext 中
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # 现在处理 sp 的问题，sscratch 由于之前的交换所以指向用户栈
    # 经过 t2 中转并存入 TrapContext 内存区域中
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 现在用户栈指针已被保存

    # 加载 kernel_satp 到 t0
    ld t0, 34*8(sp)
    # 加载 trap_handler 到 t1
    ld t1, 36*8(sp)
    # 通过对 sp 赋值使得栈指针转移到内核栈
    ld sp, 35*8(sp)
    # 至此用户态上下文被完全保存

    # 切换到内核页表，从而实现地址空间转换
    csrw satp, t0
    sfence.vma

    # t1 指向 trap_handler
    jr t1

# 从 trap_handler 返回后就会直接顺势从 __restore 继续执行, 我们用此恢复上下文
__restore:
    # a0: TrapContext in user space(Constant); 
    # a1: user space token
    csrw satp, a1
    sfence.vma
    # 地址空间切换完毕

    csrw sscratch, a0
    mv sp, a0
    # 目前 sp 指向应用空间的 TrapContext, 开始恢复现场
    # 每恢复一个寄存器，该寄存器就不能再使用，因此需要先恢复 CSR 寄存器
    ld t0, 32*8(sp) # sstatus
    ld t1, 33*8(sp) # sepc

    csrw sstatus, t0
    csrw sepc, t1

    # 恢复通用寄存器
    ld x1, 1*8(sp)
    # 跳过 sp(x2) 之后处理
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 返回用户栈
    ld sp, 2*8(sp)
    sret
