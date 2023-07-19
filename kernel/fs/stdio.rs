use component::chardev::CharDevice;

use crate::{io::UART, print, sbi::console_getchar, vfs::VfsError};

use super::{File, UserBuffer};

pub struct Stdin;
///Standard output
pub struct Stdout;
pub struct Stderr;

impl File for Stdin {
    // 标准输入可读不可写
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }

    fn read(&self, mut user_buf: UserBuffer) -> Result<usize, VfsError> {
        // assert_eq!(user_buf.len(), 1, "Only support len = 1 in FD_STDIN!");
        // //println!("before UART.read() in Stdin::read()");
        // let ch = UART.read();
        // unsafe {
        //     user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        // }
        // Ok(1)

        assert_eq!(user_buf.len(), 1);
        // busy loop
        let c: usize;
        c = console_getchar();
        assert_ne!(c, 0);
        let ch = c as u8;
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        Ok(1)
    }

    fn write(&self, _user_buf: UserBuffer) -> Result<usize, VfsError> {
        panic!("Why write to stdin?");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }

    fn read(&self, _user_buf: UserBuffer) -> Result<usize, VfsError> {
        panic!("Why read from stdout?");
    }

    fn write(&self, user_buf: UserBuffer) -> Result<usize, VfsError> {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        Ok(user_buf.len())
    }
}

impl File for Stderr {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }

    fn read(&self, _user_buf: UserBuffer) -> Result<usize, VfsError> {
        panic!("Why read from stderr?");
    }

    fn write(&self, user_buf: UserBuffer) -> Result<usize, VfsError> {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        Ok(user_buf.len())
    }
}
