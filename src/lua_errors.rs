//! `lua-errors` subcommand: load UI, collect Lua errors, output unique errors as JSON.

use crate::lua_api::{SimState, WowLuaEnv};
use crate::startup::{collect_lua_error_startup, run_lua_error_update_ticks};
use std::collections::BTreeMap;

const ERRORS_BY_ADDON_ENV: &str = "WOW_SIM_LUA_ERRORS_BY_ADDON";

/// A unique Lua error with its occurrence count.
#[derive(serde::Serialize)]
struct LuaError {
    message: String,
    count: usize,
}

struct UniqueLuaError {
    key: String,
    message: String,
}

/// Run headless startup, collect Lua errors, and print unique errors as JSON to stdout.
///
/// `saved_stdout` is the original stdout fd (redirected to stderr during loading).
/// We restore it before printing JSON so only JSON goes to stdout.
/// `exec_lua` is optional Lua code to execute after startup events.
pub fn run_lua_errors(
    env: &WowLuaEnv,
    saved_stdout: Option<i32>,
    exec_lua: Option<&str>,
    exec_lua_secure: bool,
) -> bool {
    // Suppress stderr during startup events (errors are collected in SimState)
    let saved_stderr = suppress_stderr();

    collect_lua_error_startup(env);

    restore_stderr(saved_stderr);

    // Restore stdout BEFORE exec-lua so user `print(...)` lands on the
    // real stdout, not the stderr-redirect that startup logging used.
    // The trailing JSON also writes to real stdout — that's intentional;
    // a post-startup probe (`--exec-lua 'print(state)'`) and the error
    // baseline both go to stdout, with the JSON last.
    restore_stdout(saved_stdout);

    if let Some(code) = exec_lua {
        // Re-route `print` to `io.stdout:write` for the duration of the
        // probe. By the time we get here, Blizzard_PrintHandler.lua has
        // overwritten the global `print` to send into chat-frame plumbing
        // (and is set up before our shared_bootstrap could reroute) — so
        // user `print(...)` from --exec-lua otherwise vanishes. See
        // PLAN.classic.md Phase 6.5 for the investigation.
        // Trailing flush: libc stdout is fully-buffered when stdout is a
        // file (not a TTY); without an explicit flush the print output
        // would land AFTER the JSON instead of before it.
        let probe_code = format!(
            "print = function(...) \
                local n = select('#', ...) \
                for i = 1, n do \
                    if i > 1 then io.stdout:write('\\t') end \
                    io.stdout:write(tostring((select(i, ...)))) \
                end \
                io.stdout:write('\\n') \
            end\n{code}\n\
            if io and io.stdout and io.stdout.flush then io.stdout:flush() end"
        );
        if let Err(e) = env.exec_maybe_secure(&probe_code, exec_lua_secure) {
            eprintln!("[exec-lua] error: {e}");
        }
        run_lua_error_update_ticks(env);
    }

    print_rehash_stats();
    print_intern_stats();
    print_errors_by_addon_if_requested(env);

    let errors = collect_unique_errors(env);
    let json = serde_json::to_string_pretty(&errors).expect("JSON serialization failed");
    println!("{json}");
    errors.is_empty()
}

fn print_errors_by_addon_if_requested(env: &WowLuaEnv) {
    if std::env::var_os(ERRORS_BY_ADDON_ENV).is_none() {
        return;
    }

    let state = env.state().borrow();
    for line in errors_by_addon_lines(&state) {
        eprintln!("{line}");
    }
}

fn errors_by_addon_lines(state: &SimState) -> Vec<String> {
    let grouped = grouped_errors_by_addon(state);
    if grouped.is_empty() {
        return Vec::new();
    }

    let mut lines = vec![String::from("[lua-errors] errors by addon:")];
    for (addon_name, messages) in grouped {
        lines.push(format!("  {addon_name}: {} error(s)", messages.len()));
        for message in messages {
            lines.push(format!("    {}", indent_continuation_lines(&message)));
        }
    }
    lines
}

