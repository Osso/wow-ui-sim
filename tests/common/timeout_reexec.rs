use super::TIMEOUT_FIXTURE_CAPACITY_ENV;
use super::prefork_process::{
    configure_command_process_group, kill_process_group_and_child, signal_process_group,
    terminate_and_reap_child,
};
use super::workload_gate::{self, Mode};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Once;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const CHILD_TEST_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_TEST";
const HANDSHAKE_PATH_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_HANDSHAKE";
const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(10);
const TERMINATION_GRACE: Duration = Duration::from_millis(100);
const TIMEOUT_CHILD_SLOTS: usize = 2;
const TIMEOUT_SLOT_POLL_INTERVAL: Duration = Duration::from_millis(10);

static HANDSHAKE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static CHILD_HANDSHAKE: Once = Once::new();

pub(super) fn run<F: FnOnce() + Send + 'static>(secs: u64, mode: Mode, closure: F) {
    run_with_gate(secs, mode, None, closure);
}

pub(super) fn run_at<F: FnOnce() + Send + 'static>(secs: u64, path: &Path, mode: Mode, closure: F) {
    run_with_gate(secs, mode, Some(path), closure);
}

fn run_with_gate<F: FnOnce() + Send + 'static>(
    secs: u64,
    mode: Mode,
    path: Option<&Path>,
    closure: F,
) {
    let test_name = current_test_name();
    if let Some(guarded_test) = env::var_os(CHILD_TEST_ENV) {
        run_guarded_child(&test_name, guarded_test, closure);
        return;
    }

    let run_parent = || run_parent(&test_name, secs, path.map(Path::to_path_buf), closure);
    if fixture_capacity_is_reserved() {
        run_parent();
        return;
    }
    match path {
        Some(path) => workload_gate::with_lock_at(path, mode, run_parent),
        None => workload_gate::with_lock(mode, run_parent),
    }
}

fn fixture_capacity_is_reserved() -> bool {
    matches!(
        env::var(TIMEOUT_FIXTURE_CAPACITY_ENV).as_deref(),
        Ok("reserved")
    )
}

fn current_test_name() -> String {
    thread::current()
        .name()
        .unwrap_or_else(|| panic!("with_timeout must run on a named libtest test thread"))
        .to_string()
}

fn run_guarded_child<F: FnOnce()>(test_name: &str, guarded_test: std::ffi::OsString, closure: F) {
    assert_eq!(
        guarded_test.to_string_lossy(),
        test_name,
        "timeout child guard selected a different registered test"
    );
    CHILD_HANDSHAKE.call_once(|| record_handshake(test_name));
    closure();
}

fn record_handshake(test_name: &str) {
    let path = env::var_os(HANDSHAKE_PATH_ENV).expect("timeout child handshake path");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap_or_else(|error| {
            panic!(
                "timeout child entered guarded closure more than once at {}: {error}",
                Path::new(&path).display()
            )
        });
    file.write_all(test_name.as_bytes())
        .expect("write timeout child handshake");
}

fn run_parent<F>(test_name: &str, secs: u64, workload_path: Option<PathBuf>, closure: F)
where
    F: FnOnce() + Send + 'static,
{
    drop(closure);
    let slot_prefix = workload_path.unwrap_or_else(default_timeout_workload_path);
    let _timeout_permit = TimeoutChildPermit::acquire(slot_prefix);
    let execution = execute_timeout_child(test_name, secs);
    report_timeout_child(test_name, secs, execution);
}

fn default_timeout_workload_path() -> PathBuf {
    env::temp_dir().join("wow-ui-sim-test-workloads.lock")
}

struct ChildExecution {
    completion: Result<Completion, String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    handshake_result: Result<(), String>,
    replay_success: bool,
}

fn execute_timeout_child(test_name: &str, secs: u64) -> ChildExecution {
    let handshake = Handshake::new();
    let replay_success = parent_requests_visible_output();
    let mut child = spawn_exact_test(test_name, handshake.path(), replay_success)
        .unwrap_or_else(|error| panic!("spawn timeout child for `{test_name}`: {error}"));
    let pid = child.id() as libc::pid_t;
    let (stdout_drain, stderr_drain) = spawn_child_drains(&mut child);
    let timeout = Duration::from_secs(secs);
    let completion = collect_timeout_child(&mut child, pid, timeout);
    let stdout = join_drain(stdout_drain, "stdout");
    let stderr = join_drain(stderr_drain, "stderr");
    let handshake_result = validate_handshake(handshake.path(), test_name);
    ChildExecution {
        completion,
        stdout,
        stderr,
        handshake_result,
        replay_success,
    }
}

fn spawn_child_drains(
    child: &mut Child,
) -> (
    JoinHandle<io::Result<Vec<u8>>>,
    JoinHandle<io::Result<Vec<u8>>>,
) {
    let stdout = child
        .stdout
        .take()
        .expect("timeout child stdout must be piped");
    let stderr = child
        .stderr
        .take()
        .expect("timeout child stderr must be piped");
    (spawn_drain(stdout), spawn_drain(stderr))
}

fn collect_timeout_child(
    child: &mut Child,
    pid: libc::pid_t,
    timeout: Duration,
) -> Result<Completion, String> {
    match wait_for_child(child, pid, timeout) {
        Ok(completion) => Ok(completion),
        Err(error) => match terminate_and_reap_child(pid) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(format!("{error}; cleanup failed: {cleanup_error}")),
        },
    }
}

fn report_timeout_child(test_name: &str, secs: u64, execution: ChildExecution) {
    let failure = timeout_child_failure(
        test_name,
        secs,
        execution.completion,
        &execution.handshake_result,
    );
    if let Some(message) = failure {
        replay_output(&execution.stdout, &execution.stderr);
        panic!("{message}");
    }
    if execution.replay_success {
        replay_output(&execution.stdout, &execution.stderr);
    }
}

fn timeout_child_failure(
    test_name: &str,
    secs: u64,
    completion: Result<Completion, String>,
    handshake_result: &Result<(), String>,
) -> Option<String> {
    let handshake_detail = format_handshake_detail(handshake_result);
    match completion {
        Ok(Completion::Exited(status)) if status.success() && handshake_result.is_ok() => None,
        Ok(Completion::Exited(status)) => Some(format!(
            "timeout child for `{test_name}` failed ({}){handshake_detail}",
            describe_status(status)
        )),
        Ok(Completion::TimedOut) => Some(format!(
            "test `{test_name}` timed out after {secs}s{handshake_detail}"
        )),
        Err(error) => Some(format!(
            "timeout child for `{test_name}` could not be collected: {error}"
        )),
    }
}

fn format_handshake_detail(handshake_result: &Result<(), String>) -> String {
    handshake_result
        .as_ref()
        .err()
        .map(|error| format!("; {error}"))
        .unwrap_or_default()
}

struct TimeoutChildPermit {
    _file: File,
}

impl TimeoutChildPermit {
    fn acquire(workload_path: PathBuf) -> Self {
        acquire_timeout_child_permit(workload_path)
    }
}

fn acquire_timeout_child_permit(workload_path: PathBuf) -> TimeoutChildPermit {
    let prefix = workload_path.to_string_lossy();
    loop {
        for slot in 0..TIMEOUT_CHILD_SLOTS {
            let path = format!("{prefix}.timeout-slot-{slot}");
            let file = open_and_try_lock_timeout_slot(&path)
                .unwrap_or_else(|error| panic!("acquire timeout child slot `{path}`: {error}"));
            if let Some(file) = file {
                return TimeoutChildPermit { _file: file };
            }
        }
        thread::sleep(TIMEOUT_SLOT_POLL_INTERVAL);
    }
}

fn open_and_try_lock_timeout_slot(path: &str) -> io::Result<Option<File>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    if try_lock_timeout_slot(&file)? {
        Ok(Some(file))
    } else {
        Ok(None)
    }
}

