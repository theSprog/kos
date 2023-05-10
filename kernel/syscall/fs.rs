use crate::print;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let slice = unsafe { core::slice::from_raw_parts(buf, len) };
    let str = core::str::from_utf8(slice).unwrap();

    match fd {
        FD_STDOUT => {
            // 向控制台写字符串
            crate::console::print(format_args!("{}", str));
            len as isize // 返回写入的字符个数
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
