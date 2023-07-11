use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use component::fs::vfs::VfsError;
use spin::Mutex;

use crate::process::processor::api::suspend_and_run_next;

use super::File;

pub struct Pipe {
    readable: bool,
    writable: bool,

    // 每个读端或写端中都保存着所属管道自身的强引用计数
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    // 设置 buffer 读端
    pub fn read_end(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    // 设置 buffer 写端
    pub fn write_end(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}

const RING_BUFFER_SIZE: usize = 4096;

#[derive(Debug, Copy, Clone, PartialEq)]
enum RingBufferStatus {
    Full,  // FULL 表示缓冲区已满不能再继续写入
    Empty, // EMPTY 表示缓冲区为空无法从里面读取
    Normal,
}

pub struct PipeRingBuffer {
    // arr/head/tail 三个字段用来维护一个循环队列
    arr: Vec<u8>,
    head: usize,
    tail: usize,

    status: RingBufferStatus,
    write_end: Option<Weak<Pipe>>, // 某些下需要确认该管道所有的写端是否都已经被关闭
    read_end: Option<Weak<Pipe>>,  // 某些下需要确认该管道所有的读端是否都已经被关闭
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: alloc::vec![0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::Empty,
            write_end: None,
            read_end: None,
        }
    }

    // 关联读写端, 但是 weak ref
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
    }
    pub fn set_read_end(&mut self, read_end: &Arc<Pipe>) {
        self.read_end = Some(Arc::downgrade(read_end));
    }

    // 从管道中读一个 byte, 注意在调用它之前需要确保管道缓冲区中不是空的。
    // 有可能会使状态变 EMPTY
    pub fn read_byte(&mut self) -> u8 {
        assert_ne!(self.status, RingBufferStatus::Empty);

        self.status = RingBufferStatus::Normal;
        let byte = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::Empty;
        }
        byte
    }

    pub fn write_byte(&mut self, byte: u8) {
        assert_ne!(self.status, RingBufferStatus::Full);

        self.status = RingBufferStatus::Normal;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::Full;
        }
    }

    // 返回剩余可读字节数
    pub fn rest_read(&self) -> usize {
        match self.status {
            RingBufferStatus::Empty => 0,
            RingBufferStatus::Full | RingBufferStatus::Normal => {
                if self.tail > self.head {
                    self.tail - self.head
                } else {
                    self.tail + RING_BUFFER_SIZE - self.head
                }
            }
        }
    }

    // 返回剩余可写字节数
    pub fn rest_write(&self) -> usize {
        match self.status {
            RingBufferStatus::Full => 0,
            RingBufferStatus::Empty | RingBufferStatus::Normal => {
                RING_BUFFER_SIZE - self.rest_read()
            }
        }
    }

    // 如果所有的写端都关闭, 从而管道中的数据不会再得到补充,
    // 待管道中仅剩的数据被读取完毕之后，管道就可以被销毁了
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
    // 如果读端都被关闭, 数据就再也不可能被读取, 需要销毁此管道
    pub fn all_read_ends_closed(&self) -> bool {
        self.read_end.as_ref().unwrap().upgrade().is_none()
    }
}

// Pipe 也是一个文件
impl File for Pipe {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }

    // 从 pipe 读若干个字节进入 user_buffer 中
    fn read(&self, buf: super::UserBuffer) -> Result<usize, VfsError> {
        assert!(self.readable());
        let want_to_read = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut already_read = 0usize;
        loop {
            // 消费数据时禁止写入
            let mut ring_buffer = self.buffer.lock();
            match ring_buffer.rest_read() {
                // 无数据可读
                0 => {
                    // 有可能是因为所有写端已经关闭
                    if ring_buffer.all_write_ends_closed() {
                        return Ok(already_read);
                    }

                    // 否则放弃锁, 让其他进程生产数据
                    // 其他进程有可能要使用 ring_buffer, 必须提前 drop 锁
                    drop(ring_buffer);
                    suspend_and_run_next();
                    // 返回本进程后继续查看是否有数据可读
                    continue;
                }

                // 有 n 字节可读
                n => {
                    for _ in 0..n {
                        match buf_iter.next() {
                            Some(byte_ptr) => {
                                // 从管道中读数据
                                unsafe {
                                    *byte_ptr = ring_buffer.read_byte();
                                }
                                already_read += 1;
                            }
                            None => {
                                // 到此处说明 userbuf 已满
                                assert_eq!(already_read, want_to_read);
                                return Ok(already_read);
                            }
                        }
                    }

                    // 到此处说明 ring_buffer 已空(n 字节全部消费), 但 user_buffer 未填满
                }
            }
        }
    }

    fn write(&self, buf: super::UserBuffer) -> Result<usize, VfsError> {
        assert!(self.writable());
        let want_to_write = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut already_write = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            match ring_buffer.rest_write() {
                0 => {
                    // 没有可写的位置是因为没有消费者
                    if ring_buffer.all_read_ends_closed() {
                        return Ok(already_write);
                    }
                    drop(ring_buffer);
                    suspend_and_run_next();
                    continue;
                }

                // 最多可写 n 字节
                n => {
                    for _ in 0..n {
                        if let Some(byte_ptr) = buf_iter.next() {
                            ring_buffer.write_byte(unsafe { *byte_ptr });
                            already_write += 1;
                        } else {
                            // userbuf 已满
                            assert_eq!(already_write, want_to_write);
                            return Ok(already_write);
                        }
                    }
                    // ringbuffer 中的数据全部被消费
                }
            }
        }
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let read_end = Arc::new(Pipe::read_end(buffer.clone()));
    let write_end = Arc::new(Pipe::write_end(buffer.clone()));
    buffer.lock().set_write_end(&write_end);
    buffer.lock().set_read_end(&read_end);
    (read_end, write_end)
}
