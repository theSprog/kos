use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use component::fs::vfs::{
    meta::{VfsFileType, VfsPermissions},
    VfsError, VfsErrorKind,
};
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

fn build_abs_path(path: *const u8) -> String {
    let token = processor::api::current_user_token();
    let path = page_table::api::translated_user_cstr(token, path);
    match path.starts_with('/') {
        true => path,
        false => {
            let pcb = processor::api::current_pcb().unwrap();
            let mut pwd = pcb.ex_inner().cwd().clone();
            pwd.forward(&path);
            pwd.to_string()
        }
    }
}

fn report_fs_err(err: VfsError) {
    match err.kind() {
        VfsErrorKind::IOError(io_err) => {
            info!("IOError: {:?}", io_err);
        }
        VfsErrorKind::FileNotFound => {
            info!("File not found");
        }
        VfsErrorKind::InvalidPath(path) => {
            info!("Invalid path: {:?}", path);
        }
        VfsErrorKind::DirectoryExists => {
            info!("Directory exists");
        }
        VfsErrorKind::FileExists => {
            info!("File exists");
        }
        VfsErrorKind::NotSupported => {
            info!("Not supported");
        }
        _ => {
            info!("Unknown error");
        }
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let path = build_abs_path(path);

    let tcb = processor::api::current_tcb();
    let flags = OpenFlags::from_bits(flags).unwrap();
    let (create, trancate) = (flags.create(), flags.truncate());

    let res = VFS.open_file(path.as_str());

    if let Ok(mut inode) = res {
        if trancate {
            let res = inode.set_len(0);
            if res.is_err() {
                report_fs_err(res.unwrap_err());
                return -1;
            }
        }
        let fd = tcb.alloc_fd();
        tcb.fd_table[fd] = Some(Arc::new(OSInode::new(flags.read(), flags.write(), inode)));
        fd as isize
    } else {
        // open 失败可能是因为不存在文件
        if create {
            let inode = VFS.create_file(path.as_str());
            if inode.is_ok() {
                let mut inode = inode.unwrap();
                inode.set_permissions(&0o666.into());
                let fd = tcb.alloc_fd();
                tcb.fd_table[fd] = Some(Arc::new(OSInode::new(flags.read(), flags.write(), inode)));
                return fd as isize;
            }
            // 出错
            report_fs_err(inode.unwrap_err());
        }
        report_fs_err(res.unwrap_err());
        -1
    }
}

pub fn sys_listdir(path: *const u8) -> isize {
    let abs_path = build_abs_path(path);

    let dir = VFS.read_dir(&abs_path);
    if dir.is_err() {
        report_fs_err(dir.unwrap_err());
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
        let colored_name = match metadata.filetype() {
            VfsFileType::RegularFile => {
                if metadata.permissions().user().execute() {
                    alloc::format!("\x1b[32m{}\x1b[0m", entry.name())
                } else {
                    entry.name().to_string()
                }
            }
            VfsFileType::Directory => alloc::format!("\x1b[94m{}\x1b[0m", entry.name()),
            VfsFileType::SymbolicLink => alloc::format!("\x1b[36m{}\x1b[0m", entry.name()),

            VfsFileType::FIFO => todo!(),

            VfsFileType::CharDev => todo!(),
            VfsFileType::BlockDev => todo!(),
            VfsFileType::Socket => todo!(),
        };

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
            colored_name
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

pub fn sys_mkdirat(path: *const u8, mode: usize) -> isize {
    let abs_path = build_abs_path(path);

    if mode > u16::MAX as usize {
        return -1;
    }
    let permissions = VfsPermissions::new(mode as u16);
    let res = VFS.create_dir(abs_path.as_str());
    if res.is_err() {
        report_fs_err(res.unwrap_err());
        return -1;
    }
    let mut dir = res.unwrap();
    dir.set_permissions(&permissions);
    0
}

pub fn sys_chdir(path: *const u8) -> isize {
    let token = processor::api::current_user_token();
    let path = page_table::api::translated_user_cstr(token, path);
    let pcb = processor::api::current_pcb().unwrap();
    match &path[..] {
        "." => 0,
        ".." => {
            let mut inner = pcb.ex_inner();
            let cwd = inner.cwd_mut();
            cwd.backward();
            0
        }
        path => {
            let mut inner = pcb.ex_inner();
            let cwd = inner.cwd_mut();
            if path.starts_with('/') {
                let meta = VFS.metadata(path.to_string()).unwrap();
                if !meta.filetype().is_dir() {
                    return -1;
                }
                cwd.replace(path);
            } else {
                cwd.forward(path);
                let meta = VFS.metadata(cwd.to_string()).unwrap();
                if !meta.filetype().is_dir() {
                    cwd.backward();
                    return -1;
                }
            }

            0
        }
    }
}

pub fn sys_getcwd(buffer: *mut u8, max_len: usize) -> isize {
    let token = processor::api::current_user_token();
    let pcb = processor::api::current_pcb().unwrap();
    let inner = pcb.ex_inner();
    let cwd_string = inner.cwd().to_string();
    if cwd_string.len() > max_len {
        return -1;
    }

    let mut user_buffer = UserBuffer::new(page_table::api::translated_byte_buffer(
        token, buffer, max_len,
    ));

    let mut cwd_begin = 0;
    for slice in user_buffer.iter_mut() {
        let slice_len = slice.len();
        let rest_cwd_len = cwd_string.len() - cwd_begin;
        // 1. 如果此 slice 足以容纳剩余 cwd_string
        // 2. 如果剩余 cwd_string 大于 slice, 则还需要下一次填充
        let this_len = rest_cwd_len.min(slice_len);
        let src = &cwd_string.as_bytes()[cwd_begin..cwd_begin + this_len];
        let dst = &mut slice[..this_len];
        dst.copy_from_slice(src);
        cwd_begin += src.len();
    }

    0
}

pub fn sys_unlinkat(path: *const u8) -> isize {
    let abs_path = build_abs_path(path);

    let res1 = VFS.remove_dir(abs_path.as_str());
    let res2 = VFS.remove_file(abs_path.as_str());
    match (&res1, &res2) {
        (Ok(_), Ok(_)) => panic!("Removed directory and file: '{}'", abs_path),
        (Ok(_), Err(_)) => 0,
        (Err(_), Ok(_)) => 0,
        (Err(_), Err(_)) => {
            // 两个都出错, 说明即不存在文件也不存在目录
            report_fs_err(res1.unwrap_err());
            report_fs_err(res2.unwrap_err());
            return -1;
        }
    }
}

pub fn sys_linkat(to: *const u8, from: *const u8) -> isize {
    let abs_to = build_abs_path(to);
    let abs_from = build_abs_path(from);
    let res = VFS.link(abs_to.as_str(), abs_from.as_str());
    if res.is_err() {
        report_fs_err(res.unwrap_err());
        return -1;
    }
    0
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

pub fn sys_io_destroy(_args0: usize, _args1: usize, _args2: usize) -> isize {
    0
}
