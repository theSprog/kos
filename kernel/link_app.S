.align 3    # 按照 2^3 = 8字节 对齐
    .section .data
    .global _num_app
_num_app:
    .quad 13
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_6_start
    .quad app_7_start
    .quad app_8_start
    .quad app_9_start
    .quad app_10_start
    .quad app_11_start
    .quad app_12_start
    .quad app_12_end

    .global _app_names
_app_names:
    .string "user/prog/cat"
    .string "user/prog/echo"
    .string "user/prog/fw"
    .string "user/prog/init"
    .string "user/prog/ls"
    .string "user/prog/mkdir"
    .string "user/prog/mv"
    .string "user/prog/pwd"
    .string "user/prog/rm"
    .string "user/prog/rw"
    .string "user/prog/shell"
    .string "user/prog/tc"
    .string "user/prog/touch"

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "user/prog/cat"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "user/prog/echo"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "user/prog/fw"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "user/prog/init"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
    .align 3
app_4_start:
    .incbin "user/prog/ls"
app_4_end:

    .section .data
    .global app_5_start
    .global app_5_end
    .align 3
app_5_start:
    .incbin "user/prog/mkdir"
app_5_end:

    .section .data
    .global app_6_start
    .global app_6_end
    .align 3
app_6_start:
    .incbin "user/prog/mv"
app_6_end:

    .section .data
    .global app_7_start
    .global app_7_end
    .align 3
app_7_start:
    .incbin "user/prog/pwd"
app_7_end:

    .section .data
    .global app_8_start
    .global app_8_end
    .align 3
app_8_start:
    .incbin "user/prog/rm"
app_8_end:

    .section .data
    .global app_9_start
    .global app_9_end
    .align 3
app_9_start:
    .incbin "user/prog/rw"
app_9_end:

    .section .data
    .global app_10_start
    .global app_10_end
    .align 3
app_10_start:
    .incbin "user/prog/shell"
app_10_end:

    .section .data
    .global app_11_start
    .global app_11_end
    .align 3
app_11_start:
    .incbin "user/prog/tc"
app_11_end:

    .section .data
    .global app_12_start
    .global app_12_end
    .align 3
app_12_start:
    .incbin "user/prog/touch"
app_12_end:
