//! Regression: a copied test executable must retry ETXTBSY on spawn.
#![cfg(unix)]

use std::fs::{self, OpenOptions};
use std::io;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn output_with_busy_retry_waits_for_open_write_descriptor() {
    let fixture = tempfile::tempdir().expect("create executable fixture");
    let executable = fixture.path().join("busy-child");
    fs::copy(std::env::current_exe().unwrap(), &executable)
        .expect("copy integration executable into fixture");

    // A write descriptor open in this process makes exec of the file fail.
    let writer = OpenOptions::new()
        .write(true)
        .open(&executable)
        .expect("open copied executable for write");
    let error = Command::new(&executable)
        .arg("--list")
        .output()
        .expect_err("an open write descriptor must block exec");
    assert_eq!(error.kind(), io::ErrorKind::ExecutableFileBusy);

    let child = executable.clone();
    let waiter = thread::spawn(move || {
        Command::new(child)
            .arg("--list")
            .output()
            .expect("spawn after the write descriptor is released")
    });
    thread::sleep(Duration::from_millis(400));
    drop(writer);

    let output = waiter.join().expect("retry thread");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
