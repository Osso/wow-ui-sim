//! Host-owned elapsed-time watchdogs for tainted addon execution.

use std::time::{Duration, Instant};

use super::methods::borrow_state;
use rilua::vm::state::{DEFAULT_WATCHDOG_DISPATCH_INTERVAL, LuaState};
use rilua::{LuaResult, Val};

// INFERRED (R88bn): timer units are milliseconds and the phase mapping is simulator policy.
const STARTUP_WATCHDOG_TIMEOUT_MS: u64 = 19_000;
const RUNTIME_WATCHDOG_TIMEOUT_MS: u64 = 200;

pub(crate) fn watchdog_timeout(state: &LuaState) -> LuaResult<Duration> {
    let sim = borrow_state(state)?;
    let timeout_ms = if sim.startup_script_watchdog_phase {
        STARTUP_WATCHDOG_TIMEOUT_MS
    } else {
        RUNTIME_WATCHDOG_TIMEOUT_MS
    };
    Ok(sim
        .script_watchdog_timeout_override
        .unwrap_or(Duration::from_millis(timeout_ms)))
}

pub(crate) fn with_addon_watchdog<T>(
    state: &mut LuaState,
    owner: &str,
    timeout: Duration,
    operation: impl FnOnce(&mut LuaState) -> LuaResult<T>,
) -> LuaResult<T> {
    state.with_instruction_owner(owner, |state| {
        state.with_execution_watchdog(timeout, DEFAULT_WATCHDOG_DISPATCH_INTERVAL, operation)
    })
}

pub(crate) fn call_frame_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler: Val,
    args: &[Val],
    event: Option<&str>,
) -> Result<Vec<Val>, String> {
    let owner = frame_addon_owner(state, frame_id).map_err(|error| error.to_string())?;
    let Some(owner) = owner else {
        return call_handler(state, handler, args).map_err(|error| error.to_string());
    };
    let timeout = watchdog_timeout(state).map_err(|error| error.to_string())?;
    let started = Instant::now();
    let result = state.with_instruction_owner(&owner, |state| {
        state.with_execution_watchdog(timeout, DEFAULT_WATCHDOG_DISPATCH_INTERVAL, |state| {
            call_handler(state, handler, args)
        })
    });
    if result.is_err() && super::handler_timing::is_enabled() {
        eprintln!(
            "{}",
            format_handler_watchdog_error(&owner, frame_id, event, timeout, started.elapsed(),)
        );
    }
    result.map_err(|error| error.to_string())?
}

fn call_handler(state: &mut LuaState, handler: Val, args: &[Val]) -> LuaResult<Vec<Val>> {
    super::script_helpers::protected_lua_pcall_state(state, handler, args)
        .map_err(rilua::runtime_error)
}

fn format_handler_watchdog_error(
    owner: &str,
    frame_id: u64,
    event: Option<&str>,
    timeout: Duration,
    elapsed: Duration,
) -> String {
    let mut line = format!("[handler-budget-error] owner={owner:?} frame=#{frame_id}");
    if let Some(event) = event {
        line.push_str(&format!(" event={event:?}"));
    }
    line.push_str(&format_watchdog_fields(timeout, elapsed));
    line
}

pub(crate) fn format_file_watchdog_error(
    owner: &str,
    chunk_name: &str,
    timeout: Duration,
    elapsed: Duration,
) -> String {
    format!(
        "[file-budget-error] owner={owner:?} file={chunk_name:?}{}",
        format_watchdog_fields(timeout, elapsed),
    )
}

pub(crate) fn format_file_watchdog_success(
    owner: &str,
    chunk_name: &str,
    timeout: Duration,
    elapsed: Duration,
) -> String {
    format!(
        "[file-budget-success] owner={owner:?} file={chunk_name:?}{}",
        format_watchdog_fields(timeout, elapsed),
    )
}

fn format_watchdog_fields(timeout: Duration, elapsed: Duration) -> String {
    format!(
        " timeout_ms={} elapsed_ms={}",
        timeout.as_millis(),
        elapsed.as_millis(),
    )
}

fn frame_addon_owner(state: &LuaState, frame_id: u64) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    let addon = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.owner_addon)
        .and_then(|index| sim.addons.get(index as usize));
    Ok(addon
        .filter(|addon| {
            addon.folder_name != "__BuiltIn"
                && !crate::blizzard_ui_sync::is_builtin_addon_folder(&addon.folder_name)
        })
        .map(|addon| addon.folder_name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_watchdog_error_reports_owner_event_and_elapsed_time() {
        assert_eq!(
            format_handler_watchdog_error(
                "DynamicAnchors",
                42,
                Some("PLAYER_LOGIN"),
                Duration::from_millis(200),
                Duration::from_millis(219),
            ),
            "[handler-budget-error] owner=\"DynamicAnchors\" frame=#42 event=\"PLAYER_LOGIN\" timeout_ms=200 elapsed_ms=219"
        );
    }

    #[test]
    fn file_watchdog_success_escapes_owner_and_chunk() {
        assert_eq!(
            format_file_watchdog_success(
                "Example\"\n",
                "@Interface/AddOns/Example/Group\"Tools\n.lua",
                Duration::from_millis(19_000),
                Duration::from_millis(27),
            ),
            "[file-budget-success] owner=\"Example\\\"\\n\" file=\"@Interface/AddOns/Example/Group\\\"Tools\\n.lua\" timeout_ms=19000 elapsed_ms=27"
        );
    }
}
