//! Trusted configured countdown handoff into renderer-only text, not child Lua fields.

use super::FORMATTER_ROOTS;
use crate::lua_api::frame::methods::secret_origin::require_readable;
use crate::lua_api::globals::lua_duration_object::formatting::{format_number, identify_formatter};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, registry_get, table_get, val_to_string,
};
use rilua::table_security::{unwrap_secret, wrap_host_secret_number};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn tick_countdown_formatters(state: &mut LuaState) -> LuaResult<()> {
    let attachments = read_attached_frame_ids(state)?;
    if attachments.is_empty() {
        return Ok(());
    }
    // Same monotonic origin as GetTime and quad emission, not tick-delta accumulation.
    let now = borrow_state(state)?.start_time.elapsed().as_secs_f64();
    for id in attachments {
        let text = match format_active_countdown(state, id, now) {
            Ok(text) => text,
            Err(error) => {
                let message = format!("Cooldown {id} countdown formatter: {error}");
                crate::lua_api::script_helpers::call_error_handler_state(state, &message);
                None
            }
        };
        write_render_text(state, id, text)?;
    }
    Ok(())
}

fn read_attached_frame_ids(state: &mut LuaState) -> LuaResult<Vec<u64>> {
    let roots = registry_get(state, FORMATTER_ROOTS);
    let Val::Table(reference) = roots else {
        return if roots == Val::Nil {
            Ok(Vec::new())
        } else {
            Err(runtime_error("Cooldown formatter roots must be a table"))
        };
    };
    let entries = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("Cooldown formatter roots are unavailable"))?
        .hash_entries();
    entries
        .into_iter()
        .map(|(key, _)| {
            let id = val_to_string(state, key)
                .and_then(|key| key.parse::<u64>().ok())
                .ok_or_else(|| runtime_error("Cooldown formatter root requires a frame ID"))?;
            Ok(id)
        })
        .collect()
}

fn read_attached_formatter(state: &mut LuaState, id: u64) -> LuaResult<Val> {
    let roots = registry_get(state, FORMATTER_ROOTS);
    let Val::Table(reference) = roots else {
        return Err(runtime_error("Cooldown formatter roots must be a table"));
    };
    if state.gc.tables.get(reference).is_none() {
        return Err(runtime_error("Cooldown formatter roots are unavailable"));
    }
    Ok(table_get(state, roots, &id.to_string()))
}

fn format_active_countdown(state: &mut LuaState, id: u64, now: f64) -> LuaResult<Option<String>> {
    // Earlier callbacks may replace or clear this attachment and collect its old handle.
    let formatter = read_attached_formatter(state, id)?;
    if formatter == Val::Nil {
        return Ok(None);
    }
    let (remaining, secret) = {
        let sim = borrow_state(state)?;
        let frame = sim
            .widgets
            .get(id)
            .ok_or_else(|| runtime_error("Cooldown formatter frame is unavailable"))?;
        let Some(remaining) = frame.cooldown_remaining_seconds(now) else {
            return Ok(None);
        };
        (remaining, frame.secret_timing)
    };
    format_countdown_text(state, formatter, remaining, secret).map(Some)
}

fn format_countdown_text(
    state: &mut LuaState,
    formatter: Val,
    remaining: f64,
    secret: bool,
) -> LuaResult<String> {
    require_readable(state, secret)?;
    let previous_top = state.top;
    // A callback may detach even this active formatter before collecting garbage.
    state.push(formatter);
    let text = (|| {
        let kind = identify_formatter(state, formatter)?;
        let input = if secret {
            wrap_host_secret_number(state, remaining)
        } else {
            Val::Num(remaining)
        };
        state.push(input);
        let output = format_number(state, kind, formatter, input)?;
        state.push(output);
        // Authenticate secret output and inspect only an actual string. Never
        // call public tostring/FormatNumber or pass decoded timing to Lua.
        let output = unwrap_secret(state, output)?;
        val_to_string(state, output)
            .ok_or_else(|| runtime_error("Cooldown NumericFormatter must return a string"))
    })();
    state.top = previous_top;
    text
}

fn write_render_text(state: &mut LuaState, id: u64, text: Option<String>) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.cooldown_formatted_countdown_text != text);
    if changed && let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.cooldown_formatted_countdown_text = text;
    }
    Ok(())
}
