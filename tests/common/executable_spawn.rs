//! Spawning helpers for tests that copy an executable before running it.
use std::io;
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

const MAX_ATTEMPTS: u32 = 40;
const INITIAL_BACKOFF: Duration = Duration::from_millis(5);
const MAX_BACKOFF: Duration = Duration::from_millis(50);

/// Runs a command, retrying `ExecutableFileBusy` (ETXTBSY) with bounded backoff.
///
/// Between copying an executable and executing the copy, another test thread
/// can fork. Its child temporarily inherits the copy's open write descriptor
/// until the child execs, so our exec of the freshly written file fails with
/// ETXTBSY. Cargo retries the same condition.
pub(crate) fn output_with_busy_retry(command: &mut Command) -> io::Result<Output> {
    let mut attempts = 1;
    let mut backoff = INITIAL_BACKOFF;
    loop {
        match command.output() {
            Err(error)
                if error.kind() == io::ErrorKind::ExecutableFileBusy
                    && attempts < MAX_ATTEMPTS =>
            {
                attempts += 1;
                thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
            result => return result,
        }
    }
}
