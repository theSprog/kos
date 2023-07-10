use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use component::fs::vfs::{
    meta::{VfsFileType, VfsPermissions},
    IOErrorKind, VfsError, VfsErrorKind,
};
use logger::info;
use sys_interface::syserr;

use crate::fs::{
    inode::{OSInode, OpenFlags},
    pipe,
    userbuf::UserBuffer,
    VFS,
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
        return syserr::EBADF;
    }

    if let Some(file) = &tcb.fd_table[fd] {
        if !file.writable() {
            return syserr::EBADF;
        }

        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::new(page_table::api::translated_byte_buffer(
            token, buf, len,
        )))
        .unwrap() as isize
    } else {
        syserr::EBADF
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = processor::api::current_user_token();
    let pcb = processor::api::current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    let tcb = inner.tcb();
    if fd >= tcb.fd_table.len() {
        return syserr::EBADF;
    }

    if let Some(file) = &tcb.fd_table[fd] {
        // 不可读
        if !file.readable() {
            return syserr::EBADF;
        }

        let file = file.clone();

        drop(inner);
        file.read(UserBuffer::new(page_table::api::translated_byte_buffer(
            token, buf, len,
        )))
        .unwrap() as isize
    } else {
        syserr::EBADF
    }
}

fn build_abs_path(path: *const u8) -> String {
    let token = processor::api::current_user_token();
    let path = page_table::api::translated_user_cstr(token, path);
    let ret_path = match path.starts_with('/') {
        true => path,
        false => {
            let pcb = processor::api::current_pcb().unwrap();
            let mut pwd = pcb.ex_inner().cwd().clone();
            pwd.forward(&path);
            pwd.to_string()
        }
    };
    info!("user abs path: {:?}", ret_path);
    ret_path
}

// report errors and return syserr
fn report_fs_err(err: VfsError) -> isize {
    match err.kind() {
        VfsErrorKind::IOError(io_err) => {
            info!("IOError: {:?}", io_err);
            match io_err.kind() {
                IOErrorKind::NotFound => syserr::ENOENT,
                IOErrorKind::PermissionDenied => syserr::EPERM,
                IOErrorKind::AlreadyExists => syserr::EEXIST,
                IOErrorKind::NotADirectory => syserr::ENOTDIR,
                IOErrorKind::NotAFile => syserr::EINVAL,
                IOErrorKind::NotASymlink => syserr::EINVAL,
                IOErrorKind::TooLongTargetSymlink => syserr::EINVAL,
                IOErrorKind::IsADirectory => syserr::EISDIR,
                IOErrorKind::TooLargeFile => syserr::EFBIG,
                IOErrorKind::TooLongFileName => syserr::ENAMETOOLONG,
                IOErrorKind::TooManyLinks => syserr::EMLINK,
                IOErrorKind::InvalidFilename => syserr::EINVAL,
                IOErrorKind::NoFreeBlocks => syserr::ENOSPC,
                IOErrorKind::NoFreeInodes => syserr::ENOSPC,
                IOErrorKind::BadSeek => syserr::ESPIPE,
            }
        }
        VfsErrorKind::FileNotFound => {
            info!("File not found");
            syserr::ENOENT
        }
        VfsErrorKind::InvalidPath(path) => {
            info!("Invalid path: {:?}", path);
            syserr::EINVAL
        }
        VfsErrorKind::DirectoryExists => {
            info!("Directory exists");
            syserr::EEXIST
        }
        VfsErrorKind::FileExists => {
            info!("File exists");
            syserr::EEXIST
        }
        VfsErrorKind::NotSupported => {
            info!("Not supported");
            syserr::EPERM
        }
        _ => {
            info!("Unknown error");
            todo!()
        }
    }
}

pub fn sys_open(path: *const u8, flags: u32, mode: u16) -> isize {
    let path = build_abs_path(path);

    let tcb = processor::api::current_tcb();
    let flags = OpenFlags::from_bits(flags).unwrap();
    let (create, trancate, append) = (flags.create(), flags.truncate(), flags.append());

    let res = VFS.open_file(path.as_str());
    if let Ok(mut inode) = res {
        if trancate {
            if let Err(err) = inode.set_len(0) {
                return report_fs_err(err);
            }
        }
        let fd = tcb.alloc_fd();
        let offset = inode.metadata().size() as usize;
        let os_inode = OSInode::new(flags.read(), flags.write(), inode);
        if append {
            os_inode.set_offset(offset);
        }
        tcb.fd_table[fd] = Some(Arc::new(os_inode));
        fd as isize
    } else {
        // open 失败可能是因为不存在文件
        if create {
            let inode = VFS.create_file(path.as_str());
            match inode {
                Ok(mut inode) => {
                    inode.set_permissions(&mode.into());
                    let fd = tcb.alloc_fd();
                    tcb.fd_table[fd] =
                        Some(Arc::new(OSInode::new(flags.read(), flags.write(), inode)));
                    return fd as isize;
                }
                Err(err) => {
                    return report_fs_err(err);
                }
            }
        }

        report_fs_err(res.unwrap_err())
    }
}

