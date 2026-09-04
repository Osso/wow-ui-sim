#[cfg(not(target_os = "linux"))]
compile_error!("prefork_full_ui is Linux-only");

#[path = "common/prefork.rs"]
mod prefork;
#[path = "common/prefork_full_ui_preload.rs"]
mod prefork_full_ui_preload;
#[path = "common/workload_gate.rs"]
mod prefork_workload_gate;
#[path = "test_keybindings_panels_detail.rs"]
mod test_keybindings_panels_detail;

include!(concat!(env!("OUT_DIR"), "/integration_tests.rs"));

use prefork::{Case, Config};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use wow_ui_sim::loader::{
    enter_bytecode_cache_parent_bypass_mode, enter_bytecode_cache_read_only_mode, load_addon,
    release_prefork_parent_bytecode_cache_memory,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const CONFORMANCE_MODE_ENV: &str = "PREFORK_CONFORMANCE_SUITE";
const DRIVER_MODE_ENV: &str = "PREFORK_CONFORMANCE_DRIVER";
const TREE_CHILD_MODE_ENV: &str = "PREFORK_CONFORMANCE_TREE_CHILD";
const START_THREAD_ENV: &str = "PREFORK_CONFORMANCE_START_THREAD";
const WORKER_STATE_ENV: &str = "PREFORK_CONFORMANCE_WORKER_STATE";
const TREE_PID_ENV: &str = "PREFORK_CONFORMANCE_TREE_PID";
const DRIVER_TIMEOUT_MS_ENV: &str = "PREFORK_CONFORMANCE_TIMEOUT_MS";
const DRIVER_TERM_GRACE_MS_ENV: &str = "PREFORK_CONFORMANCE_TERM_GRACE_MS";
const DRIVER_SETUP_MARKER_ENV: &str = "PREFORK_CONFORMANCE_SETUP_MARKER";
const DRIVER_SETUP_FAILURE_ENV: &str = "PREFORK_CONFORMANCE_SETUP_FAILURE";
const CLEANUP_REPORT_ENV: &str = "PREFORK_CONFORMANCE_CLEANUP_REPORT";
const BYTECODE_DRIVER_ENV: &str = "PREFORK_CONFORMANCE_BYTECODE_DRIVER";
const BYTECODE_PARENT_BYPASS_DRIVER_ENV: &str = "PREFORK_CONFORMANCE_BYTECODE_PARENT_BYPASS_DRIVER";
const WORKER_HOLD: Duration = Duration::from_millis(40);
const RESOURCE_SHORT_HOLD: Duration = Duration::from_secs(1);
const RESOURCE_LONG_HOLD: Duration = Duration::from_secs(30);
const RESOURCE_CHILD_WAIT: Duration = Duration::from_secs(2);
const PROCESS_DEATH_WAIT: Duration = Duration::from_secs(2);
const FD_SCAN_LIMIT: i32 = 256;

struct ConformanceState {
    executable: PathBuf,
}

struct FixtureState {
    cow_value: AtomicUsize,
}

struct BytecodeFixtureState {
    env: WowLuaEnv,
    child_toc: PathBuf,
    expected_value: String,
}

#[derive(Debug, PartialEq, Eq)]
struct CachePathSnapshot {
    relative_path: PathBuf,
    is_directory: bool,
    is_file: bool,
    len: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    inode: u64,
    mode: u32,
    bytes: Option<Vec<u8>>,
}

static BYTECODE_CHILD_SETUP_RAN: AtomicBool = AtomicBool::new(false);

const CONFORMANCE_CASES: &[Case<ConformanceState>] = &[
    Case::new("conformance::filtering_and_listing", filtering_and_listing),
    Case::new(
        "conformance::generated_registry_lists_nested_marker_case",
        generated_registry_lists_nested_marker_case,
    ),
    Case::new(
        "conformance::generated_registry_lists_path_declared_marker_case",
        generated_registry_lists_path_declared_marker_case,
    ),
    Case::new(
        "conformance::generated_registry_lists_foundation_batch",
        generated_registry_lists_foundation_batch,
    ),
    Case::new(
        "conformance::generated_registry_lists_post_start_explicit_batch",
        generated_registry_lists_post_start_explicit_batch,
    ),
    Case::new(
        "conformance::generated_registry_lists_post_start_housing_batch",
        generated_registry_lists_post_start_housing_batch,
    ),
    Case::new(
        "conformance::generated_registry_lists_post_start_generic_batch",
        generated_registry_lists_post_start_generic_batch,
    ),
    Case::new(
        "conformance::environment_cleanup_restore_errors_are_contextual",
        environment_cleanup_restore_errors_are_contextual,
    ),
    Case::new("conformance::copy_on_write", copy_on_write),
    Case::new("conformance::panic_and_capture", panic_and_capture),
    Case::new("conformance::nocapture", nocapture),
    Case::new(
        "conformance::timeout_and_tree_cleanup",
        timeout_and_tree_cleanup,
    ),
    Case::new("conformance::exit_classification", exit_classification),
    Case::new("conformance::bounded_workers", bounded_workers),
    Case::new(
        "conformance::rejects_multithreaded_fork",
        rejects_multithreaded_fork,
    ),
    Case::new("conformance::rejects_bad_arguments", rejects_bad_arguments),
    Case::new(
        "conformance::validates_worker_counts",
        validates_worker_counts,
    ),
    Case::new(
        "conformance::cleans_up_after_resource_failure",
        cleans_up_after_resource_failure,
    ),
    Case::new(
        "conformance::read_only_bytecode_cache_child",
        read_only_bytecode_cache_child,
    ),
    Case::new(
        "conformance::parent_bypass_bytecode_cache",
        parent_bypass_bytecode_cache,
    ),
];

const BYTECODE_FIXTURE_CASES: &[Case<BytecodeFixtureState>] = &[Case::new(
    "bytecode::read_only_child",
    fixture_read_only_bytecode_cache,
)];

const MANUAL_FULL_UI_CASES: &[Case<WowLuaEnv>] = &[
    Case::new(
        "test_keybindings_panels_detail::keybind_m_opens_world_map",
        test_keybindings_panels_detail::keybind_m_opens_world_map,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_floor_dropdown_hidden_without_subzone_or_map_group",
        test_keybindings_panels_detail::world_map_floor_dropdown_hidden_without_subzone_or_map_group,
    ),
    Case::new(
        "test_keybindings_panels_detail::keybind_m_toggles_world_map_without_errors",
        test_keybindings_panels_detail::keybind_m_toggles_world_map_without_errors,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_title_text_is_non_empty_after_opening",
        test_keybindings_panels_detail::world_map_title_text_is_non_empty_after_opening,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_exploration_pin_has_visible_overlay_textures_after_opening",
        test_keybindings_panels_detail::world_map_exploration_pin_has_visible_overlay_textures_after_opening,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_exploration_pin_converges_visible_after_onupdate_ticks",
        test_keybindings_panels_detail::world_map_exploration_pin_converges_visible_after_onupdate_ticks,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_exploration_pin_first_open_settles_without_reopen",
        test_keybindings_panels_detail::world_map_exploration_pin_first_open_settles_without_reopen,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_exploration_pin_recovers_when_first_overlay_fetch_is_empty",
        test_keybindings_panels_detail::world_map_exploration_pin_recovers_when_first_overlay_fetch_is_empty,
    ),
    Case::new(
        "test_keybindings_panels_detail::world_map_registers_fog_of_war_pin_template_as_fog_of_war_frame",
        test_keybindings_panels_detail::world_map_registers_fog_of_war_pin_template_as_fog_of_war_frame,
    ),
];

const GENERATED_FULL_UI_CASES: &[Case<WowLuaEnv>] =
    include!(concat!(env!("OUT_DIR"), "/prefork_full_ui_cases.rs"));

fn full_ui_cases() -> Vec<Case<WowLuaEnv>> {
    MANUAL_FULL_UI_CASES
        .iter()
        .chain(GENERATED_FULL_UI_CASES)
        .map(|case| Case::new(case.name, case.test))
        .collect()
}

const FIXTURE_CASES: &[Case<FixtureState>] = &[
    Case::new("alpha::one", fixture_pass),
    Case::new("alpha::two", fixture_pass),
    Case::new("beta::one", fixture_pass),
    Case::new("cow::mutate", fixture_cow_mutate),
    Case::new("cow::observe", fixture_cow_observe),
    Case::new("output::pass", fixture_output_pass),
    Case::new("output::panic", fixture_output_panic),
    Case::new("process::signal", fixture_signal),
    Case::new("process::exit", fixture_exit),
    Case::new("process::timeout_tree", fixture_timeout_tree),
    Case::new("worker::00", fixture_worker),
    Case::new("worker::01", fixture_worker),
    Case::new("worker::02", fixture_worker),
    Case::new("worker::03", fixture_worker),
    Case::new("worker::04", fixture_worker),
    Case::new("worker::05", fixture_worker),
    Case::new("worker::06", fixture_worker),
    Case::new("worker::07", fixture_worker),
    Case::new("resource::hold", fixture_resource_hold),
    Case::new("resource::release", fixture_resource_release),
    Case::new("resource::pending", fixture_pass),
];

fn main() -> ExitCode {
    if env::var_os(TREE_CHILD_MODE_ENV).is_some() {
        run_tree_child();
    }
    if env::var_os(DRIVER_MODE_ENV).is_some() {
        return run_fixture_driver();
    }
    if env::var_os(CONFORMANCE_MODE_ENV).is_some() {
        return run_conformance_suite();
    }

    let config = Config {
        timeout: Duration::from_secs(120),
        child_setup: wow_ui_sim::loader::enter_bytecode_cache_read_only_mode,
        ..Config::default()
    };
    let full_ui_cases = full_ui_cases();
    prefork_workload_gate::with_lock(prefork_workload_gate::Mode::Shared, || {
        prefork::run_with_setup(&full_ui_cases, config, || {
            run_conformance_subprocess()?;
            prefork_full_ui_preload::preload_full_game_ui()
        })
    })
}

fn run_conformance_suite() -> ExitCode {
    let state = ConformanceState {
        executable: env::current_exe().expect("resolve prefork conformance executable"),
    };
    prefork::run(&state, CONFORMANCE_CASES, Config::default())
}

fn run_conformance_subprocess() -> Result<(), String> {
    let executable = env::current_exe()
        .map_err(|error| format!("resolve prefork conformance executable: {error}"))?;
    let output = Command::new(executable)
        .env(CONFORMANCE_MODE_ENV, "1")
        .env_remove(DRIVER_MODE_ENV)
        .env_remove(TREE_CHILD_MODE_ENV)
        .output()
        .map_err(|error| format!("run prefork conformance subprocess: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    std::io::stdout()
        .write_all(&output.stdout)
        .map_err(|error| format!("print conformance stdout: {error}"))?;
    std::io::stderr()
        .write_all(&output.stderr)
        .map_err(|error| format!("print conformance stderr: {error}"))?;
    Err(format!(
        "prefork conformance subprocess failed with {}",
        output.status
    ))
}

fn run_fixture_driver() -> ExitCode {
    if env::var_os(BYTECODE_PARENT_BYPASS_DRIVER_ENV).is_some() {
        return run_parent_bypass_bytecode_driver();
    }
    if env::var_os(BYTECODE_DRIVER_ENV).is_some() {
        return run_bytecode_driver();
    }

    let thread_guard = start_optional_guard_thread();
    let config = Config {
        timeout: duration_from_env(DRIVER_TIMEOUT_MS_ENV, 2_000),
        term_grace: duration_from_env(DRIVER_TERM_GRACE_MS_ENV, 100),
        ..Config::default()
    };
    let cleanup_report = prepare_cleanup_report();
    let result = prefork::run_with_setup(FIXTURE_CASES, config, || {
        if env::var_os(DRIVER_SETUP_FAILURE_ENV).is_some() {
            return Err("deliberate fixture setup failure".to_string());
        }
        if let Some(marker_path) = env::var_os(DRIVER_SETUP_MARKER_ENV) {
            std::fs::write(marker_path, "setup ran").map_err(|error| error.to_string())?;
        }
        Ok(FixtureState {
            cow_value: AtomicUsize::new(7),
        })
    });
    write_cleanup_report(cleanup_report);
    drop(thread_guard);
    result
}

fn run_bytecode_driver() -> ExitCode {
    let addon_dir = TempDir::new().expect("create bytecode conformance addon directory");
    let unique = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos()
    );
    let prewarm_toc = write_lua_addon(
        addon_dir.path(),
        "PreforkBytecodePrewarm",
        &format!("_G.__prefork_bytecode_prewarm = {unique:?}"),
    );
    let child_toc = write_lua_addon(
        addon_dir.path(),
        "PreforkBytecodeChild",
        &format!("_G.__prefork_bytecode_child = {unique:?}"),
    );
    let parent_after_toc = write_lua_addon(
        addon_dir.path(),
        "PreforkBytecodeParentAfter",
        &format!("_G.__prefork_bytecode_parent_after = {unique:?}"),
    );

    let env = WowLuaEnv::new().expect("create bytecode conformance Lua environment");
    load_addon(&env.loader_env(), &prewarm_toc).expect("prewarm parent bytecode cache");
    let cache_root = PathBuf::from(env::var_os("XDG_CACHE_HOME").expect("XDG cache root"))
        .join("wow-ui-sim")
        .join("lua-bytecode");
    let before = snapshot_cache_tree(&cache_root);
    assert!(
        before.iter().any(|entry| entry
            .relative_path
            .file_name()
            .is_some_and(|name| name == "pack.bin")),
        "parent prewarm must create pack.bin"
    );
    let released_bytes = release_prefork_parent_bytecode_cache_memory()
        .expect("release prewarmed parent bytecode cache memory");
    assert!(
        released_bytes > 0,
        "parent prewarm must release non-empty in-memory bytecode pack state"
    );

    let state = BytecodeFixtureState {
        env,
        child_toc,
        expected_value: unique,
    };
    let config = Config {
        timeout: duration_from_env(DRIVER_TIMEOUT_MS_ENV, 2_000),
        term_grace: duration_from_env(DRIVER_TERM_GRACE_MS_ENV, 100),
        child_setup: enable_read_only_bytecode_cache_in_child,
    };
    let result = prefork::run(&state, BYTECODE_FIXTURE_CASES, config);
    if result != ExitCode::SUCCESS {
        return result;
    }

    assert!(
        !BYTECODE_CHILD_SETUP_RAN.load(Ordering::SeqCst),
        "child setup mutation must not reach parent process state"
    );
    assert_eq!(
        snapshot_cache_tree(&cache_root),
        before,
        "read-only child must leave cache bytes, metadata, and directory contents unchanged"
    );

    load_addon(&state.env.loader_env(), &parent_after_toc)
        .expect("parent bytecode cache must remain writable");
    assert_ne!(
        snapshot_cache_tree(&cache_root),
        before,
        "child read-only mode must not change parent cache mode"
    );
    result
}

fn run_parent_bypass_bytecode_driver() -> ExitCode {
    let addon_dir = TempDir::new().expect("create parent-bypass addon directory");
    let unique = format!("parent-bypass-{}", std::process::id());
    let parent_toc = write_lua_addon(
        addon_dir.path(),
        "PreforkBytecodeParentBypass",
        &format!("_G.__prefork_bytecode_parent_bypass = {unique:?}"),
    );
    let child_toc = write_lua_addon(
        addon_dir.path(),
        "PreforkBytecodeBypassChild",
        &format!("_G.__prefork_bytecode_child = {unique:?}"),
    );
    let cache_root = PathBuf::from(env::var_os("XDG_CACHE_HOME").expect("XDG cache root"))
        .join("wow-ui-sim")
        .join("lua-bytecode");
    let before = snapshot_cache_tree(&cache_root);
    assert!(
        before.iter().any(|entry| entry
            .relative_path
            .file_name()
            .is_some_and(|name| name == "pack.bin")),
        "parent-bypass fixture requires an existing pack"
    );

    enter_bytecode_cache_parent_bypass_mode();
    let env = WowLuaEnv::new().expect("create parent-bypass Lua environment");
    load_addon(&env.loader_env(), &parent_toc).expect("compile parent source with cache bypassed");
    assert_eq!(
        env.eval::<String>("return __prefork_bytecode_parent_bypass")
            .expect("read parent-bypass addon value"),
        unique
    );
    assert_eq!(
        release_prefork_parent_bytecode_cache_memory()
            .expect("release bypassed parent bytecode cache memory"),
        0,
        "bypassed parent must not load pack bytes"
    );
    assert_eq!(
        snapshot_cache_tree(&cache_root),
        before,
        "parent bypass must not read-and-repair or write the cache tree"
    );

    let state = BytecodeFixtureState {
        env,
        child_toc,
        expected_value: unique,
    };
    let config = Config {
        timeout: duration_from_env(DRIVER_TIMEOUT_MS_ENV, 2_000),
        term_grace: duration_from_env(DRIVER_TERM_GRACE_MS_ENV, 100),
        child_setup: enable_read_only_bytecode_cache_in_child,
    };
    let result = prefork::run(&state, BYTECODE_FIXTURE_CASES, config);
    assert_eq!(
        snapshot_cache_tree(&cache_root),
        before,
        "read-only child from bypassed parent must leave the cache tree unchanged"
    );
    result
}

fn write_lua_addon(root: &Path, name: &str, source: &str) -> PathBuf {
    let addon_dir = root.join(name);
    std::fs::create_dir(&addon_dir).expect("create bytecode conformance addon directory");
    let toc_path = addon_dir.join(format!("{name}.toc"));
    let lua_name = format!("{name}.lua");
    std::fs::write(&toc_path, format!("## Interface: 120000\n{lua_name}\n"))
        .expect("write bytecode conformance TOC");
    std::fs::write(addon_dir.join(lua_name), source).expect("write bytecode conformance Lua file");
    toc_path
}

fn snapshot_cache_tree(root: &Path) -> Vec<CachePathSnapshot> {
    let mut snapshots = Vec::new();
    snapshot_cache_directory(root, root, &mut snapshots);
    snapshots.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    snapshots
}

fn snapshot_cache_directory(root: &Path, directory: &Path, snapshots: &mut Vec<CachePathSnapshot>) {
    for entry in std::fs::read_dir(directory).expect("read bytecode cache directory") {
        let path = entry.expect("read bytecode cache entry").path();
        let metadata = std::fs::symlink_metadata(&path).expect("read bytecode cache metadata");
        let is_directory = metadata.is_dir();
        let is_file = metadata.is_file();
        let bytes = is_file.then(|| std::fs::read(&path).expect("read bytecode cache file"));
        snapshots.push(CachePathSnapshot {
            relative_path: path
                .strip_prefix(root)
                .expect("cache path below root")
                .to_path_buf(),
            is_directory,
            is_file,
            len: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            bytes,
        });
        if is_directory {
            snapshot_cache_directory(root, &path, snapshots);
        }
    }
}

fn enable_read_only_bytecode_cache_in_child() {
    BYTECODE_CHILD_SETUP_RAN.store(true, Ordering::SeqCst);
    enter_bytecode_cache_read_only_mode();
}

fn prepare_cleanup_report() -> Option<(File, Vec<i32>)> {
    let path = env::var_os(CLEANUP_REPORT_ENV)?;
    let file = File::create(path).expect("create cleanup report");
    let open_fds = open_fd_numbers();
    Some((file, open_fds))
}

fn write_cleanup_report(report: Option<(File, Vec<i32>)>) {
    let Some((mut file, before)) = report else {
        return;
    };
    let after = open_fd_numbers();
    writeln!(file, "before={before:?}").expect("write cleanup report baseline");
    writeln!(file, "after={after:?}").expect("write cleanup report result");
}

fn open_fd_numbers() -> Vec<i32> {
    (0..FD_SCAN_LIMIT)
        .filter(|fd| {
            let result = unsafe { libc::fcntl(*fd, libc::F_GETFD) };
            if result != -1 {
                return true;
            }
            let error = std::io::Error::last_os_error();
            assert_eq!(error.raw_os_error(), Some(libc::EBADF), "inspect fd {fd}");
            false
        })
        .collect()
}

fn duration_from_env(name: &str, default_ms: u64) -> Duration {
    let milliseconds = env::var(name)
        .ok()
        .map(|value| {
            value
                .parse()
                .expect("duration environment value must be an integer")
        })
        .unwrap_or(default_ms);
    Duration::from_millis(milliseconds)
}

fn start_optional_guard_thread() -> Option<std::thread::JoinHandle<()>> {
    if env::var_os(START_THREAD_ENV).is_none() {
        return None;
    }

    let barrier = Arc::new(Barrier::new(2));
    let child_barrier = Arc::clone(&barrier);
    let handle = std::thread::spawn(move || {
        child_barrier.wait();
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    });
    barrier.wait();
    Some(handle)
}

fn filtering_and_listing(state: &ConformanceState) {
    let temp = TempDir::new().expect("create lazy setup marker temp dir");
    let setup_marker = temp.path().join("setup-ran");
    let substring = run_driver_with_os_env(
        state,
        ["--list", "alpha"],
        [(DRIVER_SETUP_MARKER_ENV, setup_marker.as_os_str())],
    );
    assert!(
        !setup_marker.exists(),
        "listing must not invoke expensive state setup"
    );
    assert_success(&substring);
    assert_eq!(
        stdout(&substring),
        "alpha::one: test\nalpha::two: test\n\n2 tests, 0 benchmarks\n"
    );

    let exact = run_driver(state, ["alpha::one", "--exact", "--list"], []);
    assert_success(&exact);
    assert_eq!(stdout(&exact), "alpha::one: test\n\n1 test, 0 benchmarks\n");

    let skipped = run_driver(state, ["--list", "alpha", "--skip=two"], []);
    assert_success(&skipped);
    assert_eq!(
        stdout(&skipped),
        "alpha::one: test\n\n1 test, 0 benchmarks\n"
    );

    let split_skip = run_driver(state, ["--list", "--skip", "alpha"], []);
    assert_success(&split_skip);
    assert!(!stdout(&split_skip).contains("alpha::"));

    let repeated_skip = run_driver(state, ["--list", "--skip", "alpha", "--skip=beta"], []);
    assert_success(&repeated_skip);
    assert!(!stdout(&repeated_skip).contains("alpha::"));
    assert!(!stdout(&repeated_skip).contains("beta::"));

    let zero_match_marker = temp.path().join("zero-match-setup-ran");
    let zero_match = run_driver_with_os_env(
        state,
        ["definitely_no_matching_fixture_case"],
        [(DRIVER_SETUP_MARKER_ENV, zero_match_marker.as_os_str())],
    );
    assert_success(&zero_match);
    assert_eq!(
        stdout(&zero_match),
        "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 total\n"
    );
    assert!(
        !zero_match_marker.exists(),
        "zero-match execution must not invoke expensive state setup"
    );
}

fn generated_registry_lists_nested_marker_case(state: &ConformanceState) {
    const CASE_NAME: &str =
        "prefork_full_ui_nested::fixture::preloaded_parent_has_normal_game_startup";
    let output = Command::new(&state.executable)
        .args(["--list", CASE_NAME, "--exact"])
        .env_remove(CONFORMANCE_MODE_ENV)
        .env_remove(DRIVER_MODE_ENV)
        .env_remove(TREE_CHILD_MODE_ENV)
        .output()
        .expect("list generated prefork registry case");

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{CASE_NAME}: test\n\n1 test, 0 benchmarks\n")
    );
}

fn generated_registry_lists_path_declared_marker_case(state: &ConformanceState) {
    const CASE_NAME: &str =
        "blizzard_ui_blizzard_addonlist::surface_globals::addon_list_publishes_module_constants";
    let output = Command::new(&state.executable)
        .args(["--list", CASE_NAME, "--exact"])
        .env_remove(CONFORMANCE_MODE_ENV)
        .env_remove(DRIVER_MODE_ENV)
        .env_remove(TREE_CHILD_MODE_ENV)
        .output()
        .expect("list generated path-declared prefork registry case");

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{CASE_NAME}: test\n\n1 test, 0 benchmarks\n")
    );
}

fn generated_registry_lists_foundation_batch(state: &ConformanceState) {
    const MODULE_COUNTS: &[(&str, usize)] = &[
        ("blizzard_shared_xml_base_loads::", 16),
        ("blizzard_shared_xml_loads::", 13),
        ("blizzard_shared_talent_ui_loads::", 11),
        ("blizzard_shared_xml_game_loads::", 10),
        ("blizzard_shared_widget_frames_loads::", 9),
        ("blizzard_foundation_lane::", 4),
        ("blizzard_core_frame_lane::", 5),
    ];

    let mut total = 0;
    for (module_filter, expected_count) in MODULE_COUNTS {
        let output = Command::new(&state.executable)
            .args(["--list", module_filter])
            .env_remove(CONFORMANCE_MODE_ENV)
            .env_remove(DRIVER_MODE_ENV)
            .env_remove(TREE_CHILD_MODE_ENV)
            .output()
            .expect("list generated foundation prefork cases");

        assert_success(&output);
        let listed = stdout(&output);
        let actual_count = listed
            .lines()
            .filter(|line| line.ends_with(": test"))
            .count();
        assert_eq!(
            actual_count, *expected_count,
            "unexpected generated case count for {module_filter}:\n{listed}"
        );
        assert!(
            listed.ends_with(&format!("\n{expected_count} tests, 0 benchmarks\n")),
            "unexpected libtest-compatible summary for {module_filter}:\n{listed}"
        );
        total += actual_count;
    }
    assert_eq!(total, 68);
}

fn generated_registry_lists_post_start_explicit_batch(state: &ConformanceState) {
    const MODULE_COUNTS: &[(&str, usize)] = &[
        ("blizzard_hud_inventory_templates_loads::", 11),
        ("blizzard_item_belt_frame_loads::", 8),
        ("blizzard_spell_pick_up_indicator_loads::", 10),
        ("blizzard_reforging_ui_loads::", 8),
        ("blizzard_remix_artifact_tutorial_ui_loads::", 7),
        ("blizzard_remix_artifact_ui_loads::", 9),
        ("blizzard_spectate_frame_loads::", 13),
        ("blizzard_world_loot_object_list_loads::", 8),
        ("blizzard_wow_survey_ui_loads::", 9),
        ("blizzard_match_celebration_party_pose_ui_loads::", 13),
        ("blizzard_spell_search_loads::", 15),
    ];

    let mut total = 0;
    for (module_filter, expected_count) in MODULE_COUNTS {
        let output = Command::new(&state.executable)
            .args(["--list", module_filter])
            .env_remove(CONFORMANCE_MODE_ENV)
            .env_remove(DRIVER_MODE_ENV)
            .env_remove(TREE_CHILD_MODE_ENV)
            .output()
            .expect("list generated post-start explicit prefork cases");

        assert_success(&output);
        let listed = stdout(&output);
        let actual_count = listed
            .lines()
            .filter(|line| line.ends_with(": test"))
            .count();
        assert_eq!(
            actual_count, *expected_count,
            "unexpected generated case count for {module_filter}:\n{listed}"
        );
        assert!(
            listed.ends_with(&format!("\n{expected_count} tests, 0 benchmarks\n")),
            "unexpected libtest-compatible summary for {module_filter}:\n{listed}"
        );
        total += actual_count;
    }
    assert_eq!(total, 111);
}

fn generated_registry_lists_post_start_housing_batch(state: &ConformanceState) {
    const MODULE_COUNTS: &[(&str, usize)] = &[
        ("blizzard_housing_bulletin_board_loads::", 15),
        ("blizzard_housing_create_neighborhood_loads::", 18),
        ("blizzard_housing_dashboard_loads::", 26),
        ("blizzard_housing_house_finder_loads::", 20),
        ("blizzard_house_editor_loads::", 14),
        ("blizzard_house_list_loads::", 10),
        ("blizzard_housing_charter_loads::", 13),
        ("blizzard_housing_controls_loads::", 12),
        ("blizzard_housing_cornerstone_loads::", 20),
        ("blizzard_housing_inspect_mode_ui_loads::", 8),
        ("blizzard_housing_market_cart_loads::", 10),
        ("blizzard_housing_model_preview_loads::", 10),
        ("blizzard_housing_house_settings_loads::", 13),
    ];

    let mut total = 0;
    for (module_filter, expected_count) in MODULE_COUNTS {
        let output = Command::new(&state.executable)
            .args(["--list", module_filter])
            .env_remove(CONFORMANCE_MODE_ENV)
            .env_remove(DRIVER_MODE_ENV)
            .env_remove(TREE_CHILD_MODE_ENV)
            .output()
            .expect("list generated post-start housing prefork cases");

        assert_success(&output);
        let listed = stdout(&output);
        let actual_count = listed
            .lines()
            .filter(|line| line.ends_with(": test"))
            .count();
        assert_eq!(
            actual_count, *expected_count,
            "unexpected generated case count for {module_filter}:\n{listed}"
        );
        assert!(
            listed.ends_with(&format!("\n{expected_count} tests, 0 benchmarks\n")),
            "unexpected libtest-compatible summary for {module_filter}:\n{listed}"
        );
        total += actual_count;
    }
    assert_eq!(total, 189);
}

fn generated_registry_lists_post_start_generic_batch(state: &ConformanceState) {
    const MODULE_COUNTS: &[(&str, usize)] = &[
        ("blizzard_gm_chat_ui_loads::", 6),
        ("blizzard_guild_bank_ui_loads::", 7),
        ("blizzard_guild_control_ui_loads::", 7),
        ("blizzard_hybrid_minimap_loads::", 7),
        ("blizzard_inspect_ui_loads::", 11),
        ("blizzard_islands_party_pose_ui_loads::", 9),
        ("blizzard_islands_queue_ui_loads::", 8),
        ("blizzard_item_interaction_ui_loads::", 12),
        ("blizzard_item_socketing_ui_loads::", 10),
        ("blizzard_item_upgrade_ui_loads::", 14),
        ("blizzard_kiosk_loads::", 12),
        ("blizzard_landing_soulbinds_loads::", 7),
        ("blizzard_macro_ui_loads::", 11),
        ("blizzard_generic_trait_ui_loads::", 12),
        ("blizzard_warfronts_party_pose_ui_loads::", 9),
        ("blizzard_weekly_rewards_loads::", 10),
        ("blizzard_move_pad_loads::", 7),
    ];

    let mut total = 0;
    for (module_filter, expected_count) in MODULE_COUNTS {
        let output = Command::new(&state.executable)
            .args(["--list", module_filter])
            .env_remove(CONFORMANCE_MODE_ENV)
            .env_remove(DRIVER_MODE_ENV)
            .env_remove(TREE_CHILD_MODE_ENV)
            .output()
            .expect("list generated post-start generic prefork cases");

        assert_success(&output);
        let listed = stdout(&output);
        let actual_count = listed
            .lines()
            .filter(|line| line.ends_with(": test"))
            .count();
        assert_eq!(
            actual_count, *expected_count,
            "unexpected generated case count for {module_filter}:\n{listed}"
        );
        assert!(
            listed.ends_with(&format!("\n{expected_count} tests, 0 benchmarks\n")),
            "unexpected libtest-compatible summary for {module_filter}:\n{listed}"
        );
        total += actual_count;
    }
    assert_eq!(total, 159);
}

