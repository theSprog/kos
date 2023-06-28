use alloc::sync::Arc;
use logger::info;

use crate::fs::{
    inode::{OSInode, OpenFlags},
    UserBuffer, VFS,
};
use crate::{memory::page_table, process::processor};

/// 由于内核和应用地址空间的隔离， sys_write 不再能够直接访问位于应用空间中的数据，
/// 而需要手动查页表才能知道那些数据被放置在哪些物理页帧上并进行访问
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = processor::api::current_user_token();
    let pcb = processor::api::current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    let tcb = inner.tcb();
    if fd >= tcb.fd_table.len() {
        return -1;
    }

    if let Some(file) = &tcb.fd_table[fd] {
        if !file.writable() {
            return -1;
        }

        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::new(page_table::api::translated_byte_buffer(
            token, buf, len,
        )))
        .unwrap() as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = processor::api::current_user_token();
    let pcb = processor::api::current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    let tcb = inner.tcb();
    if fd >= tcb.fd_table.len() {
        return -1;
    }

    if let Some(file) = &tcb.fd_table[fd] {
        // 不可读
        if !file.readable() {
            return -1;
        }

        let file = file.clone();

        drop(inner);
        file.read(UserBuffer::new(page_table::api::translated_byte_buffer(
            token, buf, len,
        )))
        .unwrap() as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let tcb = processor::api::current_tcb();
    let token = processor::api::current_user_token();
    let path = page_table::api::translated_user_cstr(token, path);
    let flags = OpenFlags::from_bits(flags).unwrap();
    let (create, trancate) = (flags.create(), flags.truncate());

    //TODO: create & trancate

    if let Ok(inode) = VFS.open_file(path.as_str()) {
        let fd = tcb.alloc_fd();
        tcb.fd_table[fd] = Some(Arc::new(OSInode::new(flags.read(), flags.write(), inode)));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_listdir(path: *const u8) -> isize {
    let token = processor::api::current_user_token();
    let path = page_table::api::translated_user_cstr(token, path);
    let dir = VFS.read_dir(path.as_str());
    if dir.is_err() {
        return -1;
    }

    use crate::println;
    use component::util::human_size::bin_size;
    use component::util::time::LocalTime;
    let dir = dir.unwrap();
    println!(
        "{:>5} {:>11} {:>5} {:>10} {:>5} {:>5} {:>19} {}",
        "Inode", "Permissions", "Links", "Size", "UID", "GID", "Modified Time", "Name"
    );

    for entry in dir {
        let metadata = entry.inode().metadata();
        let name = if metadata.filetype().is_symlink() {
            alloc::format!(
                "{} -> {}",
                entry.name(),
                entry.inode().read_symlink().unwrap()
            )
        } else {
            alloc::format!("{}", entry.name())
        };

        let size_str = alloc::format!("{}", bin_size(metadata.size() as usize));
        println!(
            "{:>5}  {}{} {:>5} {:>10} {:>5} {:>5} {:>19} {}",
            entry.inode_id(),
            metadata.filetype(),
            metadata.permissions(),
            metadata.hard_links(),
            size_str,
            metadata.uid(),
            metadata.gid(),
            LocalTime::from_posix(metadata.timestamp().mtime()),
            name
        );
    }
    0
}

pub fn sys_ftruncate(fd: usize, length: usize) -> isize {
    let pcb = processor::api::current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    let tcb = inner.tcb();
    // fd 越界
    if fd >= tcb.fd_table.len() {
        return -1;
    }
    if let Some(file) = &tcb.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(inner);
        let res = file.truncate(length);
        info!("res {:?}", res);

        return match res {
            Ok(_) => 0 as isize,
            Err(_) => -1 as isize,
        };
    }
    // fd 是 none
    -1
}

pub fn sys_close(fd: usize) -> isize {
    let tcb = processor::api::current_tcb();
    if fd >= tcb.fd_table.len() {
        return -1;
    }
    if tcb.fd_table[fd].is_none() {
        return -1;
    }
    // 所有权取出, 将 None 置入
    tcb.fd_table[fd].take();
    0
}

pub fn sys_io_destroy(args0: usize, args1: usize, args2: usize) -> isize {
    0
}
