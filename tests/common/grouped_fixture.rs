//! Process-shared Blizzard addon fixture: build once, run each test in a fork.
//!
//! The parent fixture is never mutated after it is built; every test body runs in
//! a child that inherits it copy-on-write, so each test still starts from the
//! same pristine state. This mirrors the `prefork_full_ui` contract: the parent
//! is immutable, the bytecode cache is read-only in children, and libtest keeps
//! one test per process-fork instead of one full fixture build per test.
//!
//! Forking is only attempted when the test process is single-threaded. Otherwise
//! the caller falls back to building the fixture in-process, exactly as before.
//! Platforms without `fork` always take that in-process fallback.

use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Write as _;
#[cfg(target_os = "linux")]
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(target_os = "linux")]
const CHILD_DEADLINE: Duration = Duration::from_secs(300);
#[cfg(target_os = "linux")]
const DEADLINE_ENV: &str = "WOW_SIM_GROUPED_FIXTURE_TIMEOUT_SECS";
const ENABLE_ENV: &str = "WOW_SIM_GROUPED_FIXTURE";
/// Comma-separated families verified for fork reuse. Empty means no grouping.
const FAMILIES_ENV: &str = "WOW_SIM_GROUPED_FIXTURE_FAMILIES";
const REPORT_ENV: &str = "WOW_SIM_GROUPED_FIXTURE_REPORT";

struct Parent {
    key: String,
    env: WowLuaEnv,
    loaded: Vec<String>,
}

/// Process-global fixture cache. libtest may run each test on its own thread, so
/// a thread-local cache would rebuild the fixture for every case. A family often
/// alternates between two fixture shapes, so one slot would rebuild each time.
struct GlobalParents {
    entries: VecDeque<Parent>,
}

struct GlobalParent(UnsafeCell<GlobalParents>);

fn capacity() -> usize {
    std::env::var("WOW_SIM_GROUPED_FIXTURE_CACHE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(2)
}

// SAFETY: the fixture is only used when the test process is single-threaded
// (`fork_enabled` rejects anything else), and every access is serialized by the
// single running test. Children read it immediately after `fork`.
unsafe impl Sync for GlobalParent {}

static PARENT: GlobalParent = GlobalParent(UnsafeCell::new(GlobalParents {
    entries: VecDeque::new(),
}));

fn parent_slot() -> &'static mut GlobalParents {
    // SAFETY: see the `Sync` justification above; access is single-threaded.
    unsafe { &mut *PARENT.0.get() }
}

impl GlobalParents {
    /// Return the cached fixture for `key`, or build and cache it.
    fn get_or_build(&mut self, key: &str, build: impl FnOnce() -> (WowLuaEnv, Vec<String>)) {
        if let Some(index) = self.entries.iter().position(|parent| parent.key == key) {
            let parent = self.entries.remove(index).expect("cached fixture present");
            self.entries.push_back(parent);
            return;
        }
        let (env, loaded) = build();
        while self.entries.len() >= capacity() {
            self.entries.pop_front();
        }
        self.entries.push_back(Parent { key: key.to_string(), env, loaded });
    }

    #[cfg(target_os = "linux")]
    fn current(&self) -> &Parent {
        self.entries.back().expect("grouped fixture parent")
    }
}

fn fixture_key(
    shape: &str,
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
) -> String {
    let mut key = String::new();
    let _ = write!(key, "shape={shape}|ui={}|roots={roots:?}", ui_dir.display());
    for entry in overrides {
        let _ = write!(key, "|override={}+{:?}", entry.addon, entry.extra_roots);
    }
    let _ = write!(key, "|profile={}", wow_ui_sim::client_profile::ACTIVE.subdir());
    let mut environment: Vec<(String, String)> = std::env::vars()
        .filter(|(name, _)| name.starts_with("WOW_SIM_") || name == "XDG_DATA_HOME")
        .collect();
    environment.sort();
    let _ = write!(key, "|env={environment:?}");
    key
}

/// Only Linux can fork a test child.
#[cfg(target_os = "linux")]
fn fork_available() -> Result<(), &'static str> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn fork_available() -> Result<(), &'static str> {
    Err("fork is unavailable on this platform")
}

