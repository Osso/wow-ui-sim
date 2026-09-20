//! `SetCVar` / `GetCVar` globals + `C_CVar.{Set,Get,GetDefault}CVar`.
//! All routes through `SimState.cvars` (which is seeded from
//! `cvars.yaml` and persists overrides to disk) so the in-game CVar
//! surface and the simulator's
//! Rust-side knobs (e.g. `Brightness=50`, `Contrast=50`, `Gamma=1.0`
//! from `cvars.yaml`) all read the same store.
//!
//! Previously `SetCVar` was registered only on the `A_Admin` path and
//! the retail-facing global was `stub_nil`; `GetCVar` came from
//! `Blizzard_SharedXMLBase/CvarUtil.lua` which routed through the
//! Lua-only `__cvars` table — so `SimState.cvars` defaults like the
//! display sliders were invisible to addon code. Now they're not.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::string::LuaString;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const CVAR_BITS_PER_BYTE: usize = 6;
const CVAR_DATA_MASK: u8 = 0x3F;
const CVAR_DATA_TAG: u8 = 0x40;
const DEFAULT_CVAR_BITFIELD_VERSION: u8 = b'0';
const C_CVAR_FUNCTIONS: &[(&str, RustFn)] = &[
    #[cfg(feature = "retail-12-1-0")]
    ("AreCVarsLoaded", are_cvars_loaded),
    ("GetCVar", get_cvar),
    ("SetCVar", set_cvar),
    ("GetCVarBool", get_cvar_bool),
    ("GetCVarDefault", get_cvar_default),
    ("RegisterCVar", register_cvar),
    ("GetCVarBitfield", get_cvar_bitfield),
    ("SetCVarBitfield", set_cvar_bitfield),
    ("ResetTestCVars", reset_test_cvars),
];

