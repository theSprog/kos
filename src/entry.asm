    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top
    call entry

    .section .bss.stack
    .globl boot_stack_upper_bound
boot_stack_upper_bound:
    .space 4096 * 16
    .globl boot_stack_lower_bound
    .globl boot_stack_top
boot_stack_lower_bound:
boot_stack_top: