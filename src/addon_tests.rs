//! `run-tests` subcommand: run Lua test files from an addon's tests/ directory.
//!
//! Loads the TestFramework addon (assertions + test/async_test registration),
//! then executes each .lua file and reports per-test pass/fail.

use crate::lua_api::WowLuaEnv;
use crate::startup::{fire_one_on_update_tick, process_pending_timers};
use std::path::PathBuf;

const MAX_ASYNC_TICKS: u32 = 500;

/// Reset test registry between files (TestFramework globals stay alive).
const TEST_RESET: &str = r#"
__addon_tests = {}
__addon_test_results = {}
__async_done = false
__async_error = nil
"#;

/// Run sync tests, store results. Skips async tests (marked for Rust to handle).
const SYNC_RUNNER: &str = r#"
__addon_test_results = {}
for _, t in ipairs(__addon_tests) do
    if not t.async then
        local ok, err = pcall(t.fn)
        __addon_test_results[#__addon_test_results + 1] = {
            name = t.name,
            ok = ok,
            err = ok and "" or tostring(err),
        }
    end
end
"#;

/// Start a single async test by index. Sets up __async_done tracking.
/// `idx` is prepended by `run_async_test` as `local idx = <n>` — this chunk has
/// no varargs, so it must use that local rather than `...`.
const ASYNC_START: &str = r#"
local t = __addon_tests[idx]
__async_done = false
__async_error = nil
local function done(assertion_fn)
    if assertion_fn then
        local ok, err = pcall(assertion_fn)
        if not ok then
            __async_error = tostring(err)
        end
    end
    __async_done = true
end
local ok, err = pcall(t.fn, done)
if not ok then
    __async_error = tostring(err)
    __async_done = true
end
"#;

/// Load the TestFramework addon (provides test/async_test/assertions).
fn load_test_framework(env: &WowLuaEnv) {
    let Some(toc_path) = crate::paths::find_bundled_test_framework_toc() else {
        eprintln!(
            "Bundled TestFramework addon not found beside the executable or in the build source"
        );
        std::process::exit(1);
    };
    match crate::loader::load_addon(&env.loader_env(), &toc_path) {
        Ok(result) => {
            for warning in &result.warnings {
                eprintln!("  [failure] {warning}");
            }
            for observation in &result.nil_symbol_observations {
                eprintln!("  [nil-observation] {observation}");
            }
            for requirement in &result.missing_requirements {
                eprintln!("  [missing-requirement] {requirement}");
            }
        }
        Err(e) => {
            eprintln!("Failed to load TestFramework: {e}");
            std::process::exit(1);
        }
    }
}

fn find_addon_root(env: &WowLuaEnv, addon_name: &str) -> Option<PathBuf> {
    env.state()
        .borrow()
        .addon_base_paths
        .iter()
        .map(|base| base.join(addon_name))
        .find(|path| path.is_dir())
}

/// Run all .lua test files from `<base>/<addon_name>/tests/`.
pub fn run_addon_tests(
    env: &WowLuaEnv,
    addon_name: &str,
    exec_lua: Option<&str>,
    exec_lua_secure: bool,
) {
    run_exec_lua(env, exec_lua, exec_lua_secure);
    load_test_framework(env);
    let (tests_dir, test_files) = load_test_files(env, addon_name);
    eprintln!(
        "Running {} test file(s) from {}\n",
        test_files.len(),
        tests_dir.display()
    );

    let mut total_passed = 0u32;
    let mut total_failed = 0u32;
    for path in &test_files {
        let (passed, failed) = report_test_file_result(env, path);
        total_passed += passed;
        total_failed += failed;
        flush_console(env);
    }
    print_summary_and_exit(total_passed, total_failed);
}

fn run_exec_lua(env: &WowLuaEnv, exec_lua: Option<&str>, exec_lua_secure: bool) {
    if let Some(code) = exec_lua
        && let Err(e) = env.exec_maybe_secure(code, exec_lua_secure)
    {
        eprintln!("[exec-lua] error: {e}");
    }
}