#[cfg(feature = "retail-12-1-0")]
fn are_cvars_loaded(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

fn value_to_string(state: &mut LuaState, index: i32) -> String {
    match stack_val(state, index) {
        Val::Nil => String::new(),
        Val::Bool(true) => "1".to_string(),
        Val::Bool(false) => "0".to_string(),
        Val::Num(n) => {
            if n.fract() == 0.0 {
                format!("{}", n as i64)
            } else {
                format!("{n}")
            }
        }
        Val::Str(_) => Option::<String>::from_stack(state, index)
            .ok()
            .flatten()
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// `SetCVar(name, value)` — write `value` (stringified) to
/// `SimState.cvars[name]`. Returns true on success, matching retail.
fn set_cvar(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let value = value_to_string(state, 2);

    if is_ui_scale_cvar(&name) {
        if let Some(scale) = requested_ui_parent_scale(state, &name, &value)? {
            apply_ui_parent_scale(state, scale)?;
        }
        dispatch_event_now(state, "DISPLAY_SIZE_CHANGED", &[])?;
        dispatch_event_now(state, "UI_SCALE_CHANGED", &[])?;
    }

    #[cfg(feature = "client-wowforever")]
    let override_change = gamepad_override_change(state, &name, &value)?;
    let accepted = borrow_state_mut(state)?.cvars.set(&name, &value);
    if accepted {
        let name_arg = create_string(state, &name);
        let value_arg = create_string(state, &value);
        dispatch_event_now(state, "CVAR_UPDATE", &[name_arg, value_arg])?;
        #[cfg(feature = "client-wowforever")]
        if let Some((event, old, new)) = override_change {
            dispatch_event_now(state, event, &[Val::Num(old), Val::Num(new)])?;
        }
    }
    state.push(Val::Bool(accepted));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn gamepad_override_change(
    state: &LuaState,
    name: &str,
    new: &str,
) -> LuaResult<Option<(&'static str, f64, f64)>> {
    let event = match name.to_ascii_lowercase().as_str() {
        "gamepadpossessbaroverride" => "GAMEPAD_POSSESS_BAR_OVERRIDE_CHANGED",
        "gamepadstancebaroverride" => "GAMEPAD_STANCE_BAR_OVERRIDE_CHANGED",
        _ => return Ok(None),
    };
    let parse = |value: &str| {
        value
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .ok_or_else(|| {
                rilua::runtime_error(format!("{name}: expected numeric override, got {value:?}"))
            })
    };
    let old = borrow_state(state)?.cvars.get(name).ok_or_else(|| {
        rilua::runtime_error(format!("{name}: missing registered override value"))
    })?;
    let old = parse(&old)?;
    let new = parse(new)?;
    Ok((old != new).then_some((event, old, new)))
}

fn is_ui_scale_cvar(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "uiscale" | "useuiscale")
}

fn requested_ui_parent_scale(state: &LuaState, name: &str, value: &str) -> LuaResult<Option<f32>> {
    let normalized_name = name.to_ascii_lowercase();
    let scale = match normalized_name.as_str() {
        "uiscale" if borrow_state(state)?.cvars.get_bool("useUiScale") => value.parse().ok(),
        "useuiscale" if value == "1" => borrow_state(state)?
            .cvars
            .get("uiScale")
            .and_then(|value| value.parse().ok()),
        _ => None,
    };
    Ok(scale.filter(|scale| *scale > 0.0))
}

fn apply_ui_parent_scale(state: &LuaState, scale: f32) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let Some(id) = sim.widgets.get_id_by_name("UIParent") else {
        return Ok(());
    };
    let current_scale = sim.widgets.get(id).map(|frame| frame.scale).unwrap_or(1.0);
    if current_scale == scale {
        return Ok(());
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scale = scale;
    }
    sim.widgets.propagate_effective_scale(id, 1.0);
    sim.widgets.mark_rect_dirty(id);
    Ok(())
}

/// `RegisterCVar(name, value)` — create a readable runtime CVar default
/// without forcing an override. Retail startup paths use this to make a
/// previously unknown CVar visible to later `GetCVar*` reads.
fn register_cvar(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    let default = Option::<String>::from_stack(state, 2)?.filter(|value| !value.is_empty());
    borrow_state(state)?
        .cvars
        .register(&name, default.as_deref());
    Ok(0)
}

/// `GetCVar(name)` — read `SimState.cvars[name]` (override → default).
/// Returns nil for unknown names, matching retail's behaviour for
/// CVars that haven't been registered.
fn get_cvar(state: &mut LuaState) -> LuaResult<u32> {
    push_cvar_string(state, CvarStringSource::Current)
}

/// `GetCVarDefault(name)` — read the YAML/factory default for `name`,
/// ignoring any session override.
fn get_cvar_default(state: &mut LuaState) -> LuaResult<u32> {
    push_cvar_string(state, CvarStringSource::Default)
}

enum CvarStringSource {
    Current,
    Default,
}

fn push_cvar_string(state: &mut LuaState, source: CvarStringSource) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let value = match source {
        CvarStringSource::Current => borrow_state(state)?.cvars.get(&name),
        CvarStringSource::Default => borrow_state(state)?.cvars.get_default(&name),
    };
    push_optional_string(state, value);
    Ok(1)
}

/// `GetCVarBool(name)` — `true` iff the stored value is `"1"`.
fn get_cvar_bool(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let value = borrow_state(state)?.cvars.get_bool(&name);
    state.push(Val::Bool(value));
    Ok(1)
}

/// `GetCVarBitfield(name, index)` — decode the printable 6-bit packed CVar
/// format that Blizzard's `CVarCallbackRegistry` expects.
fn get_cvar_bitfield(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let Some(index) = cvar_bit_index(state, 2) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let value = borrow_state(state)?.cvars.get(&name);
    let enabled = value
        .as_deref()
        .map(|packed| packed_cvar_bit_is_set(packed, index))
        .unwrap_or(false);
    state.push(Val::Bool(enabled));
    Ok(1)
}

/// `SetCVarBitfield(name, index, enabled)` — mutate the same printable
/// 6-bit packed format that Blizzard's Lua-side helpers parse.
fn set_cvar_bitfield(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let Some(index) = cvar_bit_index(state, 2) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let enabled = Option::<bool>::from_stack(state, 3)?.unwrap_or(false);
    let current = borrow_state(state)?.cvars.get(&name).unwrap_or_default();
    let updated = set_packed_cvar_bit(&current, index, enabled);
    let accepted = borrow_state_mut(state)?.cvars.set(&name, &updated);
    state.push(Val::Bool(accepted));
    Ok(1)
}

/// `ResetTestCvars()` — retail test harness hook. No-op for the simulator.
fn reset_test_cvars(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn cvar_bit_index(state: &mut LuaState, index: i32) -> Option<usize> {
    let raw = Option::<f64>::from_stack(state, index).ok().flatten()?;
    let rounded = raw.trunc() as i64;
    (rounded > 0).then_some((rounded - 1) as usize)
}

fn packed_cvar_bit_is_set(packed: &str, index: usize) -> bool {
    let bytes = packed.as_bytes();
    let byte_index = 1 + (index / CVAR_BITS_PER_BYTE);
    let Some(encoded) = bytes.get(byte_index).copied() else {
        return false;
    };
    let bit_index = index % CVAR_BITS_PER_BYTE;
    let data = encoded & CVAR_DATA_MASK;
    (data & (1 << bit_index)) != 0
}

fn set_packed_cvar_bit(current: &str, index: usize, enabled: bool) -> String {
    let target_data_index = index / CVAR_BITS_PER_BYTE;
    let mut bytes = current.as_bytes().to_vec();
    if bytes.is_empty() {
        bytes.push(DEFAULT_CVAR_BITFIELD_VERSION);
    }
    while bytes.len() <= target_data_index + 1 {
        bytes.push(CVAR_DATA_TAG);
    }

    let bit_index = index % CVAR_BITS_PER_BYTE;
    let slot = &mut bytes[target_data_index + 1];
    let mut data = *slot & CVAR_DATA_MASK;
    if enabled {
        data |= 1 << bit_index;
    } else {
        data &= !(1 << bit_index);
    }
    *slot = CVAR_DATA_TAG | data;

    while bytes.len() > 1 && bytes.last() == Some(&CVAR_DATA_TAG) {
        bytes.pop();
    }

    String::from_utf8(bytes).expect("packed CVar bitfield should stay ASCII/UTF-8")
}

fn push_optional_string(state: &mut LuaState, value: Option<String>) {
    match value {
        Some(value) => {
            let val = create_string(state, &value);
            state.push(val);
        }
        None => state.push(Val::Nil),
    }
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "SetCVar", set_cvar)?;
    LuaApiMut::register_function(lua, "GetCVar", get_cvar)?;
    LuaApiMut::register_function(lua, "GetCVarDefault", get_cvar_default)?;
    LuaApiMut::register_function(lua, "GetCVarBool", get_cvar_bool)?;
    LuaApiMut::register_function(lua, "RegisterCVar", register_cvar)?;
    LuaApiMut::register_function(lua, "GetCVarBitfield", get_cvar_bitfield)?;
    LuaApiMut::register_function(lua, "SetCVarBitfield", set_cvar_bitfield)?;
    LuaApiMut::register_function(lua, "ResetTestCvars", reset_test_cvars)?;
    install_c_cvar_namespace(lua)?;
    Ok(())
}

/// Register `C_CVar.{Set,Get,GetBool,GetDefault,Register,Get/SetBitfield}CVar`
/// with Rust impls that route through `SimState.cvars`.
fn install_c_cvar_namespace(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    let table_ref = ensure_c_cvar_table(state);
    register_c_cvar_functions(state, table_ref)
}

fn register_c_cvar_functions(state: &mut LuaState, table_ref: GcRef<Table>) -> crate::Result<()> {
    for &(name, func) in C_CVAR_FUNCTIONS {
        crate::lua_bridge::table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn ensure_c_cvar_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(b"C_CVar");
    if let Some(table_ref) = existing_c_cvar_table(state, key) {
        return table_ref;
    }
    insert_c_cvar_table(state, key)
}

fn existing_c_cvar_table(state: &mut LuaState, key: GcRef<LuaString>) -> Option<GcRef<Table>> {
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(r)) = existing {
        return Some(r);
    }
    None
}

fn insert_c_cvar_table(state: &mut LuaState, key: GcRef<LuaString>) -> GcRef<Table> {
    let global = state.global;
    let new_val = crate::lua_api::methods::create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}