fn environment_cleanup_restore_errors_are_contextual(_: &ConformanceState) {
    let temp = TempDir::new().expect("create EnvironmentCleanup failure fixture");
    let addon_dir = temp.path().join("Blizzard_EnvironmentCleanup");
    std::fs::create_dir(&addon_dir).expect("create EnvironmentCleanup addon directory");
    let toc_path = addon_dir.join("Blizzard_EnvironmentCleanup.toc");
    std::fs::write(
        &toc_path,
        "## Interface: 120000\nBlizzard_EnvironmentCleanup.lua\n",
    )
    .expect("write EnvironmentCleanup fixture TOC");
    std::fs::write(
        addon_dir.join("Blizzard_EnvironmentCleanup.lua"),
        "setmetatable = nil\n",
    )
    .expect("write EnvironmentCleanup failure fixture");

    let env = WowLuaEnv::new().expect("create EnvironmentCleanup fixture environment");
    let error = prefork_full_ui_preload::load_blizzard_addon(
        &env,
        "Blizzard_EnvironmentCleanup",
        &toc_path,
    )
    .expect_err("post-cleanup restore failure must fail preload");
    assert!(
        error.contains(
            "restore post-cleanup globals after Blizzard addon `Blizzard_EnvironmentCleanup`"
        ),
        "missing addon restore context: {error}"
    );
}

fn copy_on_write(state: &ConformanceState) {
    let output = run_driver(state, ["cow::", "--test-threads", "1", "--nocapture"], []);
    assert_success(&output);
}

fn panic_and_capture(state: &ConformanceState) {
    let output = run_driver(state, ["output::panic", "--exact"], []);
    assert_failure(&output);
    let text = stdout(&output);
    assert!(text.contains("FAILED (panic: deliberate panic text)"));
    assert!(text.contains("---- output::panic stdout ----\nchild stdout marker"));
    assert!(text.contains("---- output::panic stderr ----"));
    assert!(text.contains("child stderr marker"));
}

fn nocapture(state: &ConformanceState) {
    let captured = run_driver(state, ["output::pass", "--exact"], []);
    assert_success(&captured);
    assert!(!stdout(&captured).contains("child stdout marker"));
    assert!(!stderr(&captured).contains("child stderr marker"));

    let inherited = run_driver(state, ["output::pass", "--exact", "--nocapture"], []);
    assert_success(&inherited);
    assert!(stdout(&inherited).contains("child stdout marker"));
    assert!(stderr(&inherited).contains("child stderr marker"));
}

fn timeout_and_tree_cleanup(state: &ConformanceState) {
    let temp = TempDir::new().expect("create process tree temp dir");
    let pid_path = temp.path().join("grandchild.pid");
    let timeout = [
        (TREE_PID_ENV, pid_path.as_os_str()),
        (DRIVER_TIMEOUT_MS_ENV, "120".as_ref()),
    ];
    let output = run_driver_with_os_env(state, ["process::timeout_tree", "--exact"], timeout);
    assert_failure(&output);
    assert!(stdout(&output).contains("FAILED (timeout after 120ms)"));

    let pid: libc::pid_t = std::fs::read_to_string(&pid_path)
        .expect("read grandchild pid")
        .trim()
        .parse()
        .expect("parse grandchild pid");
    assert_process_disappears(pid);
}

fn exit_classification(state: &ConformanceState) {
    let signaled = run_driver(state, ["process::signal", "--exact"], []);
    assert_failure(&signaled);
    assert!(stdout(&signaled).contains("FAILED (signal SIGUSR1)"));

    let exited = run_driver(state, ["process::exit", "--exact"], []);
    assert_failure(&exited);
    assert!(stdout(&exited).contains("FAILED (unexpected exit code 17)"));
}

fn bounded_workers(state: &ConformanceState) {
    assert_worker_limit(
        state,
        ["worker::", "--test-threads=2"],
        [("RUST_TEST_THREADS", "1")],
        2,
    );
    assert_worker_limit(state, ["worker::"], [("RUST_TEST_THREADS", "1")], 1);
    assert_worker_limit(
        state,
        ["worker::"],
        [("RUST_TEST_THREADS", "999")],
        prefork::HARD_MAX_WORKERS,
    );
    assert_worker_limit(state, ["worker::"], [], prefork::DEFAULT_WORKERS);
}

fn rejects_multithreaded_fork(state: &ConformanceState) {
    let output = run_driver(state, ["alpha::one", "--exact"], [(START_THREAD_ENV, "1")]);
    assert_failure(&output);
    assert!(
        stderr(&output).contains(
            "refusing to fork: /proc/self/task contains 2 tasks; exactly one is required"
        )
    );
}

fn rejects_bad_arguments(state: &ConformanceState) {
    let unsupported = run_driver(state, ["--ignored"], []);
    assert_failure(&unsupported);
    assert!(stderr(&unsupported).contains("unsupported argument: --ignored"));

    let conflicting = run_driver(state, ["--test-threads=2", "--test-threads", "3"], []);
    assert_failure(&conflicting);
    assert!(stderr(&conflicting).contains("--test-threads specified more than once"));

    let two_filters = run_driver(state, ["alpha", "beta"], []);
    assert_failure(&two_filters);
    assert!(stderr(&two_filters).contains("only one positional filter is supported"));

    let empty_split_skip = run_driver(state, ["--skip", ""], []);
    assert_failure(&empty_split_skip);
    assert!(stderr(&empty_split_skip).contains("--skip requires a non-empty value"));

    let setup_failure = run_driver(
        state,
        ["alpha::one", "--exact"],
        [(DRIVER_SETUP_FAILURE_ENV, "1")],
    );
    assert_failure(&setup_failure);
    assert!(
        stderr(&setup_failure).contains("setup failed: deliberate fixture setup failure"),
        "setup failure should be explicit: {}",
        stderr(&setup_failure)
    );
}

