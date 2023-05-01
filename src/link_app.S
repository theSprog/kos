
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 4
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_3_end

    .section .data
    .global app_0_start
    .global app_0_end
app_0_start:
    .incbin "./user/bin/hello.bin"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
app_1_start:
    .incbin "./user/bin/power.bin"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
app_2_start:
    .incbin "./user/bin/priv_inst.bin"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
app_3_start:
    .incbin "./user/bin/store_fault.bin"
app_3_end:
