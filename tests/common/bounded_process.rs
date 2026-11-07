//! Bounded child-process execution without an external `timeout` program.
//!
//! Fixtures used to run `timeout 90 <wow-sim> ...`, which needs a Unix
//! `timeout` binary; Windows ships an unrelated `timeout.exe` that rejects the
//! same arguments. This module keeps the deadline in the test process instead:
//! spawn the child, drain its pipes, and kill it when the deadline expires.
//!
//! The expiry status matches GNU `timeout`: exit code 124. A child killed at
//! the deadline receives SIGKILL rather than GNU `timeout`'s SIGTERM, so a
//! timed-out fixture reports the same exit code but no termination signal.
//!
//! Included by path from the fixtures that need it:
//!
//! ```ignore
//! #[path = "common/bounded_process.rs"]
//! mod bounded_process;
//! ```

use std::io::{self, Read};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// How often the parent polls for child completion.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// GNU `timeout` exit code after the deadline expires.
const TIMEOUT_EXIT_CODE: u8 = 124;

/// Runs `command` to completion (or the deadline) and collects its output.
///
/// `secs` is the wall-clock deadline in seconds, matching `timeout <secs>`.
/// On expiry the child is killed and reaped, and the returned status carries
/// exit code 124.
pub fn output(secs: u64, command: &mut Command) -> io::Result<Output> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = drain(child.stdout.take().expect("child stdout is piped"));
    let stderr = drain(child.stderr.take().expect("child stderr is piped"));
    let status = wait_with_deadline(&mut child, Duration::from_secs(secs))?;
    Ok(Output {
        status,
        stdout: collect(stdout)?,
        stderr: collect(stderr)?,
    })
}

fn wait_with_deadline(child: &mut Child, deadline: Duration) -> io::Result<ExitStatus> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if started.elapsed() >= deadline {
            kill_and_reap(child)?;
            return Ok(timed_out_status());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn kill_and_reap(child: &mut Child) -> io::Result<()> {
    if let Err(kill_error) = child.kill() {
        // A failed kill is expected when the child exited between the poll and
        // the kill; any other failure leaves a live child behind.
        if child.try_wait()?.is_none() {
            return Err(kill_error);
        }
    }
    child.wait().map(|_| ())
}

#[cfg(unix)]
fn timed_out_status() -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;

    ExitStatus::from_raw(i32::from(TIMEOUT_EXIT_CODE) << 8)
}

#[cfg(windows)]
fn timed_out_status() -> ExitStatus {
    use std::os::windows::process::ExitStatusExt;

    ExitStatus::from_raw(u32::from(TIMEOUT_EXIT_CODE))
}

fn drain(pipe: impl Read + Send + 'static) -> thread::JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut pipe = pipe;
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn collect(handle: thread::JoinHandle<io::Result<Vec<u8>>>) -> io::Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| io::Error::other("child output drain thread panicked"))?
}