fn validates_worker_counts(state: &ConformanceState) {
    for value in ["invalid", "0", "18446744073709551616"] {
        let compact = run_driver(
            state,
            ["alpha::one", "--exact", &format!("--test-threads={value}")],
            [],
        );
        assert_failure(&compact);
        assert!(stderr(&compact).contains("--test-threads must be a positive integer"));

        let environment = run_driver(
            state,
            ["alpha::one", "--exact"],
            [("RUST_TEST_THREADS", value)],
        );
        assert_failure(&environment);
        assert!(stderr(&environment).contains("RUST_TEST_THREADS must be a positive integer"));
    }
}

fn cleans_up_after_resource_failure(state: &ConformanceState) {
    let temp = TempDir::new().expect("create resource-failure temp dir");
    let stdout_path = temp.path().join("stdout.txt");
    let stderr_path = temp.path().join("stderr.txt");
    let cleanup_path = temp.path().join("cleanup.txt");

    let mut command = driver_command(state);
    command
        .args(["resource::", "--test-threads=2", "--nocapture"])
        .env(CLEANUP_REPORT_ENV, &cleanup_path)
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create resource-failure stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create resource-failure stderr"),
        ));
    let mut driver = command.spawn().expect("spawn resource-failure driver");
    let child_pids = wait_for_direct_children(driver.id() as libc::pid_t, 2);
    lower_nofile_limit(driver.id() as libc::pid_t);
    let status = driver.wait().expect("wait for resource-failure driver");
    let output = Output {
        status,
        stdout: std::fs::read(&stdout_path).expect("read resource-failure stdout"),
        stderr: std::fs::read(&stderr_path).expect("read resource-failure stderr"),
    };

    let cleanup = std::fs::read_to_string(&cleanup_path).expect("read cleanup report");
    let mut report_lines = cleanup.lines();
    let before = report_lines.next().expect("cleanup baseline");
    let after = report_lines.next().expect("cleanup result");
    assert_failure(&output);
    assert!(stderr(&output).contains("pipe2 failed"));
    assert_eq!(before.strip_prefix("before="), after.strip_prefix("after="));
    assert_processes_disappear(&child_pids);
}

fn read_only_bytecode_cache_child(state: &ConformanceState) {
    let cache_root = TempDir::new().expect("create isolated XDG cache root");
    let environment = [
        (BYTECODE_DRIVER_ENV, "1".as_ref()),
        ("XDG_CACHE_HOME", cache_root.path().as_os_str()),
        ("WOW_SIM_ENABLE_BYTECODE_CACHE", "1".as_ref()),
    ];
    let output =
        run_driver_with_os_env(state, ["bytecode::read_only_child", "--exact"], environment);
    assert_success(&output);
}

fn parent_bypass_bytecode_cache(state: &ConformanceState) {
    let cache_root = TempDir::new().expect("create isolated XDG cache root");
    let warm_environment = [
        (BYTECODE_DRIVER_ENV, "1".as_ref()),
        ("XDG_CACHE_HOME", cache_root.path().as_os_str()),
        ("WOW_SIM_ENABLE_BYTECODE_CACHE", "1".as_ref()),
    ];
    let warm_output = run_driver_with_os_env(
        state,
        ["bytecode::read_only_child", "--exact"],
        warm_environment,
    );
    assert_success(&warm_output);

    let bypass_environment = [
        (BYTECODE_PARENT_BYPASS_DRIVER_ENV, "1".as_ref()),
        ("XDG_CACHE_HOME", cache_root.path().as_os_str()),
        ("WOW_SIM_ENABLE_BYTECODE_CACHE", "1".as_ref()),
    ];
    let bypass_output = run_driver_with_os_env(
        state,
        ["bytecode::read_only_child", "--exact"],
        bypass_environment,
    );
    assert_success(&bypass_output);
}

