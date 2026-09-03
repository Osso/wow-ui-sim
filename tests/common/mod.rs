//! Shared test helpers.

pub mod addon_coverage_baseline;
pub mod blizzard_addon_harness;
pub mod blizzard_addon_manifest;
mod event_helpers;
pub mod game_menu_fixture;
pub mod panel_fixtures;
#[cfg(target_os = "linux")]
pub(crate) mod prefork_process;
#[cfg(target_os = "linux")]
mod timeout_reexec;
pub(crate) mod workload_gate;

use std::ops::Deref;
use std::path::{Path, PathBuf};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

/// Run a test closure with a per-call timeout.
/// Default 120s — enough for full Blizzard UI load + test logic.
#[cfg(target_os = "linux")]
pub fn with_timeout<F: FnOnce() + Send + 'static>(secs: u64, f: F) {
    timeout_reexec::run(secs, workload_gate::Mode::Shared, f);
}

#[cfg(target_os = "linux")]
pub fn with_performance_timeout<F: FnOnce() + Send + 'static>(secs: u64, f: F) {
    timeout_reexec::run(secs, workload_gate::Mode::Exclusive, f);
}

#[cfg(target_os = "linux")]
pub(crate) const TIMEOUT_FIXTURE_CAPACITY_ENV: &str = "WOW_SIM_TIMEOUT_FIXTURE_CAPACITY";

#[cfg(target_os = "linux")]
pub(crate) fn with_timeout_fixture_capacity<T>(body: impl FnOnce() -> T) -> T {
    workload_gate::with_lock(workload_gate::Mode::Exclusive, body)
}

#[cfg(target_os = "linux")]
pub(crate) fn with_performance_timeout_at<F: FnOnce() + Send + 'static>(
    secs: u64,
    path: &Path,
    f: F,
) {
    timeout_reexec::run_at(secs, path, workload_gate::Mode::Exclusive, f);
}

#[cfg(not(target_os = "linux"))]
pub fn with_timeout<F: FnOnce() + Send + 'static>(secs: u64, f: F) {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        f();
        let _ = tx.send(());
    });
    match rx.recv_timeout(std::time::Duration::from_secs(secs)) {
        Ok(()) => handle.join().expect("test thread panicked"),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            panic!("test timed out after {secs}s")
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            handle.join().expect("test thread panicked")
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn with_performance_timeout<F: FnOnce() + Send + 'static>(secs: u64, f: F) {
    with_timeout(secs, f);
}

/// Run an ordinary expensive workload alongside other shared workloads.
pub fn with_shared_workload<T>(f: impl FnOnce() -> T) -> T {
    workload_gate::with_lock(workload_gate::Mode::Shared, f)
}

/// Run a performance workload without competing test processes.
pub fn with_exclusive_workload<T>(f: impl FnOnce() -> T) -> T {
    workload_gate::with_lock(workload_gate::Mode::Exclusive, f)
}

/// Existing full-UI call sites are ordinary shared workloads.
pub fn with_perf_lock<T>(f: impl FnOnce() -> T) -> T {
    with_shared_workload(f)
}

/// Keep a `WowLuaEnv` under the shared workload gate for the lifetime of a test.
pub struct LockedEnv {
    _permit: workload_gate::Permit,
    env: WowLuaEnv,
}

pub fn lock_env(build: impl FnOnce() -> WowLuaEnv) -> LockedEnv {
    let permit = workload_gate::acquire(workload_gate::Mode::Shared)
        .unwrap_or_else(|error| panic!("acquire shared workload gate: {error}"));
    let env = build();
    LockedEnv {
        _permit: permit,
        env,
    }
}

impl Deref for LockedEnv {
    type Target = WowLuaEnv;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

#[macro_export]
macro_rules! prefork_full_ui_case {
    (fn $name:ident($env:ident: &WowLuaEnv) $body:block) => {
        pub(crate) mod $name {
            use super::*;

            pub(crate) fn run($env: &WowLuaEnv) $body
        }
    };
}

/// Convenience macro: wraps a test body with a 120s timeout.
///
/// ```ignore
/// #[test]
/// fn my_test() {
///     test_timeout! {
///         let env = WowLuaEnv::new().unwrap();
///         // ... test body ...
///     }
/// }
/// ```
#[macro_export]
macro_rules! test_timeout {
    ($($body:tt)*) => {
        $crate::common::with_timeout(120, move || { $($body)* })
    };
}

/// Run a performance test under the existing 120-second child timeout.
#[macro_export]
macro_rules! perf_test_timeout {
    ($($body:tt)*) => {
        $crate::common::with_performance_timeout(120, move || { $($body)* })
    };
}

/// Try to create a wgpu device for GPU tests.
/// Returns None if no adapter is available (e.g., headless CI).
#[cfg(feature = "gui")]
pub fn try_create_gpu_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Test GPU Device"),
        ..Default::default()
    }))
    .ok()?;

    Some((device, queue))
}

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