fn indent_continuation_lines(message: &str) -> String {
    message.replace('\n', "\n    ")
}

#[cfg(feature = "rehash-stats")]
fn print_rehash_stats() {
    const TOP_N: usize = 20;
    let s = rilua::vm::rehash_stats::snapshot();
    eprintln!(
        "[rehash-stats] total={} from_empty={} grow={} frame_backed={} nonframe={}",
        s.total, s.from_empty, s.grow, s.frame_backed, s.nonframe
    );
    print_size_histogram("by new hash size (2^i)", &s.by_new_size, "size");
    print_size_histogram(
        "resizes to hash=0, grouped by old hash size",
        &s.to_zero_from,
        "from",
    );
    print_rehash_callsite_top(TOP_N);
    print_rehash_lua_site_top(TOP_N);
}

#[cfg(feature = "rehash-stats")]
fn print_size_histogram(header: &str, buckets: &[u64; 16], entry_prefix: &str) {
    eprintln!("[rehash-stats] {header}:");
    for (i, count) in buckets.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        let size = if i == 0 { 0 } else { 1u32 << i };
        eprintln!("  {entry_prefix} {size:>6}: {count}");
    }
}

#[cfg(feature = "rehash-stats")]
fn print_rehash_callsite_top(limit: usize) {
    eprintln!("[rehash-stats] top_{limit}_rust_callsites:");
    for ((file, line, column), count) in rilua::vm::rehash_stats::snapshot_callsite_top(limit) {
        eprintln!("  {count:>10} x {file}:{line}:{column}");
    }
}

#[cfg(feature = "rehash-stats")]
fn print_rehash_lua_site_top(limit: usize) {
    eprintln!("[rehash-stats] top_{limit}_lua_settable_sites:");
    for ((source, line), count) in rilua::vm::rehash_stats::snapshot_lua_site_top(limit) {
        eprintln!("  {count:>10} x {source}:{line}");
    }
}

#[cfg(not(feature = "rehash-stats"))]
fn print_rehash_stats() {}

#[cfg(feature = "intern-stats")]
fn print_intern_stats() {
    const TOP_N: usize = 40;
    let top = rilua::vm::intern_stats::snapshot_top(TOP_N);
    let total = rilua::vm::intern_stats::total_calls();
    let unique = rilua::vm::intern_stats::unique_strings();
    eprintln!("[intern-stats] total_calls={total} unique_strings={unique} top_{TOP_N}:",);
    for (data, count) in top {
        let preview = String::from_utf8_lossy(&data);
        let shown: String = preview.chars().take(48).collect();
        let suffix = if preview.len() > 48 { "…" } else { "" };
        eprintln!("  {count:>10} x {shown:?}{suffix}");
    }
}

#[cfg(not(feature = "intern-stats"))]
fn print_intern_stats() {}

/// Deduplicate collected Lua errors by their first line. Preserves first-seen order.
fn collect_unique_errors(env: &WowLuaEnv) -> Vec<LuaError> {
    let state = env.state().borrow();
    unique_error_messages(&state)
        .into_iter()
        .map(|error| LuaError {
            count: state.lua_error_counts.get(&error.key).copied().unwrap_or(0),
            message: error.message,
        })
        .collect()
}

pub fn grouped_errors_by_addon(state: &SimState) -> BTreeMap<String, Vec<String>> {
    let mut grouped = BTreeMap::new();

    for record in &state.lua_error_records {
        let addon_name = record
            .addon_name
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        let message = format_error_for_display(&record.message);
        grouped
            .entry(addon_name)
            .or_insert_with(Vec::new)
            .push(message);
    }

    grouped
}

pub(crate) fn suppressed_error_summary_lines(state: &SimState) -> Vec<String> {
    unique_error_messages(state)
        .into_iter()
        .filter_map(|error| {
            let count = state.lua_error_counts.get(&error.key).copied().unwrap_or(0);
            (count > 1).then(|| {
                format!(
                    "Lua error suppressed {} additional times: {}",
                    count - 1,
                    error.message
                )
            })
        })
        .collect()
}

