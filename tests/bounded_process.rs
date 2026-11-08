//! Behaviour of the shared bounded child-process helper.

#[path = "common/bounded_process.rs"]
mod bounded_process;

use std::process::Command;
use std::time::{Duration, Instant};

/// Selects what the re-executed child test does; unset in the parent process.
const CHILD_MODE_ENV: &str = "WOW_SIM_BOUNDED_PROCESS_CHILD";

#[test]
fn bounded_process_child() {
    match std::env::var(CHILD_MODE_ENV).as_deref() {
        Ok("complete") => println!("BOUNDED_CHILD_COMPLETE"),
        Ok("fail") => std::process::exit(3),
        Ok("hang") => std::thread::sleep(Duration::from_secs(300)),
        _ => {}
    }
}

/// Re-executes this binary for the child test under `mode`.
fn child(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("locate test binary"));
    command
        .args([
            child_test_name().as_str(),
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_MODE_ENV, mode);
    command
}

/// Full path of `bounded_process_child` in the enclosing test module.
fn child_test_name() -> String {
    let current = std::thread::current()
        .name()
        .expect("bounded_process tests run on a named libtest thread")
        .to_string();
    let module = current
        .rsplit_once("::")
        .map(|(module, _)| module)
        .expect("libtest thread names are module-qualified");
    format!("{module}::bounded_process_child")
}

#[test]
fn completed_child_reports_its_own_output_and_status() {
    let output = bounded_process::output(90, &mut child("complete")).expect("run completed child");

    assert!(output.status.success(), "status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BOUNDED_CHILD_COMPLETE"),
        "child stdout: {stdout}"
    );
}

#[test]
fn failing_child_reports_its_nonzero_exit_code() {
    let output = bounded_process::output(90, &mut child("fail")).expect("run failing child");

    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn child_past_the_deadline_is_killed_and_reports_timeout_code() {
    let started = Instant::now();
    let output = bounded_process::output(1, &mut child("hang")).expect("run hanging child");
    let elapsed = started.elapsed();

    assert_eq!(output.status.code(), Some(124), "status: {}", output.status);
    assert!(
        elapsed < Duration::from_secs(30),
        "deadline did not bound the child: {elapsed:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("BOUNDED_CHILD_COMPLETE"),
        "hung child reported completion: {stdout}"
    );
}
