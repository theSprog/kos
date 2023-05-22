.align 3    # 按照 2^3 = 8字节 对齐
    .section .data
    .global _num_app
_num_app:
    .quad 1
    .quad app_0_start
    .quad app_0_end

    .global _app_names
_app_names:
    .string "init"

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "./user/prog/init"
app_0_end:
