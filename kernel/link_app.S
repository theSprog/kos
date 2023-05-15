
    .align 3    # 按照 2^3 = 8字节 对齐
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
        .align 3
    app_0_start:
        .incbin "./user/prog/stack_deep"
    app_0_end:

        .section .data
        .global app_1_start
        .global app_1_end
        .align 3
    app_1_start:
        .incbin "./user/prog/write_a"
    app_1_end:

        .section .data
        .global app_2_start
        .global app_2_end
        .align 3
    app_2_start:
        .incbin "./user/prog/write_b"
    app_2_end:

        .section .data
        .global app_3_start
        .global app_3_end
        .align 3
    app_3_start:
        .incbin "./user/prog/write_c"
    app_3_end:
