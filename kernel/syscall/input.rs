use crate::driver::chardev::UART;

pub fn sys_key_pressed() -> isize {
    let res = !UART.read_buffer_is_empty();
    if res {
        1
    } else {
        0
    }
}