pub(crate) fn print_suppressed_error_summary(state: &SimState) {
    for line in suppressed_error_summary_lines(state) {
        eprintln!("{line}");
    }
}

fn unique_error_messages(state: &SimState) -> Vec<UniqueLuaError> {
    let mut order = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in &state.lua_errors {
        let key = extract_error_message(raw);
        if state.lua_error_counts.contains_key(&key) && seen.insert(key.clone()) {
            order.push(UniqueLuaError {
                message: format_error_for_display(raw),
                key,
            });
        }
    }
    order
}

/// Extract the core error message, stripping "runtime error: " prefix and the
/// `<source>:<line>:` location chunk so similar errors with different paths
/// dedup against the same key.
pub(crate) fn extract_error_message(raw: &str) -> String {
    let first_line = raw.lines().next().unwrap_or(raw);
    normalize_error_headline(first_line, /*strip_location=*/ true)
}

/// Format an error for human/JSON display. Preserves the `<source>:<line>:`
/// location so the reader can find the exact call site — distinct from
/// `extract_error_message` which strips it for stable dedup keys.
fn format_error_for_display(raw: &str) -> String {
    let mut lines = raw.lines();
    let first_line = lines.next().unwrap_or(raw);
    let mut formatted = normalize_error_headline(first_line, /*strip_location=*/ false);
    for line in lines {
        formatted.push('\n');
        formatted.push_str(line);
    }
    formatted
}

fn normalize_error_headline(first_line: &str, strip_location: bool) -> String {
    let stripped = first_line
        .strip_prefix("runtime error: ")
        .unwrap_or(first_line);
    if strip_location {
        strip_lua_location_prefix(stripped)
    } else {
        stripped.to_string()
    }
}

fn strip_lua_location_prefix(msg: &str) -> String {
    let Some((prefix, body)) = msg.rsplit_once(": ") else {
        return msg.to_string();
    };
    let Some((before_line, line)) = prefix.rsplit_once(':') else {
        return msg.to_string();
    };
    if line.parse::<usize>().is_err() {
        return msg.to_string();
    }

    match before_line.rsplit_once(": ") {
        Some((context, _source)) => format!("{context}: {body}"),
        None => body.to_string(),
    }
}

/// Redirect stdout (fd 1) to stderr (fd 2). Returns saved original stdout fd.
pub fn redirect_stdout_to_stderr() -> Option<i32> {
    unsafe {
        let saved = libc::dup(1);
        if saved < 0 {
            return None;
        }
        libc::dup2(2, 1); // stdout now points to stderr
        Some(saved)
    }
}

/// Restore stdout from a saved fd.
pub fn restore_stdout(saved: Option<i32>) {
    if let Some(fd) = saved {
        unsafe {
            libc::dup2(fd, 1);
            libc::close(fd);
        }
    }
}

/// Redirect stderr to /dev/null. Returns saved original stderr fd.
fn suppress_stderr() -> Option<i32> {
    unsafe {
        let saved = libc::dup(2);
        if saved < 0 {
            return None;
        }
        let devnull = libc::open(b"/dev/null\0".as_ptr().cast(), libc::O_WRONLY);
        if devnull < 0 {
            libc::close(saved);
            return None;
        }
        libc::dup2(devnull, 2);
        libc::close(devnull);
        Some(saved)
    }
}