fn load_test_files(env: &WowLuaEnv, addon_name: &str) -> (PathBuf, Vec<PathBuf>) {
    let Some(addon_root) = find_addon_root(env, addon_name) else {
        eprintln!("Addon '{addon_name}' not found in the configured addon roots");
        std::process::exit(1);
    };
    let tests_dir = addon_root.join("tests");
    if !tests_dir.exists() {
        eprintln!("No tests directory found: {}", tests_dir.display());
        std::process::exit(1);
    }

    let mut test_files = match collect_test_files(&tests_dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    test_files.sort();
    (tests_dir, test_files)
}

fn report_test_file_result(env: &WowLuaEnv, path: &PathBuf) -> (u32, u32) {
    let file_name = path.file_name().unwrap().to_string_lossy();
    match run_test_file(env, path) {
        Ok((passed, failed)) => {
            if failed == 0 {
                eprintln!("  \x1b[32m\u{2713}\x1b[0m {file_name} ({passed} tests)");
            }
            (passed, failed)
        }
        Err(e) => {
            eprintln!("  \x1b[31m\u{2717}\x1b[0m {file_name} (load error)");
            eprintln!("    {e}");
            (0, 1)
        }
    }
}

fn print_summary_and_exit(total_passed: u32, total_failed: u32) {
    let total = total_passed + total_failed;
    eprintln!(
        "\n{total} tests, \x1b[32m{total_passed} passed\x1b[0m, \x1b[31m{total_failed} failed\x1b[0m"
    );
    if total_failed > 0 {
        std::process::exit(1);
    }
}

/// Collect .lua files from a tests directory.
fn collect_test_files(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read {}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "lua"))
        .collect();

    if files.is_empty() {
        return Err(format!("No .lua test files found in {}", dir.display()));
    }
    Ok(files)
}

/// Run a single test file: reset registry, load file, run sync then async tests.
fn run_test_file(env: &WowLuaEnv, path: &PathBuf) -> Result<(u32, u32), String> {
    let file_name = path.file_name().unwrap().to_string_lossy();
    let code = std::fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;

    // Reset test registry between files
    env.exec(TEST_RESET)
        .map_err(|e| format!("reset error: {e}"))?;

    // Load the test file (registers tests via test() calls)
    let chunk_name = format!("@{}", path.display());
    env.exec_named(&code, &chunk_name)
        .map_err(|e| format!("{e}"))?;

    // Run sync tests
    env.exec(SYNC_RUNNER)
        .map_err(|e| format!("runner error: {e}"))?;

    // Read sync results
    let (mut passed, mut failed) = read_test_results(env, &file_name)?;

    // Run async tests one by one with tick loop
    let async_indices = get_async_test_indices(env)?;
    for idx in async_indices {
        let name = get_test_name(env, idx)?;
        match run_async_test(env, idx) {
            Ok(()) => passed += 1,
            Err(e) => {
                eprintln!("  \x1b[31m\u{2717}\x1b[0m {file_name} > {name}");
                eprintln!("    {e}");
                failed += 1;
            }
        }
    }

    Ok((passed, failed))
}

/// Get indices of async tests from __addon_tests.
fn get_async_test_indices(env: &WowLuaEnv) -> Result<Vec<i64>, String> {
    let csv: String = env
        .eval(
            r#"
            local indices = {}
            for idx, entry in ipairs(__addon_tests or {}) do
                if entry.async then
                    indices[#indices + 1] = tostring(idx)
                end
            end
            return table.concat(indices, ",")
            "#,
        )
        .map_err(|e| format!("failed to read __addon_tests: {e}"))?;
    if csv.is_empty() {
        return Ok(Vec::new());
    }
    csv.split(',')
        .map(|s| s.parse::<i64>().map_err(|e| e.to_string()))
        .collect()
}

/// Get test name by index from __addon_tests.
fn get_test_name(env: &WowLuaEnv, idx: i64) -> Result<String, String> {
    env.eval::<String>(&format!("return (__addon_tests[{idx}] or {{}}).name or ''"))
        .map_err(|e| format!("{e}"))
}

/// Run a single async test: call fn(done), tick until __async_done or timeout.
fn run_async_test(env: &WowLuaEnv, idx: i64) -> Result<(), String> {
    let code = format!("local idx = {idx}\n{ASYNC_START}");
    env.exec(&code).map_err(|e| format!("{e}"))?;

    for _ in 0..MAX_ASYNC_TICKS {
        let done: bool = env.eval("__async_done").unwrap_or(false);
        if done {
            break;
        }
        fire_one_on_update_tick(env);
        process_pending_timers(env);
        flush_console(env);
    }

    let done: bool = env.eval("__async_done").unwrap_or(false);
    if !done {
        return Err(format!("timed out after {MAX_ASYNC_TICKS} ticks"));
    }

    let err: String = env.eval("__async_error or ''").unwrap_or_default();
    if err.is_empty() { Ok(()) } else { Err(err) }
}

/// Read `__addon_test_results` from Lua and print failures.
fn read_test_results(env: &WowLuaEnv, file_name: &str) -> Result<(u32, u32), String> {
    let count: i32 = env
        .eval("return #__addon_test_results")
        .map_err(|e| format!("failed to count results: {e}"))?;
    let mut passed = 0;
    let mut failed = 0;
    for idx in 1..=count {
        let (ok, name, error): (bool, String, String) = env
            .eval(&format!(
                "local entry = __addon_test_results[{idx}]\n\
                 return entry.ok, tostring(entry.name or ''), tostring(entry.err or '')"
            ))
            .map_err(|e| format!("failed to read result {idx}: {e}"))?;
        if ok {
            passed += 1;
        } else {
            eprintln!("  \x1b[31m\u{2717}\x1b[0m {file_name} > {name}");
            eprintln!("    {error}");
            failed += 1;
        }
    }
    Ok((passed, failed))
}

#[cfg(all(test, feature = "retail-12-0-5"))]
mod tests {
    use super::*;
    use crate::lua_api::AddonInfo;

    #[test]
    fn synchronous_tests_reset_addon_instruction_usage() {
        let env = WowLuaEnv::new().unwrap();
        {
            let mut sim = env.state().borrow_mut();
            let index = sim.addons.len() as u16;
            sim.addons.push(AddonInfo {
                folder_name: "BudgetAddon".into(),
                ..Default::default()
            });
            sim.loading_addon_index = Some(index);
        }
        env.exec(
            r#"
            BudgetFrame = CreateFrame('Frame')
            BudgetFrame:RegisterEvent('PLAYER_LOGIN')
            Completed = 0
            BudgetFrame:SetScript('OnEvent', function()
                local n = 0
                for i = 1, 300 do n = n + i end
                Completed = Completed + 1
            end)
        "#,
        )
        .unwrap();
        env.state().borrow_mut().loading_addon_index = None;
        env.loader_env()
            .with_state(|state| {
                state.set_instruction_budget("BudgetAddon", Some(1000));
                Ok::<_, rilua::LuaError>(())
            })
            .unwrap();
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            file.path(),
            r#"
            test('first', function()
                A_Admin.FireEvent('PLAYER_LOGIN')
                assert(Completed == 1, 'first handler did not complete')
            end)
            test('second', function()
                A_Admin.FireEvent('PLAYER_LOGIN')
                assert(Completed == 2, 'second handler did not complete')
            end)
        "#,
        )
        .unwrap();
        load_test_framework(&env);
        assert_eq!(
            run_test_file(&env, &file.path().to_path_buf()).unwrap(),
            (2, 0)
        );
    }
}

/// Flush Lua print() output from console_output to stderr.
fn flush_console(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    for line in state.console_output.drain(..) {
        eprintln!("{line}");
    }
}
