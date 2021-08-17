use std::fs::File;
use std::io::{Read, Write};

#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

fn main() {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
    }
    let logdir = std::env::temp_dir().join("pipedump");
    if !logdir.exists() {
        std::fs::create_dir(&logdir).unwrap();
    }
    let stdin = pipe_as_file(std::io::stdin());
    let stdout = pipe_as_file(std::io::stdout());
    let stderr = pipe_as_file(std::io::stderr());
    let args: Vec<_> = std::env::args_os().collect();
    let mut target = std::process::Command::new(&args[1])
        .args(&args[2..])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let stdin2 = pipe_into_file(target.stdin.take().unwrap());
    let stdout2 = pipe_into_file(target.stdout.take().unwrap());
    let stderr2 = pipe_into_file(target.stderr.take().unwrap());
    let inlog = File::create(logdir.join("in.log")).unwrap();
    let outlog = File::create(logdir.join("out.log")).unwrap();
    let errlog = File::create(logdir.join("err.log")).unwrap();
    let _inthread = std::thread::spawn(|| dump(stdin, stdin2, inlog));
    let outthread = std::thread::spawn(|| dump(stdout2, stdout, outlog));
    let errthread = std::thread::spawn(|| dump(stderr2, stderr, errlog));
    let exitcode = target.wait().unwrap();
    //_inthread.join().unwrap();     // stdin 不需要等待，目标进程都没了，输入传给谁去
    outthread.join().unwrap();
    errthread.join().unwrap();
    std::process::exit(exitcode.code().unwrap());
}

#[cfg(windows)]
fn pipe_as_file<T: AsRawHandle>(p: T) -> File {
    unsafe { File::from_raw_handle(p.as_raw_handle()) }
}

#[cfg(windows)]
fn pipe_into_file<T: IntoRawHandle>(p: T) -> File {
    unsafe { File::from_raw_handle(p.into_raw_handle()) }
}

#[cfg(unix)]
fn pipe_as_file<T: AsRawFd>(p: T) -> File {
    unsafe { File::from_raw_fd(p.as_raw_fd()) }
}

#[cfg(unix)]
fn pipe_into_file<T: IntoRawFd>(p: T) -> File {
    unsafe { File::from_raw_fd(p.into_raw_fd()) }
}

fn dump(mut from: File, mut to: File, mut logfile: File) {
    let mut buf = [0u8; 1024];
    while let Ok(n) = from.read(&mut buf) {
        if n == 0 {
            break;
        }
        let s = &buf[..n];
        logfile.write(s).unwrap();
        to.write(s).unwrap();
    }
    logfile.flush().unwrap();
}