/// Restore stderr from a saved fd.
fn restore_stderr(saved: Option<i32>) {
    if let Some(fd) = saved {
        unsafe {
            libc::dup2(fd, 2);
            libc::close(fd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_unique_errors, errors_by_addon_lines, grouped_errors_by_addon};
    use crate::lua_api::AddonInfo;
    use crate::lua_api::WowLuaEnv;
    use crate::startup::collect_lua_error_startup;
    use rilua::LuaApi;

    #[test]
    fn collect_unique_errors_preserves_traceback_in_output_message() {
        let env = WowLuaEnv::new().expect("lua env");
        crate::lua_api::script_helpers::collect_lua_error(
            env.rilua().state(),
            "runtime error: repeated boom\nstack traceback:\n\t[C]: in function 'error'",
        );

        let errors = collect_unique_errors(&env);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].count, 1);
        assert_eq!(
            errors[0].message,
            "repeated boom\nstack traceback:\n\t[C]: in function 'error'"
        );
    }

    #[test]
    fn collect_lua_error_startup_captures_deferred_on_update_errors() {
        let env = WowLuaEnv::new().expect("lua env");
        env.exec(
            r#"
            local frame = CreateFrame("Frame", "LuaErrorsDeferredOnUpdateProbe", UIParent)
            frame:SetScript("OnUpdate", function(self)
                self:SetScript("OnUpdate", function()
                    error("deferred onupdate boom")
                end)
            end)
            "#,
        )
        .expect("install deferred OnUpdate probe");

        collect_lua_error_startup(&env);

        let errors = env.state().borrow().lua_errors.clone();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("deferred onupdate boom")),
            "lua-errors startup collection should run enough OnUpdate ticks to catch deferred handler errors: {errors:?}"
        );
    }

    #[test]
    fn grouped_errors_by_addon_preserves_traceback_lines() {
        let env = WowLuaEnv::new().expect("lua env");
        env.register_addon(AddonInfo {
            folder_name: "TestAddon".to_string(),
            title: "TestAddon".to_string(),
            enabled: true,
            loaded: true,
            ..Default::default()
        });
        let loading_index = {
            let state = env.state().borrow();
            state
                .addons
                .iter()
                .position(|addon| addon.folder_name == "TestAddon")
                .expect("TestAddon should be registered") as u16
        };
        env.state().borrow_mut().loading_addon_index = Some(loading_index);

        crate::lua_api::script_helpers::collect_lua_error(
            env.rilua().state(),
            "runtime error: [OnLoad] SomeFrame: Interface/AddOns/TestAddon/Main.lua:9: boom\nstack traceback:\n\t[C]: in function 'error'",
        );

        let state = env.state().borrow();
        let grouped = grouped_errors_by_addon(&state);
        // Preserves the source:line prefix so the reader can find the call site —
        // see Phase 6.2 of PLAN.classic.md for the rationale.
        assert_eq!(
            grouped.get("TestAddon"),
            Some(&vec![String::from(
                "[OnLoad] SomeFrame: Interface/AddOns/TestAddon/Main.lua:9: boom\nstack traceback:\n\t[C]: in function 'error'"
            )])
        );
    }

    #[test]
    fn errors_by_addon_lines_include_counts_and_indented_tracebacks() {
        let env = WowLuaEnv::new().expect("lua env");
        env.register_addon(AddonInfo {
            folder_name: "TraceAddon".to_string(),
            title: "TraceAddon".to_string(),
            enabled: true,
            loaded: true,
            ..Default::default()
        });
        let loading_index = {
            let state = env.state().borrow();
            state
                .addons
                .iter()
                .position(|addon| addon.folder_name == "TraceAddon")
                .expect("TraceAddon should be registered") as u16
        };
        env.state().borrow_mut().loading_addon_index = Some(loading_index);

        crate::lua_api::script_helpers::collect_lua_error(
            env.rilua().state(),
            "runtime error: Interface/AddOns/TraceAddon/Main.lua:4: boom\nstack traceback:\n\t[C]: in function 'error'",
        );

        let state = env.state().borrow();
        let lines = errors_by_addon_lines(&state);
        assert_eq!(lines[0], "[lua-errors] errors by addon:");
        assert_eq!(lines[1], "  TraceAddon: 1 error(s)");
        assert_eq!(
            lines[2],
            "    Interface/AddOns/TraceAddon/Main.lua:4: boom\n    stack traceback:\n    \t[C]: in function 'error'"
        );
    }
}
