#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, fork, get_time_ms, pipe, read, wait, write};

const LENGTH: usize = 0o114514;
#[no_mangle]
pub fn main() -> i32 {
    // create pipe
    let mut pipe_fd = [0usize; 2];
    pipe(&mut pipe_fd);

    let read_end = pipe_fd[0];
    let write_end = pipe_fd[1];

    let mut random_str = [0u8; LENGTH];
    for ch in random_str.iter_mut() {
        *ch = get_time_ms() as u8;
    }

    // 我们希望管道中的数据从父进程流向子进程
    if fork() == 0 {
        // 子进程读数据
        close(write_end);
        let mut offset = 0;
        loop {
            let mut buffer = [0u8; 4101];
            let len_read = read(read_end, &mut buffer) as usize;
            if len_read == 0 {
                break;
            };
            assert_eq!(&buffer[..len_read], &random_str[offset..offset + len_read]);
            offset += len_read;
        }
        // close read_end
        close(read_end);
        println!("Read OK, child process exited!");
        0
    } else {
        // 父进程写数据
        close(read_end);
        assert_eq!(write(write_end, &random_str), random_str.len() as isize);
        // close write end
        close(write_end);
        let mut child_exit_code: i32 = 0;
        // 等待子进程
        wait(&mut child_exit_code);
        assert_eq!(child_exit_code, 0);
        println!("pipetest passed!");
        0
    }
}