pub fn sys_listdir(path: *const u8) -> isize {
    let abs_path = build_abs_path(path);

    let dir = match VFS.read_dir(&abs_path) {
        Ok(dir) => dir,
        Err(err) => {
            return report_fs_err(err);
        }
    };

    use crate::println;
    use component::util::human_size::bin_size;
    use component::util::time::LocalTime;
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
            VfsFileType::Socket => entry.name().to_string(),
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
        return syserr::EBADF;
    }
    if let Some(file) = &tcb.fd_table[fd] {
        if !file.writable() {
            return syserr::EBADF;
        }
        let file = file.clone();
        drop(inner);
        let res = file.truncate(length);

        return match res {
            Ok(_) => 0,
            Err(err) => report_fs_err(err),
        };
    }
    // fd 是 none
    -1
}

pub fn sys_mkdirat(path: *const u8, mode: usize) -> isize {
    let abs_path = build_abs_path(path);

    if mode > u16::MAX as usize {
        return syserr::EINVAL;
    }
    let permissions = VfsPermissions::new(mode as u16);
    match VFS.create_dir(abs_path.as_str()) {
        Err(err) => {
            return report_fs_err(err);
        }
        Ok(mut dir) => {
            dir.set_permissions(&permissions);
        }
    };

    0
}

pub fn sys_fstat(fd: usize, stat_buf: *mut u8) -> isize {
    let token = processor::api::current_user_token();
    let mut stat_buf = UserBuffer::new(page_table::api::translated_byte_buffer(token, stat_buf, 0));

    let pcb = processor::api::current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    let tcb = inner.tcb();
    // fd 越界
    if fd >= tcb.fd_table.len() {
        return syserr::EBADF;
    }

    if let Some(file) = &tcb.fd_table[fd] {
        match file.metadata() {
            Err(err) => return report_fs_err(err),
            Ok(_) => {
                // TODO
                stat_buf.write(&[0]);
                return 0;
            }
        }
    }

    -1
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let token = processor::api::current_user_token();
    let tcb = processor::api::current_tcb();

    let (pipe_read, pipe_write) = pipe::make_pipe();
    let read_fd = tcb.alloc_fd();
    tcb.fd_table[read_fd] = Some(pipe_read);
    let write_fd = tcb.alloc_fd();
    tcb.fd_table[write_fd] = Some(pipe_write);

    let read_slot = page_table::api::translated_refmut(token, unsafe { pipe.add(0) });
    let write_slot = page_table::api::translated_refmut(token, unsafe { pipe.add(1) });

    *read_slot = read_fd;
    *write_slot = write_fd;

    0
}

/// 功能：将一个文件描述符复制到当前可用的最低数值文件描述符，返回新复制的文件描述符。
pub fn sys_dup(fd: usize) -> isize {
    let tcb = processor::api::current_tcb();

    if fd >= tcb.fd_table.len() || tcb.fd_table[fd].is_none() {
        return syserr::EBADF;
    }

    let new_fd = tcb.alloc_fd();
    // clone 一份打开的 fd
    tcb.fd_table[new_fd] = tcb.fd_table[fd].clone();

    new_fd as isize
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
                let meta = VFS.metadata(path);
                match meta {
                    Err(err) => return report_fs_err(err),
                    Ok(meta) if !meta.filetype().is_dir() => return syserr::ENOTDIR,
                    Ok(_) => (),
                }
                cwd.replace(path);
            } else {
                cwd.forward(path);
                let meta = VFS.metadata(cwd.to_string());
                match meta {
                    Err(err) => return report_fs_err(err),
                    // 存在但是不是 dir
                    Ok(meta) if !meta.filetype().is_dir() => {
                        // 撤销 forwards
                        cwd.backward();
                        return syserr::ENOTDIR;
                    }
                    Ok(_) => return 0,
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
    // 如果当前路径已经超过了最大容量
    if cwd_string.len() > max_len {
        return syserr::EINVAL;
    }

    let mut user_buffer = UserBuffer::new(page_table::api::translated_byte_buffer(
        token, buffer, max_len,
    ));
    user_buffer.write(cwd_string.as_bytes());

    0
}

pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize {
    let pcb = processor::api::current_pcb().unwrap();
    let mut inner = pcb.ex_inner();
    let tcb = inner.tcb();
    // fd 越界
    if fd >= tcb.fd_table.len() {
        return syserr::EBADF;
    }

    if let Some(file) = &tcb.fd_table[fd] {
        match file.seek(offset, whence) {
            Err(err) => return report_fs_err(err),
            Ok(_) => return 0,
        }
    }

    -1
}

pub fn sys_unlinkat(path: *const u8) -> isize {
    let abs_path = build_abs_path(path);

    let meta = VFS.metadata(abs_path.as_str());
    match meta {
        Err(err) => return report_fs_err(err),
        Ok(meta) => {
            let filetype = meta.filetype();
            if filetype.is_dir() {
                VFS.remove_dir(abs_path).unwrap();
            } else {
                VFS.remove_file(abs_path).unwrap();
            }
        }
    }

    0
}

pub fn sys_linkat(to: *const u8, from: *const u8) -> isize {
    let abs_to = build_abs_path(to);
    let abs_from = build_abs_path(from);
    match VFS.link(abs_to.as_str(), abs_from.as_str()) {
        Ok(_) => 0,
        Err(err) => report_fs_err(err),
    }
}

pub fn sys_close(fd: usize) -> isize {
    let tcb = processor::api::current_tcb();
    if fd >= tcb.fd_table.len() {
        return syserr::EBADF;
    }
    if tcb.fd_table[fd].is_none() {
        return syserr::EBADF;
    }
    // 所有权取出, 将 None 置入
    tcb.fd_table[fd].take();
    0
}

pub fn sys_io_destroy(_args0: usize, _args1: usize, _args2: usize) -> isize {
    0
}
