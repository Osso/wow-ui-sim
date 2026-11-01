//! `C_MacOptions` on a non-macOS host: no OS permission is granted and the
//! OS-shortcut toggle only persists its CVar. Best-guess, native UNKNOWN; see
//! `docs/specs/mac-options.md`.

use crate::lua_api::methods::{borrow_state, create_string_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const OS_SHORTCUTS_CVAR: &str = "MacDisableOsShortcuts";
const GAME_BUNDLE_NAME: &str = "World of Warcraft";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_MacOptions")?;
    let methods: &[(&'static str, rilua::vm::closure::RustFn)] = &[
        ("AreOSShortcutsDisabled", are_os_shortcuts_disabled),
        ("GetGameBundleName", get_game_bundle_name),
        ("HasNewStyleInputMonitoring", permission_not_granted),
        ("IsInputMonitoringEnabled", permission_not_granted),
        ("IsMicrophoneEnabled", permission_not_granted),
        ("IsUniversalAccessEnabled", permission_not_granted),
        ("OpenInputMonitoring", open_system_panel),
        ("OpenMicrophoneRequestDialogue", open_system_panel),
        ("OpenUniversalAccess", open_system_panel),
        ("SetOSShortcutsDisabled", set_os_shortcuts_disabled),
    ];
    for (name, func) in methods {
        table_set_rust_fn_static(state, namespace, name, *func)?;
    }
    Ok(())
}

/// Returns the persisted CVar request and whether the OS applied it; a
/// non-macOS host never applies it.
fn are_os_shortcuts_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let requested = borrow_state(state)?.cvars.get_bool(OS_SHORTCUTS_CVAR);
    state.push(Val::Bool(requested));
    state.push(Val::Bool(false));
    Ok(2)
}

fn set_os_shortcuts_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let disable = !matches!(stack_val(state, 1), Val::Nil | Val::Bool(false));
    let value = if disable { "1" } else { "0" };
    borrow_state(state)?.cvars.set(OS_SHORTCUTS_CVAR, value);
    Ok(0)
}

fn get_game_bundle_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = create_string_static(state, GAME_BUNDLE_NAME);
    state.push(name);
    Ok(1)
}

fn permission_not_granted(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// macOS System Settings panels do not exist on the simulator host.
fn open_system_panel(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
