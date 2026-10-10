use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::Command;

const SETUP_READY: u8 = 0;
const SETUP_FAILED: u8 = 1;
const SETUP_RELEASE: u8 = 2;

pub(crate) struct Pipe {
    pub(crate) read_fd: OwnedFd,
    pub(crate) write_fd: OwnedFd,
}

pub(crate) fn create_pipe() -> Result<Pipe, String> {
    let mut fds = [0; 2];
    let result = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if result == -1 {
        return Err(format!("pipe2 failed: {}", io::Error::last_os_error()));
    }
    Ok(Pipe {
        read_fd: unsafe { OwnedFd::from_raw_fd(fds[0]) },
        write_fd: unsafe { OwnedFd::from_raw_fd(fds[1]) },
    })
}

pub(crate) fn create_setup_socket() -> Result<(OwnedFd, OwnedFd), String> {
    let mut fds = [0; 2];
    let socket_type = libc::SOCK_STREAM | libc::SOCK_CLOEXEC;
    let result = unsafe { libc::socketpair(libc::AF_UNIX, socket_type, 0, fds.as_mut_ptr()) };
    if result == -1 {
        return Err(format!(
            "setup socketpair failed: {}",
            io::Error::last_os_error()
        ));
    }
    Ok((unsafe { OwnedFd::from_raw_fd(fds[0]) }, unsafe {
        OwnedFd::from_raw_fd(fds[1])
    }))
}

pub(crate) fn prepare_capture_pipe(pipe: Option<Pipe>) -> Result<Option<OwnedFd>, String> {
    let Some(pipe) = pipe else {
        return Ok(None);
    };
    drop(pipe.write_fd);
    set_nonblocking(pipe.read_fd.as_raw_fd())?;
    Ok(Some(pipe.read_fd))
}

pub(crate) fn establish_child_process_group(setup_fd: RawFd) -> Result<(), String> {
    let result = unsafe { libc::setpgid(0, 0) };
    if result == -1 {
        let error = io::Error::last_os_error();
        report_child_setup_failure(setup_fd, &error);
        return Err(format!("setpgid failed: {error}"));
    }
    send_all_socket(setup_fd, &[SETUP_READY], "report child setup")?;

    let mut release = [0_u8; 1];
    read_exact_fd(setup_fd, &mut release, "await parent setup release")?;
    if release[0] != SETUP_RELEASE {
        return Err("invalid setup release".to_string());
    }
    Ok(())
}

pub(crate) fn verify_and_release_child(pid: libc::pid_t, setup_fd: OwnedFd) -> Result<(), String> {
    await_child_process_group(pid, &setup_fd)?;
    verify_child_process_group(pid)?;
    send_all_socket(
        setup_fd.as_raw_fd(),
        &[SETUP_RELEASE],
        "release child setup",
    )
}

pub(crate) fn configure_command_process_group(command: &mut Command) {
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

pub(crate) fn signal_process_group(pid: libc::pid_t, signal: i32) -> Result<(), String> {
    let result = unsafe { libc::kill(-pid, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }
    Err(format!("signal process group {pid} failed: {error}"))
}

pub(crate) fn kill_process_group_and_child(pid: libc::pid_t) -> Result<(), String> {
    let group_result = signal_process_group(pid, libc::SIGKILL);
    let child_result = signal_process(pid, libc::SIGKILL);
    combine_results(group_result, child_result)
}

pub(crate) fn terminate_and_reap_child(pid: libc::pid_t) -> Result<(), String> {
    let kill_result = kill_process_group_and_child(pid);
    let reap_result = reap_child(pid);
    combine_results(kill_result, reap_result)
}

pub(crate) fn reap_child(pid: libc::pid_t) -> Result<(), String> {
    loop {
        let mut status = 0;
        let result = unsafe { libc::waitpid(pid, &mut status, 0) };
        if result == pid {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            continue;
        }
        if error.raw_os_error() == Some(libc::ECHILD) {
            return Ok(());
        }
        return Err(format!("waitpid for {pid} during cleanup failed: {error}"));
    }
}

fn set_nonblocking(fd: RawFd) -> Result<(), String> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 {
        return Err(format!(
            "fcntl F_GETFL failed: {}",
            io::Error::last_os_error()
        ));
    }
    let result = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if result == -1 {
        return Err(format!(
            "fcntl F_SETFL failed: {}",
            io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn report_child_setup_failure(setup_fd: RawFd, error: &io::Error) {
    let errno = error.raw_os_error().unwrap_or(libc::EIO);
    let mut message = [0_u8; 5];
    message[0] = SETUP_FAILED;
    message[1..].copy_from_slice(&errno.to_ne_bytes());
    let _ = send_all_socket(setup_fd, &message, "report child setup failure");
}

fn await_child_process_group(pid: libc::pid_t, setup_fd: &OwnedFd) -> Result<(), String> {
    let mut kind = [0_u8; 1];
    read_exact_fd(setup_fd.as_raw_fd(), &mut kind, "read child setup status")?;
    match kind[0] {
        SETUP_READY => Ok(()),
        SETUP_FAILED => read_child_setup_error(pid, setup_fd),
        value => Err(format!("child {pid} reported invalid setup status {value}")),
    }
}

fn read_child_setup_error(pid: libc::pid_t, setup_fd: &OwnedFd) -> Result<(), String> {
    let mut errno = [0_u8; 4];
    read_exact_fd(setup_fd.as_raw_fd(), &mut errno, "read child setup error")?;
    let error = io::Error::from_raw_os_error(i32::from_ne_bytes(errno));
    Err(format!("setpgid for child {pid} failed: {error}"))
}

fn verify_child_process_group(pid: libc::pid_t) -> Result<(), String> {
    let process_group = unsafe { libc::getpgid(pid) };
    if process_group == -1 {
        return Err(format!(
            "getpgid for child {pid} failed: {}",
            io::Error::last_os_error()
        ));
    }
    if process_group != pid {
        return Err(format!(
            "child {pid} entered unexpected process group {process_group}"
        ));
    }
    Ok(())
}

fn read_exact_fd(fd: RawFd, mut bytes: &mut [u8], operation: &str) -> Result<(), String> {
    while !bytes.is_empty() {
        let read = unsafe { libc::read(fd, bytes.as_mut_ptr().cast(), bytes.len()) };
        if read > 0 {
            bytes = &mut bytes[read as usize..];
            continue;
        }
        if read == 0 {
            return Err(format!("{operation} failed: unexpected EOF"));
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            continue;
        }
        return Err(format!("{operation} failed: {error}"));
    }
    Ok(())
}

fn send_all_socket(fd: RawFd, mut bytes: &[u8], operation: &str) -> Result<(), String> {
    while !bytes.is_empty() {
        let sent =
            unsafe { libc::send(fd, bytes.as_ptr().cast(), bytes.len(), libc::MSG_NOSIGNAL) };
        if sent > 0 {
            bytes = &bytes[sent as usize..];
            continue;
        }
        if sent == 0 {
            return Err(format!("{operation} failed: zero-byte send"));
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            continue;
        }
        return Err(format!("{operation} failed: {error}"));
    }
    Ok(())
}

fn signal_process(pid: libc::pid_t, signal: i32) -> Result<(), String> {
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }
    Err(format!("signal child {pid} failed: {error}"))
}

fn combine_results(first: Result<(), String>, second: Result<(), String>) -> Result<(), String> {
    match (first, second) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(first), Err(second)) => Err(format!("{first}; {second}")),
    }
}
