#![cfg(target_os = "linux")]

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const FIXTURE_MODE_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_FIXTURE";
const SIBLING_MARKER_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_SIBLING_MARKER";
const DESCENDANT_PID_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_DESCENDANT_PID";
const CONCURRENCY_STATE_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_CONCURRENCY_STATE";
const EXACT_SELECTED_MARKER_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_EXACT_SELECTED_MARKER";
const EXACT_FORBIDDEN_MARKER_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_EXACT_FORBIDDEN_MARKER";
const PERMIT_RELEASE_DIR_ENV: &str = "WOW_SIM_TIMEOUT_REEXEC_PERMIT_RELEASE_DIR";
const PROCESS_DEATH_WAIT: Duration = Duration::from_secs(2);
const FIXTURE_MARKER_WAIT: Duration = Duration::from_secs(4);
const CONCURRENCY_HOLD: Duration = Duration::from_millis(600);
fn expected_timeout_child_limit() -> u32 {
    match std::env::var("WOW_SIM_TIMEOUT_CHILD_SLOTS") {
        Ok(value) => value
            .parse()
            .expect("configured timeout child limit is an integer"),
        Err(std::env::VarError::NotPresent) => 2,
        Err(error) => panic!("read configured timeout child limit: {error}"),
    }
}

#[test]
fn panic_failure_is_aggregated_and_later_sibling_runs() {
    let temp = TempDir::new().expect("create panic fixture tempdir");
    let sibling_marker = temp.path().join("panic-sibling-complete");
    let output = run_fixture(
        "timeout_reexec::fixtures::panic_case::",
        "panic",
        &sibling_marker,
        None,
    );
    let stdout = stdout(&output);
    let stderr = stderr(&output);

    assert!(
        !output.status.success(),
        "panic fixture must report one failure"
    );
    assert!(
        sibling_marker.exists(),
        "later panic sibling did not execute\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("timeout reexec panic stdout marker"));
    assert!(stderr.contains("timeout reexec panic stderr marker"));
    assert!(stdout.contains("timeout_reexec::fixtures::panic_case::b_records_completion ... ok"));
    assert!(stdout.contains("test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured;"));
    assert_final_failure_list(&stdout, "timeout_reexec::fixtures::panic_case::a_panics");
}

#[test]
fn timeout_kills_descendant_and_later_sibling_runs() {
    let temp = TempDir::new().expect("create timeout fixture tempdir");
    let sibling_marker = temp.path().join("timeout-sibling-complete");
    let descendant_pid_path = temp.path().join("descendant.pid");
    let output = run_fixture(
        "timeout_reexec::fixtures::timeout_case::",
        "timeout",
        &sibling_marker,
        Some(&descendant_pid_path),
    );
    let stdout = stdout(&output);
    let stderr = stderr(&output);
    let descendant_pid = read_pid(&descendant_pid_path);
    let descendant_disappeared = wait_for_process_exit(descendant_pid);
    if !descendant_disappeared {
        kill_process(descendant_pid);
    }

    assert!(
        !output.status.success(),
        "timeout fixture must report one failure"
    );
    assert!(
        sibling_marker.exists(),
        "later timeout sibling did not execute\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("timeout reexec timeout stdout marker"));
    assert!(stderr.contains("timeout reexec timeout stderr marker"));
    assert!(stdout.contains("timeout_reexec::fixtures::timeout_case::b_records_completion ... ok"));
    assert!(stdout.contains("test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured;"));
    assert!(
        stderr.contains("timed out after 1s"),
        "timeout reason was not forwarded\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        descendant_disappeared,
        "timeout descendant {descendant_pid} survived process-group cleanup"
    );
    assert_final_failure_list(
        &stdout,
        "timeout_reexec::fixtures::timeout_case::a_times_out_with_descendant",
    );
}

#[test]
fn nested_guards_execute_each_closure_once() {
    let temp = TempDir::new().expect("create nested fixture tempdir");
    let marker = temp.path().join("nested-closures");
    let output = run_inventory_fixture(&marker, None);
    let fixture_stdout = stdout(&output);
    let fixture_stderr = stderr(&output);

    assert!(
        output.status.success(),
        "nested timeout guards failed\nstdout:\n{fixture_stdout}\nstderr:\n{fixture_stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(&marker).expect("read nested closure marker"),
        "first\nsecond\n"
    );
    assert!(
        fixture_stderr.is_empty(),
        "disabled inventory changed stderr: {fixture_stderr}"
    );
    let entries: Vec<_> = std::fs::read_dir(temp.path())
        .expect("read fixture directory")
        .map(|entry| entry.expect("fixture entry").file_name())
        .collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("nested-closures")]);

    let inventory = temp.path().join("timeout-entry.txt");
    let recorded = run_inventory_fixture(&marker, Some(&inventory));
    assert!(
        recorded.status.success(),
        "inventory fixture failed: {}",
        stderr(&recorded)
    );
    assert!(stderr(&recorded).is_empty());
    let without_timing = |text: String| {
        text.lines()
            .map(|line| line.split("finished in").next().expect("timing prefix"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        without_timing(fixture_stdout),
        without_timing(stdout(&recorded))
    );
    assert_eq!(
        std::fs::read_to_string(inventory).expect("read timeout entry inventory"),
        "timeout_reexec::fixtures::nested_case::nested_guards_run_both_closures\n"
    );
}

#[test]
fn timeout_children_are_limited_to_two_concurrent_processes() {
    assert_concurrency_fixture(true);
}

#[test]
fn timeout_fixtures_can_run_concurrently() {
    std::thread::scope(|scope| {
        let launchers = [
            scope.spawn(|| assert_concurrency_fixture(false)),
            scope.spawn(assert_permit_release_fixture),
            scope.spawn(|| assert_concurrency_fixture(false)),
            scope.spawn(assert_permit_release_fixture),
            scope.spawn(|| assert_concurrency_fixture(false)),
            scope.spawn(assert_permit_release_fixture),
        ];
        for launcher in launchers {
            launcher.join().expect("timeout fixture launcher thread");
        }
    });
}

fn assert_concurrency_fixture(expect_saturation: bool) {
    let temp = TempDir::new().expect("create concurrency fixture tempdir");
    let state_path = temp.path().join("concurrency-state");
    std::fs::write(&state_path, "0 0\n").expect("initialize concurrency state");

    let output = run_concurrency_fixture(&state_path);
    let stdout = stdout(&output);
    let stderr = stderr(&output);
    assert!(
        output.status.success(),
        "concurrency fixture failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let (active, peak) = read_concurrency_state(&state_path);
    assert_eq!(active, 0, "all timeout children must release their slots");
    let child_limit = expected_timeout_child_limit();
    if expect_saturation {
        assert_eq!(
            peak, child_limit,
            "single fixture must reach the child limit"
        );
    } else {
        assert!(
            peak > 0 && peak <= child_limit,
            "concurrent fixture peak must stay within the configured child limit"
        );
    }
}

#[test]
fn timeout_child_runs_only_exact_selected_test() {
    let temp = TempDir::new().expect("create exact-selection fixture tempdir");
    let selected_marker = temp.path().join("selected");
    let forbidden_marker = temp.path().join("forbidden");
    let output = run_exact_selection_fixture(&selected_marker, &forbidden_marker);
    let stdout = stdout(&output);
    let stderr = stderr(&output);

    assert!(
        output.status.success(),
        "exact-selection fixture failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(&selected_marker).expect("read selected marker"),
        "selected\n",
        "selected timeout closure must execute exactly once"
    );
    assert!(
        !forbidden_marker.exists(),
        "suffix-named test was selected unexpectedly"
    );
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;"),
        "exact fixture did not report one selected test\nstdout:\n{stdout}"
    );
}

#[test]
fn timed_out_child_releases_permit_for_waiting_sibling() {
    assert_permit_release_fixture();
}

fn assert_permit_release_fixture() {
    let temp = TempDir::new().expect("create permit-release fixture tempdir");
    let output = run_permit_release_fixture(temp.path());
    let stdout = stdout(&output);
    let stderr = stderr(&output);

    assert!(
        !output.status.success(),
        "permit-release fixture must time out once"
    );
    assert!(
        temp.path().join("waiting-complete").exists(),
        "waiting timeout sibling did not run\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured;"),
        "permit-release fixture reported unexpected results\nstdout:\n{stdout}"
    );
    assert!(
        stderr.contains("timed out after 1s"),
        "permit-release timeout reason was not forwarded\nstderr:\n{stderr}"
    );
    assert_final_failure_list(
        &stdout,
        "timeout_reexec::fixtures::permit_release_case::a_times_out_holding_permit",
    );
}

fn run_inventory_fixture(marker: &Path, inventory: Option<&Path>) -> Output {
    let mut command = Command::new(std::env::current_exe().expect("integration test binary"));
    command
        .args([
            "timeout_reexec::fixtures::nested_case::",
            "--test-threads=1",
            "--nocapture",
        ])
        .current_dir(marker.parent().expect("fixture directory"))
        .env(FIXTURE_MODE_ENV, "nested")
        .env(SIBLING_MARKER_ENV, marker)
        .env_remove("WOW_SIM_TIMEOUT_INVENTORY");
    if let Some(path) = inventory {
        command.env("WOW_SIM_TIMEOUT_INVENTORY", path);
    }
    command.output().expect("run timeout inventory fixture")
}

fn run_fixture(
    filter: &str,
    mode: &str,
    sibling_marker: &Path,
    descendant_pid_path: Option<&Path>,
) -> Output {
    let mut command =
        Command::new(std::env::current_exe().expect("resolve integration test binary"));
    command
        .args([filter, "--test-threads=1", "--nocapture"])
        .env(FIXTURE_MODE_ENV, mode)
        .env(SIBLING_MARKER_ENV, sibling_marker);
    if let Some(path) = descendant_pid_path {
        command.env(DESCENDANT_PID_ENV, path);
    }
    command.output().expect("run timeout re-exec fixture")
}

fn run_concurrency_fixture(state_path: &Path) -> Output {
    let mut command =
        Command::new(std::env::current_exe().expect("resolve integration test binary"));
    command
        .args([
            "timeout_reexec::fixtures::concurrency_case::",
            "--test-threads=6",
            "--nocapture",
        ])
        .env(FIXTURE_MODE_ENV, "concurrency")
        .env(CONCURRENCY_STATE_ENV, state_path);
    spawn_capacity_reserved_fixture(command)
}

fn run_exact_selection_fixture(selected_marker: &Path, forbidden_marker: &Path) -> Output {
    let mut command =
        Command::new(std::env::current_exe().expect("resolve integration test binary"));
    command
        .args([
            "timeout_reexec::fixtures::exact_selection_case::selected_target",
            "--exact",
            "--test-threads=1",
            "--nocapture",
        ])
        .env(FIXTURE_MODE_ENV, "exact-selection")
        .env(EXACT_SELECTED_MARKER_ENV, selected_marker)
        .env(EXACT_FORBIDDEN_MARKER_ENV, forbidden_marker);
    command
        .output()
        .expect("run timeout exact-selection fixture")
}

fn run_permit_release_fixture(release_dir: &Path) -> Output {
    let mut command =
        Command::new(std::env::current_exe().expect("resolve integration test binary"));
    command
        .args([
            "timeout_reexec::fixtures::permit_release_case::",
            "--test-threads=3",
            "--nocapture",
        ])
        .env(FIXTURE_MODE_ENV, "permit-release")
        .env(PERMIT_RELEASE_DIR_ENV, release_dir);
    spawn_capacity_reserved_fixture(command)
}

fn spawn_capacity_reserved_fixture(mut command: Command) -> Output {
    crate::common::with_timeout_fixture_capacity(|| {
        command
            .env(crate::common::TIMEOUT_FIXTURE_CAPACITY_ENV, "reserved")
            .output()
            .expect("run timeout re-exec fixture")
    })
}

fn assert_final_failure_list(stdout: &str, expected_failure: &str) {
    let failures = stdout
        .rsplit_once("failures:\n")
        .map(|(_, failures)| failures)
        .expect("normal libtest failures section");
    assert!(failures.contains(expected_failure));
    assert_eq!(
        failures
            .lines()
            .filter(|line| line.trim_start().starts_with("timeout_reexec::fixtures::"))
            .count(),
        1,
        "expected exactly one named fixture failure in final summary:\n{failures}"
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn read_pid(path: &Path) -> libc::pid_t {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read descendant pid from {}: {error}", path.display()))
        .trim()
        .parse()
        .expect("parse descendant pid")
}

fn wait_for_process_exit(pid: libc::pid_t) -> bool {
    let deadline = Instant::now() + PROCESS_DEATH_WAIT;
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

fn process_exists(pid: libc::pid_t) -> bool {
    let result = unsafe { libc::kill(pid, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn kill_process(pid: libc::pid_t) {
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
}

fn read_concurrency_state(path: &Path) -> (u32, u32) {
    let state = std::fs::read_to_string(path).expect("read timeout concurrency state");
    parse_concurrency_state(&state)
}

fn parse_concurrency_state(state: &str) -> (u32, u32) {
    let mut values = state.split_whitespace();
    let active = values
        .next()
        .expect("concurrency state active count")
        .parse()
        .expect("parse concurrency active count");
    let peak = values
        .next()
        .expect("concurrency state peak count")
        .parse()
        .expect("parse concurrency peak count");
    assert!(values.next().is_none(), "unexpected concurrency state data");
    (active, peak)
}

fn update_concurrency_state(path: &Path, delta: i32) {
    let mut file = open_locked_concurrency_state(path);
    let current = read_locked_concurrency_state(&mut file);
    let updated = apply_concurrency_delta(current, delta);
    write_locked_concurrency_state(&mut file, updated);
}

fn open_locked_concurrency_state(path: &Path) -> File {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open timeout concurrency state");
    lock_exclusively(&file);
    file
}

fn read_locked_concurrency_state(file: &mut File) -> (u32, u32) {
    let mut state = String::new();
    file.read_to_string(&mut state)
        .expect("read locked timeout concurrency state");
    parse_concurrency_state(&state)
}

fn apply_concurrency_delta((active, peak): (u32, u32), delta: i32) -> (u32, u32) {
    let next_active = if delta >= 0 {
        active.checked_add(delta as u32)
    } else {
        active.checked_sub(delta.unsigned_abs())
    }
    .expect("timeout concurrency active count overflow");
    (next_active, peak.max(next_active))
}

fn write_locked_concurrency_state(file: &mut File, (active, peak): (u32, u32)) {
    // The configured child limit is at most six, so both counters stay one digit.
    // Keep the four-byte record in place; termination must not leave it truncated.
    file.seek(SeekFrom::Start(0))
        .expect("rewind timeout concurrency state");
    file.write_all(format!("{active} {peak}\n").as_bytes())
        .expect("write timeout concurrency state");
}

fn lock_exclusively(file: &File) {
    loop {
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if result == 0 {
            return;
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            panic!("lock timeout concurrency state: {error}");
        }
    }
}

mod fixtures {
    use super::*;

    pub(super) mod panic_case {
        use super::*;

        #[test]
        fn a_panics() {
            if fixture_mode() != Some("panic") {
                return;
            }
            crate::common::with_timeout(5, || {
                println!("timeout reexec panic stdout marker");
                eprintln!("timeout reexec panic stderr marker");
                panic!("deliberate timeout reexec panic");
            });
        }

        #[test]
        fn b_records_completion() {
            if fixture_mode() != Some("panic") {
                return;
            }
            record_sibling_completion();
        }
    }

    pub(super) mod timeout_case {
        use super::*;

        #[test]
        fn a_times_out_with_descendant() {
            if fixture_mode() != Some("timeout") {
                return;
            }
            crate::common::with_timeout(1, || {
                println!("timeout reexec timeout stdout marker");
                eprintln!("timeout reexec timeout stderr marker");
                let descendant = Command::new("sleep")
                    .arg("60")
                    .spawn()
                    .expect("spawn timeout descendant");
                let pid_path = PathBuf::from(
                    std::env::var_os(DESCENDANT_PID_ENV)
                        .expect("timeout descendant pid path environment"),
                );
                std::fs::write(&pid_path, descendant.id().to_string())
                    .expect("write timeout descendant pid");
                drop(descendant);
                thread::sleep(Duration::from_secs(30));
            });
        }

        #[test]
        fn b_records_completion() {
            if fixture_mode() != Some("timeout") {
                return;
            }
            record_sibling_completion();
        }
    }

    pub(super) mod nested_case {
        use super::*;

        #[test]
        fn nested_guards_run_both_closures() {
            if fixture_mode() != Some("nested") {
                return;
            }
            crate::common::with_timeout(5, || {
                record_nested_closure("first");
                crate::common::with_timeout(5, || record_nested_closure("second"));
            });
        }
    }

    pub(super) mod concurrency_case {
        use super::*;

        #[test]
        fn a_records_concurrency() {
            if fixture_mode() == Some("concurrency") {
                crate::common::with_timeout(5, record_concurrency_window);
            }
        }

        #[test]
        fn b_records_concurrency() {
            if fixture_mode() == Some("concurrency") {
                crate::common::with_timeout(5, record_concurrency_window);
            }
        }

        #[test]
        fn c_records_concurrency() {
            if fixture_mode() == Some("concurrency") {
                crate::common::with_timeout(5, record_concurrency_window);
            }
        }

        #[test]
        fn d_records_concurrency() {
            if fixture_mode() == Some("concurrency") {
                crate::common::with_timeout(5, record_concurrency_window);
            }
        }

        #[test]
        fn e_records_concurrency() {
            if fixture_mode() == Some("concurrency") {
                crate::common::with_timeout(5, record_concurrency_window);
            }
        }

        #[test]
        fn f_records_concurrency() {
            if fixture_mode() == Some("concurrency") {
                crate::common::with_timeout(5, record_concurrency_window);
            }
        }
    }

    pub(super) mod exact_selection_case {
        use super::*;

        #[test]
        fn selected_target() {
            if fixture_mode() == Some("exact-selection") {
                crate::common::with_timeout(1, record_exact_selection);
            }
        }

        #[test]
        fn selected_target_suffix() {
            if fixture_mode() == Some("exact-selection") {
                let marker = marker_from_env(EXACT_FORBIDDEN_MARKER_ENV);
                std::fs::write(marker, "forbidden\n").expect("write forbidden exact marker");
            }
        }
    }

    pub(super) mod permit_release_case {
        use super::*;

        #[test]
        fn a_times_out_holding_permit() {
            if fixture_mode() == Some("permit-release") {
                crate::common::with_timeout(1, || {
                    write_release_marker("timeout-started");
                    thread::sleep(Duration::from_secs(30));
                });
            }
        }

        #[test]
        fn b_holds_other_permit() {
            if fixture_mode() == Some("permit-release") {
                crate::common::with_timeout(5, || {
                    write_release_marker("holder-started");
                    wait_for_release_marker("waiting-complete");
                });
            }
        }

        #[test]
        fn c_waits_for_released_permit() {
            if fixture_mode() == Some("permit-release") {
                wait_for_release_marker("timeout-started");
                wait_for_release_marker("holder-started");
                crate::common::with_timeout(1, || write_release_marker("waiting-complete"));
            }
        }
    }

    fn fixture_mode() -> Option<&'static str> {
        match std::env::var(FIXTURE_MODE_ENV).as_deref() {
            Ok("panic") => Some("panic"),
            Ok("timeout") => Some("timeout"),
            Ok("nested") => Some("nested"),
            Ok("concurrency") => Some("concurrency"),
            Ok("exact-selection") => Some("exact-selection"),
            Ok("permit-release") => Some("permit-release"),
            _ => None,
        }
    }

    fn record_concurrency_window() {
        let state_path = PathBuf::from(
            std::env::var_os(CONCURRENCY_STATE_ENV).expect("timeout concurrency state environment"),
        );
        update_concurrency_state(&state_path, 1);
        thread::sleep(CONCURRENCY_HOLD);
        update_concurrency_state(&state_path, -1);
    }

    fn record_exact_selection() {
        let marker = marker_from_env(EXACT_SELECTED_MARKER_ENV);
        let mut marker = OpenOptions::new()
            .create(true)
            .append(true)
            .open(marker)
            .expect("open selected exact marker");
        writeln!(marker, "selected").expect("write selected exact marker");
    }

    fn write_release_marker(name: &str) {
        std::fs::write(release_marker(name), "complete\n").expect("write permit-release marker");
    }

    fn wait_for_release_marker(name: &str) {
        let marker = release_marker(name);
        let deadline = Instant::now() + FIXTURE_MARKER_WAIT;
        while Instant::now() < deadline {
            if marker.exists() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("permit-release marker did not appear: {}", marker.display());
    }

    fn release_marker(name: &str) -> PathBuf {
        marker_from_env(PERMIT_RELEASE_DIR_ENV).join(name)
    }

    fn marker_from_env(name: &str) -> PathBuf {
        PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("{name} environment")))
    }

    fn record_sibling_completion() {
        let marker = PathBuf::from(
            std::env::var_os(SIBLING_MARKER_ENV).expect("sibling marker path environment"),
        );
        std::fs::write(marker, "complete\n").expect("record later sibling completion");
    }

    fn record_nested_closure(label: &str) {
        let marker = PathBuf::from(
            std::env::var_os(SIBLING_MARKER_ENV).expect("nested closure marker environment"),
        );
        let mut marker = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(marker)
            .expect("open nested closure marker");
        writeln!(marker, "{label}").expect("record nested closure");
    }
}