fn wait_for_direct_children(parent_pid: libc::pid_t, expected: usize) -> Vec<libc::pid_t> {
    let children_path = format!("/proc/{parent_pid}/task/{parent_pid}/children");
    let deadline = Instant::now() + RESOURCE_CHILD_WAIT;
    while Instant::now() < deadline {
        let contents = std::fs::read_to_string(&children_path).expect("read driver children");
        let children = contents
            .split_whitespace()
            .map(|value| value.parse().expect("parse driver child pid"))
            .collect::<Vec<_>>();
        if children.len() >= expected {
            return children;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("driver did not start {expected} children before resource failure");
}

fn lower_nofile_limit(pid: libc::pid_t) {
    let mut current = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let read_result =
        unsafe { libc::prlimit(pid, libc::RLIMIT_NOFILE, std::ptr::null(), &mut current) };
    assert_eq!(read_result, 0, "read driver RLIMIT_NOFILE");

    let limited = libc::rlimit {
        rlim_cur: 0,
        rlim_max: current.rlim_max,
    };
    let write_result =
        unsafe { libc::prlimit(pid, libc::RLIMIT_NOFILE, &limited, std::ptr::null_mut()) };
    assert_eq!(write_result, 0, "lower driver RLIMIT_NOFILE");
}

fn assert_processes_disappear(pids: &[libc::pid_t]) {
    let deadline = Instant::now() + RESOURCE_CHILD_WAIT;
    loop {
        let lingering = pids
            .iter()
            .copied()
            .filter(|pid| unsafe { libc::kill(*pid, 0) } == 0)
            .collect::<Vec<_>>();
        if lingering.is_empty() {
            return;
        }
        if Instant::now() >= deadline {
            for pid in &lingering {
                unsafe {
                    libc::kill(*pid, libc::SIGKILL);
                }
            }
            panic!("resource-failure children still exist after runner error: {lingering:?}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn assert_worker_limit<const N: usize, const E: usize>(
    state: &ConformanceState,
    args: [&str; N],
    environment: [(&str, &str); E],
    expected_max: usize,
) {
    let temp = TempDir::new().expect("create worker count temp dir");
    let worker_path = temp.path().join("workers.txt");
    std::fs::write(&worker_path, "0 0").expect("initialize worker count file");
    let mut command = driver_command(state);
    command.args(args).env(WORKER_STATE_ENV, &worker_path);
    command.envs(environment);
    let output = command.output().expect("run worker-count fixture driver");
    assert_success(&output);

    let (_, observed_max) = read_worker_counts(&worker_path);
    assert_eq!(observed_max, expected_max);
}

fn run_driver<const N: usize, const E: usize>(
    state: &ConformanceState,
    args: [&str; N],
    environment: [(&str, &str); E],
) -> Output {
    let mut command = driver_command(state);
    command.args(args).envs(environment);
    command.output().expect("run fixture driver")
}

fn run_driver_with_os_env<const N: usize, const E: usize>(
    state: &ConformanceState,
    args: [&str; N],
    environment: [(&str, &std::ffi::OsStr); E],
) -> Output {
    let mut command = driver_command(state);
    command.args(args).envs(environment);
    command.output().expect("run fixture driver")
}

fn driver_command(state: &ConformanceState) -> Command {
    let mut command = Command::new(&state.executable);
    command
        .env(DRIVER_MODE_ENV, "1")
        .env_remove(START_THREAD_ENV)
        .env_remove(WORKER_STATE_ENV)
        .env_remove(TREE_PID_ENV)
        .env_remove(DRIVER_TIMEOUT_MS_ENV)
        .env_remove(DRIVER_TERM_GRACE_MS_ENV)
        .env_remove(DRIVER_SETUP_MARKER_ENV)
        .env_remove(DRIVER_SETUP_FAILURE_ENV)
        .env_remove(BYTECODE_DRIVER_ENV)
        .env_remove("RUST_TEST_THREADS");
    command
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "driver failed\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn assert_failure(output: &Output) {
    assert!(!output.status.success(), "driver unexpectedly succeeded");
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_process_disappears(pid: libc::pid_t) {
    let deadline = Instant::now() + PROCESS_DEATH_WAIT;
    while Instant::now() < deadline {
        let result = unsafe { libc::kill(pid, 0) };
        if result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("grandchild process {pid} still exists after process-group cleanup");
}

fn fixture_read_only_bytecode_cache(state: &BytecodeFixtureState) {
    assert!(
        BYTECODE_CHILD_SETUP_RAN.load(Ordering::SeqCst),
        "child setup must run before the registered test body"
    );
    let result = load_addon(&state.env.loader_env(), &state.child_toc)
        .expect("compile unique Lua chunk in read-only child");
    assert_eq!(result.timing.cache_misses, 1);
    assert_eq!(result.timing.cache_store_successes, 0);
    assert_eq!(result.timing.cache_store_failures, 0);
    let loaded_value: String = state
        .env
        .eval("return __prefork_bytecode_child")
        .expect("read value assigned by unique child chunk");
    assert_eq!(loaded_value, state.expected_value);
}

fn fixture_pass(_: &FixtureState) {}

fn fixture_cow_mutate(state: &FixtureState) {
    state.cow_value.store(99, Ordering::SeqCst);
}

fn fixture_cow_observe(state: &FixtureState) {
    assert_eq!(state.cow_value.load(Ordering::SeqCst), 7);
}

fn fixture_output_pass(_: &FixtureState) {
    println!("child stdout marker");
    eprintln!("child stderr marker");
}

fn fixture_output_panic(_: &FixtureState) {
    println!("child stdout marker");
    eprintln!("child stderr marker");
    panic!("deliberate panic text");
}

fn fixture_signal(_: &FixtureState) {
    unsafe {
        libc::raise(libc::SIGUSR1);
    }
}

fn fixture_exit(_: &FixtureState) {
    unsafe {
        libc::_exit(17);
    }
}

fn fixture_timeout_tree(_: &FixtureState) {
    unsafe {
        libc::signal(libc::SIGTERM, libc::SIG_IGN);
    }
    let pid_path = env::var_os(TREE_PID_ENV).expect("tree pid path environment");
    let child = Command::new(env::current_exe().expect("resolve current executable"))
        .env(TREE_CHILD_MODE_ENV, "1")
        .spawn()
        .expect("spawn process-tree grandchild");
    std::fs::write(pid_path, child.id().to_string()).expect("write process-tree grandchild pid");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn run_tree_child() -> ! {
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn fixture_worker(_: &FixtureState) {
    let path = PathBuf::from(env::var_os(WORKER_STATE_ENV).expect("worker state path"));
    update_worker_counts(&path, 1);
    std::thread::sleep(WORKER_HOLD);
    update_worker_counts(&path, -1);
}

fn fixture_resource_hold(_: &FixtureState) {
    std::thread::sleep(RESOURCE_LONG_HOLD);
}

fn fixture_resource_release(_: &FixtureState) {
    std::thread::sleep(RESOURCE_SHORT_HOLD);
}

fn update_worker_counts(path: &Path, delta: isize) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open worker state file");
    lock_file(&file);
    let (active, observed_max) = read_counts_from_file(&mut file);
    let next_active = active
        .checked_add_signed(delta)
        .expect("valid active worker count");
    let next_max = observed_max.max(next_active);
    write_counts_to_file(&mut file, next_active, next_max);
    unlock_file(&file);
}

fn read_worker_counts(path: &Path) -> (usize, usize) {
    let mut file = File::open(path).expect("open final worker state file");
    read_counts_from_file(&mut file)
}

fn read_counts_from_file(file: &mut File) -> (usize, usize) {
    file.seek(SeekFrom::Start(0))
        .expect("seek worker state file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("read worker state file");
    let mut fields = contents.split_whitespace();
    let active = fields
        .next()
        .expect("active worker field")
        .parse()
        .expect("parse active workers");
    let observed_max = fields
        .next()
        .expect("maximum worker field")
        .parse()
        .expect("parse maximum workers");
    (active, observed_max)
}

fn write_counts_to_file(file: &mut File, active: usize, observed_max: usize) {
    file.set_len(0).expect("truncate worker state file");
    file.seek(SeekFrom::Start(0))
        .expect("rewind worker state file");
    write!(file, "{active} {observed_max}").expect("write worker state file");
    file.flush().expect("flush worker state file");
}

fn lock_file(file: &File) {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_EX) };
    assert_eq!(result, 0, "lock worker state file");
}

fn unlock_file(file: &File) {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_UN) };
    assert_eq!(result, 0, "unlock worker state file");
}
