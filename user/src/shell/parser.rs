use alloc::vec::Vec;
use user_lib::*;

pub fn parse_line(line: &str) -> i32 {
    let args = line.split_whitespace().collect::<Vec<_>>();
    match args {
        _ if args.contains(&"|") => pipe_cmd(line),
        _ if args.contains(&">>") => redirect_append_cmd(line),
        _ if args.contains(&">") => redirect_cmd(line),
        _ => normal_cmd(line),
    }
}

fn pipe_cmd(line: &str) -> i32 {
    let parts = line.split("|").collect::<Vec<_>>();
    if parts.len() != 2 {
        eprintln!("\"|\" just supports once");
    }

    let cmd1 = parts[0].trim();
    let cmd2 = parts[1].trim();

    let mut pipe_fd = [0usize; 2];
    let err = pipe(&mut pipe_fd);
    if err < 0 {
        eprintln!("pipe: {}", err_msg(err));
        return err as i32;
    }
    let read_end = pipe_fd[0];
    let write_end = pipe_fd[1];

    let pid1 = fork();
    if pid1 == 0 {
        close(1);
        dup(write_end);

        close(read_end);
        close(write_end);

        let res = exec(cmd1, None);
        if res < 0 {
            eprintln!("{}", err_msg(res));
            return res as i32;
        }
    }

    let pid2 = fork();
    if pid2 == 0 {
        close(0);
        dup(read_end);

        close(read_end);
        close(write_end);

        let res = exec(cmd2, None);
        if res < 0 {
            eprintln!("{}", err_msg(res));
            return res as i32;
        }
    }

    close(read_end);
    close(write_end);

    windup(pid1);
    windup(pid2);

    0
}

// 支持 I/O 重定向
fn redirect_cmd(line: &str) -> i32 {
    let parts = line.split(">").collect::<Vec<_>>();
    if parts.len() != 2 {
        println!("\">\" just supports once");
    }

    let cmd = parts[0].trim();
    let file = parts[1].trim();
    let fd = open(
        file,
        OpenFlags::WRONLY | OpenFlags::TRUNC | OpenFlags::CREATE,
        0o644,
    );
    if fd < 0 {
        println!("{}: {:?}: {}", cmd, file, err_msg(fd));
    }
    let fd = fd as usize;

    let pid = fork();
    if pid == 0 {
        close(1);
        dup(fd);
        let res = exec(cmd, None);
        if res != 0 {
            println!("{}", err_msg(res));
            return syserr::errno(res) as i32;
        }
        unreachable!()
    } else {
        close(fd);
        windup(pid)
    }
}

fn redirect_append_cmd(line: &str) -> i32 {
    let parts = line.split(">>").collect::<Vec<_>>();
    if parts.len() != 2 {
        println!("\">>\" just supports once");
    }

    let cmd = parts[0].trim();
    let file = parts[1].trim();
    let fd = open(
        file,
        OpenFlags::WRONLY | OpenFlags::CREATE | OpenFlags::APPEND,
        0o644,
    );
    if fd < 0 {
        println!("{}: {:?}: {}", cmd, file, err_msg(fd));
    }
    let fd = fd as usize;

    let pid = fork();
    if pid == 0 {
        close(1);
        dup(fd);
        let res = exec(cmd, None);
        if res != 0 {
            println!("{}", err_msg(res));
            return syserr::errno(res) as i32;
        }
        unreachable!()
    } else {
        close(fd);
        windup(pid)
    }
}

fn normal_cmd(cmd: &str) -> i32 {
    let pid = fork();
    if pid == 0 {
        // 子进程部分
        let res = exec(cmd, None);
        // exec 成功则不会回到此处, 会到此处则说明 exec 时出错
        if res != 0 {
            eprintln!("{}", err_msg(res));
            return 0;
        }
        unreachable!();
    } else {
        windup(pid)
    }
}

fn windup(pid: isize) -> i32 {
    let mut exit_code: i32 = 0;
    let exit_pid = waitpid(pid, &mut exit_code);
    assert_eq!(pid, exit_pid);

    let msg = alloc::format!("[Shell] Process {} exited with code {}", pid, exit_code);
    match exit_code == 0 {
        true => green!("{}", msg),
        false => red!("{}", msg),
    };
    0
}