fn try_lock_timeout_slot(file: &File) -> io::Result<bool> {
    loop {
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if result == 0 {
            return Ok(true);
        }
        let error = io::Error::last_os_error();
        match error.kind() {
            io::ErrorKind::Interrupted => continue,
            io::ErrorKind::WouldBlock => return Ok(false),
            _ => return Err(error),
        }
    }
}

fn spawn_exact_test(
    test_name: &str,
    handshake_path: &Path,
    replay_success: bool,
) -> io::Result<Child> {
    let executable = env::current_exe()?;
    let current_dir = env::current_dir()?;
    let mut command = Command::new(executable);
    command
        .arg(test_name)
        .args(["--exact", "--include-ignored", "--test-threads=1"])
        .current_dir(current_dir)
        .env(CHILD_TEST_ENV, test_name)
        .env(HANDSHAKE_PATH_ENV, handshake_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if replay_success {
        forward_output_options(&mut command);
    }
    configure_command_process_group(&mut command);
    command.spawn()
}

fn parent_requests_visible_output() -> bool {
    env::args_os().any(|argument| {
        matches!(
            argument.to_str(),
            Some("--nocapture" | "--no-capture" | "--show-output")
        )
    })
}

fn forward_output_options(command: &mut Command) {
    for argument in env::args_os() {
        match argument.to_str() {
            Some("--nocapture" | "--no-capture") => {
                command.arg("--nocapture");
            }
            Some("--show-output") => {
                command.arg("--show-output");
            }
            _ => {}
        }
    }
}

enum Completion {
    Exited(ExitStatus),
    TimedOut,
}

fn wait_for_child(
    child: &mut Child,
    pid: libc::pid_t,
    timeout: Duration,
) -> Result<Completion, String> {
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("poll child {pid}: {error}"))?
        {
            kill_process_group_and_child(pid)?;
            return Ok(Completion::Exited(status));
        }
        if started.elapsed() >= timeout {
            terminate_timed_out_child(child, pid)?;
            return Ok(Completion::TimedOut);
        }
        thread::sleep(CHILD_POLL_INTERVAL);
    }
}

fn terminate_timed_out_child(child: &mut Child, pid: libc::pid_t) -> Result<(), String> {
    signal_process_group(pid, libc::SIGTERM)?;
    let deadline = Instant::now() + TERMINATION_GRACE;
    while Instant::now() < deadline {
        if child
            .try_wait()
            .map_err(|error| format!("poll terminating child {pid}: {error}"))?
            .is_some()
        {
            break;
        }
        thread::sleep(CHILD_POLL_INTERVAL);
    }
    kill_process_group_and_child(pid)?;
    child
        .wait()
        .map_err(|error| format!("reap timed-out child {pid}: {error}"))?;
    Ok(())
}

fn spawn_drain<R>(mut reader: R) -> JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn join_drain(handle: JoinHandle<io::Result<Vec<u8>>>, stream: &str) -> Vec<u8> {
    handle
        .join()
        .unwrap_or_else(|_| panic!("timeout child {stream} drain thread panicked"))
        .unwrap_or_else(|error| panic!("read timeout child {stream}: {error}"))
}

fn replay_output(stdout: &[u8], stderr: &[u8]) {
    if !stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(stdout));
    }
    if !stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(stderr));
    }
}

fn validate_handshake(path: &Path, expected_test: &str) -> Result<(), String> {
    let entered_test = fs::read_to_string(path)
        .map_err(|error| format!("guarded closure handshake missing: {error}"))?;
    if entered_test == expected_test {
        Ok(())
    } else {
        Err(format!(
            "guarded closure handshake named `{entered_test}`, expected `{expected_test}`"
        ))
    }
}

fn describe_status(status: ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;

    if let Some(code) = status.code() {
        format!("exit code {code}")
    } else if let Some(signal) = status.signal() {
        format!("signal {signal}")
    } else {
        "unknown exit status".to_string()
    }
}

struct Handshake {
    path: PathBuf,
}

impl Handshake {
    fn new() -> Self {
        let sequence = HANDSHAKE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "wow-ui-sim-timeout-handshake-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Handshake {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
