//! Severity colors and synthetic Edit Mode previews, not encounter warning storage or dispatch.
use rilua::vm::state::LuaState;
#[cfg(feature = "retail-12-1-0")]
use rilua::vm::{gc::arena::GcRef, table::Table};
use rilua::{LuaResult, Val, runtime_error};

#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::{call_function_state, create_string, create_table, table_set};
use crate::lua_bridge::stack_val;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::table_set_rust_fn_static;

/// INFERRED explicit availability and severity sound inputs. Zero means no sound.
#[derive(Debug, Default)]
pub struct WarningSettings {
    pub available: bool,
    /// INFERRED: unconfigured visibility and custom-sound settings are false.
    pub warnings_shown: bool,
    pub play_custom_sounds_when_hidden: bool,
    pub sound_kits: [u32; 3],
}

#[cfg(feature = "retail-12-0-0")]
pub(crate) fn register_settings(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_EncounterWarnings")?;
    crate::lua_bridge::table_set_rust_fn_static(state, ns, "IsFeatureAvailable", |s| {
        let available = crate::lua_api::methods::borrow_state(s)?
            .encounter_warning_settings
            .available;
        s.push(Val::Bool(available));
        Ok(1)
    })?;
    crate::lua_bridge::table_set_rust_fn_static(state, ns, "IsFeatureEnabled", |s| {
        let enabled = {
            let sim = crate::lua_api::methods::borrow_state(s)?;
            sim.encounter_warning_settings.available
                && sim
                    .cvars
                    .get("encounterWarningsEnabled")
                    .is_some_and(|v| v != "0")
        };
        s.push(Val::Bool(enabled));
        Ok(1)
    })?;
    #[cfg(feature = "retail-12-0-5")]
    register_visibility_settings(state, ns)?;
    crate::lua_bridge::table_set_rust_fn_static(state, ns, "GetSoundKitForSeverity", |s| {
        let severity = read_severity(s)?;
        let kit = crate::lua_api::methods::borrow_state(s)?
            .encounter_warning_settings
            .sound_kits[severity as usize];
        s.push(Val::Num(f64::from(kit)));
        Ok(1)
    })
}

#[cfg(feature = "retail-12-0-5")]
fn register_visibility_settings(
    state: &mut LuaState,
    ns: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    use crate::lua_api::methods::{borrow_state, borrow_state_mut};
    use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
    table_set_rust_fn_static(state, ns, "GetWarningsShown", |s| {
        let value = borrow_state(s)?.encounter_warning_settings.warnings_shown;
        s.push(Val::Bool(value));
        Ok(1)
    })?;
    table_set_rust_fn_static(state, ns, "SetWarningsShown", |s| {
        let value = bool::from_stack(s, 1)?;
        borrow_state_mut(s)?
            .encounter_warning_settings
            .warnings_shown = value;
        Ok(0)
    })?;
    table_set_rust_fn_static(state, ns, "GetPlayCustomSoundsWhenHidden", |s| {
        let value = borrow_state(s)?
            .encounter_warning_settings
            .play_custom_sounds_when_hidden;
        s.push(Val::Bool(value));
        Ok(1)
    })?;
    table_set_rust_fn_static(state, ns, "SetPlayCustomSoundsWhenHidden", |s| {
        let value = bool::from_stack(s, 1)?;
        borrow_state_mut(s)?
            .encounter_warning_settings
            .play_custom_sounds_when_hidden = value;
        Ok(0)
    })
}

fn read_severity(state: &LuaState) -> LuaResult<u8> {
    match stack_val(state, 1) {
        Val::Num(0.0) => Ok(0),
        Val::Num(1.0) => Ok(1),
        Val::Num(2.0) => Ok(2),
        _ => Err(runtime_error(
            "warning severity must be Low (0), Medium (1), or High (2)",
        )),
    }
}

#[cfg(feature = "retail-12-1-0")]
pub(crate) use previews::register_preview;

#[cfg(feature = "retail-12-1-0")]
mod previews {
    use super::*;
    const PREVIEW_DURATION_SECONDS: f64 = 5.0;
    const PREVIEW_ICON_FILE_ID: f64 = 136122.0;

    pub(crate) fn register_preview(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
        table_set_rust_fn_static(state, namespace, "GetColorForSeverity", color_for_severity)?;
        table_set_rust_fn_static(state, namespace, "GetEditModeWarningInfo", preview)
    }

    /// Simulator palette (white, amber, red); the native client's colors are not documented.
    fn severity_rgb(severity: u8) -> (f64, f64, f64) {
        match severity {
            0 => (1.0, 1.0, 1.0),
            1 => (1.0, 0.75, 0.1),
            _ => (1.0, 0.15, 0.05),
        }
    }

    /// Fresh ColorMixin per call so callers can mutate their copy.
    fn create_severity_color(state: &mut LuaState, severity: u8) -> LuaResult<Val> {
        let (red, green, blue) = severity_rgb(severity);
        let factory = crate::c_api::global_val(state, "CreateColor");
        call_function_state(
            state,
            factory,
            &[
                Val::Num(red),
                Val::Num(green),
                Val::Num(blue),
                Val::Num(1.0),
            ],
        )
    }

    fn color_for_severity(state: &mut LuaState) -> LuaResult<u32> {
        let severity = read_severity(state)?;
        let color = create_severity_color(state, severity)?;
        state.push(color);
        Ok(1)
    }

    fn preview(state: &mut LuaState) -> LuaResult<u32> {
        let severity = read_severity(state)?;
        let text = match severity {
            0 => "Simulated Low Warning",
            1 => "Simulated Medium Warning",
            _ => "Simulated High Warning",
        };
        let color = create_severity_color(state, severity)?;
        let info = create_table(state);
        publish_preview_identity(state, info, text);
        publish_preview_display(state, info, severity);
        table_set(state, info, "color", color);
        state.push(info);
        Ok(1)
    }

    fn publish_preview_identity(state: &mut LuaState, info: Val, text: &str) {
        for (field, value) in [
            ("text", text),
            ("casterGUID", "Sim-Warning-Caster"),
            ("casterName", "Simulator Caster"),
            ("targetGUID", "Sim-Warning-Target"),
            ("targetName", "Simulator Target"),
        ] {
            let value = create_string(state, value);
            table_set(state, info, field, value);
        }
    }

    fn publish_preview_display(state: &mut LuaState, info: Val, severity: u8) {
        for (field, value) in [
            ("iconFileID", Val::Num(PREVIEW_ICON_FILE_ID)),
            ("tooltipSpellID", Val::Num(0.0)),
            ("isDeadly", Val::Bool(severity == 2)),
            ("duration", Val::Num(PREVIEW_DURATION_SECONDS)),
            ("severity", Val::Num(f64::from(severity))),
            ("shouldPlaySound", Val::Bool(false)),
            ("shouldShowChatMessage", Val::Bool(false)),
            ("shouldShowWarning", Val::Bool(true)),
        ] {
            table_set(state, info, field, value);
        }
    }
}