fn fork_enabled<R: 'static>() -> Result<(), &'static str> {
    if let Err(reason) = fork_available() {
        return Err(reason);
    }
    if TypeId::of::<R>() != TypeId::of::<()>() {
        return Err("assertions return a value");
    }
    if matches!(std::env::var(ENABLE_ENV).as_deref(), Ok("0")) {
        return Err("disabled by WOW_SIM_GROUPED_FIXTURE=0");
    }
    let family = std::thread::current()
        .name()
        .unwrap_or("<unknown>")
        .split("::")
        .next()
        .unwrap_or("<unknown>")
        .to_string();
    let verified = std::env::var(FAMILIES_ENV).unwrap_or_default();
    if !verified.split(',').any(|entry| entry.trim() == family) {
        return Err("family is not verified for grouped fixtures");
    }
    if std::env::var("RUST_TEST_THREADS").as_deref() != Ok("1") {
        return Err("test process is not single-threaded");
    }
    Ok(())
}

fn record(reason: &str) {
    let Some(path) = std::env::var_os(REPORT_ENV) else {
        return;
    };
    let name = std::thread::current().name().unwrap_or("<unknown>").to_string();
    let line = format!("{name}\t{reason}\n");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[cfg(target_os = "linux")]
fn child_deadline() -> Duration {
    std::env::var(DEADLINE_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(CHILD_DEADLINE)
}

/// Run `assertions` against a fixture built once per process for this key.
pub fn with_fixture<R, B, F>(
    shape: &str,
    ui_dir: &Path,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
    build: B,
    assertions: F,
) -> R
where
    R: Default + 'static,
    B: FnOnce() -> (WowLuaEnv, Vec<String>),
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    if let Err(reason) = fork_enabled::<R>() {
        record(reason);
        let (env, loaded) = build();
        return assertions(&env, &loaded);
    }
    let key = fixture_key(shape, ui_dir, roots, overrides);
    let mut build = Some(build);
    parent_slot().get_or_build(&key, || {
        (build.take().expect("grouped fixture builds once"))()
    });
    drop(build);
    run_in_child(&key, assertions)
}

/// Run a body against a fixture built once per process, without the `loaded` list.
pub fn with_env<R, B, F>(shape: &str, build: B, body: F) -> R
where
    R: Default + 'static,
    B: FnOnce() -> WowLuaEnv,
    F: FnOnce(&WowLuaEnv) -> R,
{
    with_fixture(
        shape,
        Path::new("grouped-env"),
        &[],
        &[],
        || (build(), Vec::new()),
        |env, _| body(env),
    )
}

/// Fork, run `assertions` in the child, and report its status against this test.
#[cfg(target_os = "linux")]
fn run_in_child<R, F>(key: &str, assertions: F) -> R
where
    R: Default,
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    let name = std::thread::current().name().unwrap_or("<unknown>").to_string();
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    // SAFETY: the caller guarantees a single-threaded test process, so no other
    // thread can hold a lock at fork time. The child only runs `assertions`.
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        wow_ui_sim::loader::enter_bytecode_cache_read_only_mode();
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let parent = parent_slot().current();
            assertions(&parent.env, &parent.loaded);
        }));
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        let code = match outcome {
            Ok(()) => 0,
            Err(_) => {
                eprintln!("[grouped fixture] child panic for test `{name}` (fixture {key})");
                101
            }
        };
        // SAFETY: skip atexit handlers and parent-owned buffers in the child.
        unsafe { libc::_exit(code) };
    }
    if pid < 0 {
        record("fork failed");
        return R::default();
    }
    let deadline = Instant::now() + child_deadline();
    let mut status = 0;
    loop {
        // SAFETY: `pid` is our child and `status` is a valid out-parameter.
        let waited = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if waited == pid {
            break;
        }
        if waited < 0 {
            panic!("grouped fixture: waitpid failed for test `{name}`");
        }
        if Instant::now() >= deadline {
            // SAFETY: killing our own timed-out child before reaping it.
            unsafe { libc::kill(pid, libc::SIGKILL) };
            // SAFETY: reaping the child we just killed.
            unsafe { libc::waitpid(pid, &mut status, 0) };
            panic!("grouped fixture: child for test `{name}` exceeded {:?}", child_deadline());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let exited_ok = libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0;
    if !exited_ok {
        let detail = if libc::WIFSIGNALED(status) {
            format!("signal {}", libc::WTERMSIG(status))
        } else {
            format!("exit {}", libc::WEXITSTATUS(status))
        };
        panic!("grouped fixture: child for test `{name}` failed ({detail}, fixture {key})");
    }
    R::default()
}

/// `fork_available` rejects every non-Linux process, so `with_fixture` builds the
/// fixture in-process and never reaches this path.
#[cfg(not(target_os = "linux"))]
fn run_in_child<R, F>(_key: &str, _assertions: F) -> R
where
    R: Default,
    F: FnOnce(&WowLuaEnv, &[String]) -> R,
{
    unreachable!("grouped fixture: fork is unavailable on this platform")
}