pub fn load_required_blizzard_addon(env: &WowLuaEnv, ui: &Path, addon_name: &str) {
    let addon_dir = ui.join(addon_name);
    let toc_path = find_toc_file(&addon_dir).unwrap_or_else(|| {
        panic!(
            "required Blizzard addon `{addon_name}` has no compatible TOC under {}",
            addon_dir.display()
        )
    });
    load_addon(&env.loader_env(), &toc_path).unwrap_or_else(|error| {
        panic!(
            "failed to load required Blizzard addon `{addon_name}` from {}: {error}",
            toc_path.display()
        )
    });
}

/// Load only the current Click Binding public publisher, keeping its LoD frame unloaded.
pub fn load_click_binding_bootstrap(env: &WowLuaEnv, ui: &Path) {
    let path = ui.join("Blizzard_ClickBindingUI/Blizzard_ClickBindingUI_Bootstrap.lua");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    env.exec(&source)
        .unwrap_or_else(|error| panic!("failed to load {}: {error}", path.display()));
}

/// Helper to load Blizzard_SharedXML templates for tests that need them.
/// Returns the environment with templates loaded.
pub fn env_with_shared_xml() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let ui = blizzard_ui_dir();

    load_required_blizzard_addon(&env, &ui, "Blizzard_SharedXMLBase");
    load_required_blizzard_addon(&env, &ui, "Blizzard_SharedXML");

    env
}

pub fn fire_addon_loaded(env: &WowLuaEnv, addon_name: &str) {
    let _ = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(addon_name)]);
}

pub use event_helpers::fire_player_entering_world;

pub fn call_global_if_present(env: &WowLuaEnv, function_name: &str) {
    let is_present = env
        .eval::<bool>(&format!("return type(_G[{function_name:?}]) == 'function'"))
        .unwrap_or(false);
    if is_present {
        let _ = env.exec(&format!(r#"_G[{function_name:?}]()"#));
    }
}

pub fn install_error_collector(env: &WowLuaEnv, global_name: &str) {
    env.exec(&format!(
        r#"
        local target = {global_name:?}
        _G[target] = {{}}
        seterrorhandler(function(msg)
            local trace = ""
            if type(debugstack) == "function" then
                trace = "\n" .. tostring(debugstack())
            end
            table.insert(_G[target], tostring(msg) .. trace)
        end)
        "#
    ))
    .expect("Failed to install test error handler");
}

pub fn drain_string_table(env: &WowLuaEnv, global_name: &str) -> Vec<String> {
    const SEP: char = '\u{1f}';
    let joined = env
        .eval::<String>(&format!(
            r#"
            local target = {global_name:?}
            local values = _G[target]
            if type(values) ~= "table" then
                return ""
            end
            local parts = {{}}
            for i = 1, #values do
                parts[#parts + 1] = tostring(values[i])
            end
            _G[target] = {{}}
            return table.concat(parts, string.char(31))
            "#
        ))
        .unwrap_or_default();
    if joined.is_empty() {
        return Vec::new();
    }
    joined.split(SEP).map(|entry| entry.to_string()).collect()
}

type TimeoutBody = fn();
type PerfBody = fn();
type EnvBuilder = fn() -> WowLuaEnv;

// Each integration test compiles this module separately, so a helper can be
// part of the shared test API while remaining unused in one specific target.
const _: () = {
    let _ = with_timeout::<TimeoutBody> as fn(u64, TimeoutBody);
    let _ = with_performance_timeout::<TimeoutBody> as fn(u64, TimeoutBody);
    let _ = with_perf_lock::<()> as fn(PerfBody);
    let _ = std::mem::size_of::<LockedEnv>();
    let _ = lock_env as fn(EnvBuilder) -> LockedEnv;
    let _ = env_with_shared_xml as fn() -> WowLuaEnv;
    let _ = fire_addon_loaded as fn(&WowLuaEnv, &str);
    let _ = fire_player_entering_world as fn(&WowLuaEnv, bool, bool);
    let _ = call_global_if_present as fn(&WowLuaEnv, &str);
    let _ = install_error_collector as fn(&WowLuaEnv, &str);
    let _ = drain_string_table as fn(&WowLuaEnv, &str) -> Vec<String>;

    #[cfg(feature = "gui")]
    {
        let _ = try_create_gpu_device as fn() -> Option<(wgpu::Device, wgpu::Queue)>;
    }
};
