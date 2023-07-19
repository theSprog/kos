use crate::io::UART;

pub fn sys_key_pressed() -> isize {
    let res = !UART.empty_read_buffer();
    if res {
        1
    } else {
        0
    }
}
