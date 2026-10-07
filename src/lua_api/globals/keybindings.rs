//! Keybinding global registrations.
//! Backs WoW's binding API against `SimState.keybindings` plus a static command registry.

#[path = "keybindings_c_api.rs"]
mod keybindings_c_api;

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, pcall_function, table_get,
};
use crate::lua_api::script_helpers::call_error_handler_state;
use crate::lua_bridge::{FromStack, IntoStack};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::time::Instant;

// ── Binding action registry ───────────────────────────────────────────────────

/// A binding action definition: action name → the Lua statement(s) to execute.
pub struct BindingAction {
    pub action: &'static str,
    pub lua_code: &'static str,
}

/// A default key→action assignment seeded into `SimState.keybindings` on init.
struct DefaultKey {
    key: &'static str,
    action: &'static str,
}

fn default_action_for_key(key: &str) -> Option<&'static str> {
    DEFAULT_KEYS
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| entry.action)
}

fn default_keys_for_action(action: &str) -> (Option<String>, Option<String>) {
    let mut matches = DEFAULT_KEYS
        .iter()
        .filter(|entry| entry.action == action)
        .map(|entry| entry.key.to_string());
    (matches.next(), matches.next())
}

fn parse_noarg_function_path(lua_code: &str) -> Option<Vec<&str>> {
    let path = lua_code.strip_suffix("()")?;
    let segments = path.split('.').collect::<Vec<_>>();
    if segments.is_empty() {
        return None;
    }
    segments
        .iter()
        .all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
        .then_some(segments)
}

fn dispatch_noarg_function_path(lua: &mut rilua::Lua, path: &[&str]) -> crate::Result<bool> {
    let state = lua.state_mut();
    let mut current = Val::Table(state.global);
    for segment in &path[..path.len().saturating_sub(1)] {
        current = table_get(state, current, segment);
        if !matches!(current, Val::Table(_)) {
            return Ok(false);
        }
    }

    let Some(function_name) = path.last() else {
        return Ok(false);
    };
    let function = table_get(state, current, function_name);
    if !matches!(function, Val::Function(_)) {
        return Ok(false);
    }
    let _ = pcall_function(lua, function, &[]);
    Ok(true)
}

/// Full set of binding actions (mirrors master `keybindings.rs`).
pub const BINDING_ACTIONS: &[BindingAction] = &[
    BindingAction {
        action: "TOGGLEGAMEMENU",
        lua_code: "ToggleGameMenu()",
    },
    BindingAction {
        action: "TOGGLEBACKPACK",
        lua_code: "ToggleBackpack()",
    },
    BindingAction {
        action: "TOGGLEBAG1",
        lua_code: "ToggleBag(4)",
    },
    BindingAction {
        action: "TOGGLEBAG2",
        lua_code: "ToggleBag(3)",
    },
    BindingAction {
        action: "TOGGLEBAG3",
        lua_code: "ToggleBag(2)",
    },
    BindingAction {
        action: "TOGGLEBAG4",
        lua_code: "ToggleBag(1)",
    },
    BindingAction {
        action: "OPENALLBAGS",
        lua_code: "ToggleAllBags()",
    },
    BindingAction {
        action: "TOGGLECHARACTER0",
        lua_code: "ToggleCharacter(\"PaperDollFrame\")",
    },
    BindingAction {
        action: "TOGGLECHARACTER2",
        lua_code: "ToggleCharacter(\"ReputationFrame\")",
    },
    BindingAction {
        action: "TOGGLESPELLBOOK",
        lua_code: "PlayerSpellsUtil.ToggleSpellBookFrame()",
    },
    BindingAction {
        action: "TOGGLEPROFESSIONBOOK",
        lua_code: "ToggleProfessionsBook()",
    },
    BindingAction {
        action: "TOGGLETALENTS",
        lua_code: "PlayerSpellsUtil.ToggleClassTalentFrame()",
    },
    BindingAction {
        action: "TOGGLEACHIEVEMENT",
        lua_code: "ToggleAchievementFrame()",
    },
    BindingAction {
        action: "TOGGLEGROUPFINDER",
        lua_code: "if not PVEFrame_ToggleFrame then C_AddOns.LoadAddOn('Blizzard_GroupFinder') end if PVEFrame_ToggleFrame then PVEFrame_ToggleFrame() end",
    },
    BindingAction {
        action: "TOGGLECOLLECTIONS",
        lua_code: "ToggleCollectionsJournal()",
    },
    BindingAction {
        action: "TOGGLEENCOUNTERJOURNAL",
        lua_code: "ToggleEncounterJournal()",
    },
    BindingAction {
        action: "TOGGLEWORLDMAP",
        lua_code: "ToggleWorldMap()",
    },
    BindingAction {
        action: "TOGGLESOCIAL",
        lua_code: "if not FriendsFrame then C_AddOns.LoadAddOn('Blizzard_FriendsFrame') end if ToggleFriendsFrame then ToggleFriendsFrame() end",
    },
    BindingAction {
        action: "TOGGLEGUILDTAB",
        lua_code: "ToggleGuildFrame()",
    },
    BindingAction {
        action: "TOGGLEQUESTLOG",
        lua_code: "ToggleQuestLog()",
    },
    BindingAction {
        action: "INTERACTTARGET",
        lua_code: "",
    },
    BindingAction {
        action: "TARGETSELF",
        lua_code: "TargetUnit('player')",
    },
    BindingAction {
        action: "TARGETPARTYMEMBER1",
        lua_code: "TargetUnit('party1')",
    },
    BindingAction {
        action: "TARGETPARTYMEMBER2",
        lua_code: "TargetUnit('party2')",
    },
    BindingAction {
        action: "TARGETPARTYMEMBER3",
        lua_code: "TargetUnit('party3')",
    },
    BindingAction {
        action: "TARGETPARTYMEMBER4",
        lua_code: "TargetUnit('party4')",
    },
    BindingAction {
        action: "TARGETNEARESTENEMY",
        lua_code: "TargetUnit('enemy1')",
    },
    BindingAction {
        action: "ACTIONBUTTON1",
        lua_code: "ActionButtonDown(1) ActionButtonUp(1)",
    },
    BindingAction {
        action: "ACTIONBUTTON2",
        lua_code: "ActionButtonDown(2) ActionButtonUp(2)",
    },
    BindingAction {
        action: "ACTIONBUTTON3",
        lua_code: "ActionButtonDown(3) ActionButtonUp(3)",
    },
    BindingAction {
        action: "ACTIONBUTTON4",
        lua_code: "ActionButtonDown(4) ActionButtonUp(4)",
    },
    BindingAction {
        action: "ACTIONBUTTON5",
        lua_code: "ActionButtonDown(5) ActionButtonUp(5)",
    },
    BindingAction {
        action: "ACTIONBUTTON6",
        lua_code: "ActionButtonDown(6) ActionButtonUp(6)",
    },
    BindingAction {
        action: "ACTIONBUTTON7",
        lua_code: "ActionButtonDown(7) ActionButtonUp(7)",
    },
    BindingAction {
        action: "ACTIONBUTTON8",
        lua_code: "ActionButtonDown(8) ActionButtonUp(8)",
    },
    BindingAction {
        action: "ACTIONBUTTON9",
        lua_code: "ActionButtonDown(9) ActionButtonUp(9)",
    },
    BindingAction {
        action: "ACTIONBUTTON10",
        lua_code: "ActionButtonDown(10) ActionButtonUp(10)",
    },
    BindingAction {
        action: "ACTIONBUTTON11",
        lua_code: "ActionButtonDown(11) ActionButtonUp(11)",
    },
    BindingAction {
        action: "ACTIONBUTTON12",
        lua_code: "ActionButtonDown(12) ActionButtonUp(12)",
    },
    // Simulator-only bindings
    BindingAction {
        action: "TOGGLESIMCOMMANDS",
        lua_code: "if SimCommands then SimCommands:Toggle() end",
    },
    BindingAction {
        action: "QUIT",
        lua_code: "Quit()",
    },
];

fn binding_registry_action_index(action: &str) -> Option<usize> {
    BINDING_ACTIONS
        .iter()
        .position(|binding| binding.action == action)
        .map(|index| index + 1)
}

fn push_binding_row(state: &mut LuaState, action: &str) -> LuaResult<u32> {
    let (binding1, binding2) = default_keys_for_action(action);
    let action = create_string(state, action);
    let category = create_string(state, "BINDING_HEADER_OTHER");
    state.push(action);
    state.push(category);
    push_opt_string(state, binding1);
    push_opt_string(state, binding2);
    Ok(4)
}

/// Default key→action assignments seeded by `init_keybindings`.
const DEFAULT_KEYS: &[DefaultKey] = &[
    DefaultKey {
        key: "ESCAPE",
        action: "TOGGLEGAMEMENU",
    },
    DefaultKey {
        key: "BACKSPACE",
        action: "TOGGLEBACKPACK",
    },
    DefaultKey {
        key: "F8",
        action: "TOGGLEBAG1",
    },
    DefaultKey {
        key: "F9",
        action: "TOGGLEBAG2",
    },
    DefaultKey {
        key: "F10",
        action: "TOGGLEBAG3",
    },
    DefaultKey {
        key: "F11",
        action: "TOGGLEBAG4",
    },
    DefaultKey {
        key: "B",
        action: "OPENALLBAGS",
    },
    DefaultKey {
        key: "C",
        action: "TOGGLECHARACTER0",
    },
    DefaultKey {
        key: "U",
        action: "TOGGLECHARACTER2",
    },
    DefaultKey {
        key: "S",
        action: "TOGGLESPELLBOOK",
    },
    DefaultKey {
        key: "N",
        action: "TOGGLETALENTS",
    },
    DefaultKey {
        key: "A",
        action: "TOGGLEACHIEVEMENT",
    },
    DefaultKey {
        key: "L",
        action: "TOGGLEGROUPFINDER",
    },
    DefaultKey {
        key: "O",
        action: "TOGGLESOCIAL",
    },
    DefaultKey {
        key: "J",
        action: "TOGGLEGUILDTAB",
    },
    DefaultKey {
        key: "M",
        action: "TOGGLEWORLDMAP",
    },
    DefaultKey {
        key: "F1",
        action: "TARGETSELF",
    },
    DefaultKey {
        key: "F2",
        action: "TARGETPARTYMEMBER1",
    },
    DefaultKey {
        key: "F3",
        action: "TARGETPARTYMEMBER2",
    },
    DefaultKey {
        key: "F4",
        action: "TARGETPARTYMEMBER3",
    },
    DefaultKey {
        key: "F5",
        action: "TARGETPARTYMEMBER4",
    },
    DefaultKey {
        key: "F6",
        action: "TARGETNEARESTENEMY",
    },
    DefaultKey {
        key: "TAB",
        action: "TARGETNEARESTENEMY",
    },
    DefaultKey {
        key: "1",
        action: "ACTIONBUTTON1",
    },
    DefaultKey {
        key: "2",
        action: "ACTIONBUTTON2",
    },
    DefaultKey {
        key: "3",
        action: "ACTIONBUTTON3",
    },
    DefaultKey {
        key: "4",
        action: "ACTIONBUTTON4",
    },
    DefaultKey {
        key: "5",
        action: "ACTIONBUTTON5",
    },
    DefaultKey {
        key: "6",
        action: "ACTIONBUTTON6",
    },
    DefaultKey {
        key: "7",
        action: "ACTIONBUTTON7",
    },
    DefaultKey {
        key: "8",
        action: "ACTIONBUTTON8",
    },
    DefaultKey {
        key: "9",
        action: "ACTIONBUTTON9",
    },
    DefaultKey {
        key: "0",
        action: "ACTIONBUTTON10",
    },
    DefaultKey {
        key: "-",
        action: "ACTIONBUTTON11",
    },
    DefaultKey {
        key: "=",
        action: "ACTIONBUTTON12",
    },
    DefaultKey {
        key: "CTRL-P",
        action: "TOGGLESIMCOMMANDS",
    },
    DefaultKey {
        key: "CTRL-Q",
        action: "QUIT",
    },
];

/// Keep user bindings empty; default key dispatch reads `DEFAULT_KEYS` directly.
pub fn init_keybindings(state: &mut crate::lua_api::SimState) {
    state.keybindings.base.clear();
    state.keybindings.unbound.clear();
    state.keybindings.overrides.clear();
}

/// Look up `key` in the binding registry and execute the bound Lua code.
///
/// Lookup priority:
/// 1. `SimState.keybindings` — user-set bindings (via `SetBinding`), which
///    also holds overrides. Shadows the defaults for the matched key.
/// 2. `DEFAULT_KEYS` — simulator defaults; NOT stored in `SimState` so they
///    do not inflate `GetNumBindings`.
///
/// Returns `true` if a binding was found and executed, `false` otherwise.
pub fn dispatch_key_binding(lua: &mut rilua::Lua, key: &str) -> crate::Result<bool> {
    let (user_action, shadows_default) = {
        let sim = borrow_state(lua.state_mut())?;
        let user_action = sim.keybindings.action_for_key(key);
        (user_action, sim.keybindings.shadows_default_key(key))
    };
    let action = if !user_action.is_empty() {
        user_action
    } else if shadows_default {
        String::new()
    } else {
        default_action_for_key(key)
            .map(str::to_string)
            .unwrap_or_default()
    };
    if action.is_empty() {
        return Ok(false);
    }
    let Some(ba) = BINDING_ACTIONS.iter().find(|b| b.action == action) else {
        return Ok(false);
    };
    crate::logging::eprintln_elapsed(&format!("[keybind] {key} → {action} → {}", ba.lua_code));
    let exec_started = Instant::now();
    let handled = if let Some(path) = parse_noarg_function_path(ba.lua_code) {
        dispatch_noarg_function_path(lua, &path)?
    } else {
        false
    };
    if !handled && let Err(error) = lua.exec(ba.lua_code) {
        call_error_handler_state(lua.state_mut(), &error.to_string());
    }
    crate::logging::eprintln_elapsed(&format!(
        "[keybind] {key} executed in {:.1?}",
        exec_started.elapsed()
    ));
    Ok(true)
}

fn push_opt_string(state: &mut LuaState, val: Option<String>) {
    match val {
        Some(s) => {
            let v = create_string(state, &s);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
}

pub fn get_binding_key(state: &mut LuaState) -> LuaResult<u32> {
    let action = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let (mut k1, mut k2) = {
        let sim = borrow_state(state)?;
        sim.keybindings.keys_for_action(&action)
    };
    if k1.is_none() && k2.is_none() {
        (k1, k2) = default_keys_for_action(&action);
    }
    push_opt_string(state, k1);
    push_opt_string(state, k2);
    Ok(2)
}

pub fn get_binding_key_for_action(state: &mut LuaState) -> LuaResult<u32> {
    let action = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let (mut k1, _) = {
        let sim = borrow_state(state)?;
        sim.keybindings.keys_for_action(&action)
    };
    if k1.is_none() {
        let (fallback, _) = default_keys_for_action(&action);
        k1 = fallback;
    }
    push_opt_string(state, k1);
    Ok(1)
}

pub fn get_binding_action(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let action = {
        let sim = borrow_state(state)?;
        sim.keybindings.action_for_key(&key)
    };
    create_string(state, &action).into_stack(state)
}

pub fn get_binding(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1).unwrap_or(0);
    let sim = borrow_state(state)?;
    let idx_0 = (index - 1) as usize;
    let user_binding_count = sim.keybindings.base.len();
    let entry = sim.keybindings.base.get(idx_0).cloned();
    drop(sim);
    match entry {
        Some((key, action)) => {
            let action_v = create_string(state, &action);
            let key_v = create_string(state, &key);
            state.push(action_v);
            state.push(key_v);
            Ok(2)
        }
        None => {
            let registry_index = idx_0.saturating_sub(user_binding_count);
            let Some(binding) = BINDING_ACTIONS.get(registry_index) else {
                state.push(Val::Nil);
                state.push(Val::Nil);
                return Ok(2);
            };
            push_binding_row(state, binding.action)
        }
    }
}

pub fn get_num_bindings(state: &mut LuaState) -> LuaResult<u32> {
    let user_bindings = borrow_state(state)?.keybindings.base.len();
    let n = (user_bindings + BINDING_ACTIONS.len()) as i32;
    n.into_stack(state)
}

pub fn get_current_binding_set(state: &mut LuaState) -> LuaResult<u32> {
    (1i32).into_stack(state)
}

pub fn get_binding_text(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?;
    match key {
        Some(k) => create_string(state, &k).into_stack(state),
        None => create_string(state, "").into_stack(state),
    }
}

pub fn is_binding_for_game_pad(state: &mut LuaState) -> LuaResult<u32> {
    false.into_stack(state)
}

pub fn set_binding(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let action = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    if key.is_empty() {
        return false.into_stack(state);
    }
    borrow_state_mut(state)?.keybindings.set(&key, &action);
    true.into_stack(state)
}

pub fn set_binding_click(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let button_name = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    if key.is_empty() || button_name.is_empty() {
        return false.into_stack(state);
    }
    let action = format!("CLICK {button_name}:LeftButton");
    borrow_state_mut(state)?.keybindings.set(&key, &action);
    true.into_stack(state)
}

pub fn set_binding_spell(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let spell = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    if key.is_empty() || spell.is_empty() {
        return false.into_stack(state);
    }
    let action = format!("SPELL {spell}");
    borrow_state_mut(state)?.keybindings.set(&key, &action);
    true.into_stack(state)
}

pub fn set_binding_item(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let item = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    if key.is_empty() || item.is_empty() {
        return false.into_stack(state);
    }
    let action = format!("ITEM {item}");
    borrow_state_mut(state)?.keybindings.set(&key, &action);
    true.into_stack(state)
}

pub fn set_binding_macro(state: &mut LuaState) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let macro_name = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    if key.is_empty() || macro_name.is_empty() {
        return false.into_stack(state);
    }
    let action = format!("MACRO {macro_name}");
    borrow_state_mut(state)?.keybindings.set(&key, &action);
    true.into_stack(state)
}

pub fn set_override_binding(state: &mut LuaState) -> LuaResult<u32> {
    // Args: owner (frame), isPriority (bool), key (string), action (string?)
    let action = Option::<String>::from_stack(state, 4)?.unwrap_or_default();
    set_override_action(state, action)
}

fn set_override_action(state: &mut LuaState, action: String) -> LuaResult<u32> {
    let key = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    if key.is_empty() {
        return Ok(0);
    }
    borrow_state_mut(state)?
        .keybindings
        .set_override(&key, &action);
    Ok(0)
}

pub fn set_override_binding_click(state: &mut LuaState) -> LuaResult<u32> {
    let button_name = Option::<String>::from_stack(state, 4)?.unwrap_or_default();
    if button_name.is_empty() {
        return Ok(0);
    }
    let mouse_button = Option::<String>::from_stack(state, 5)?
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "LeftButton".to_string());
    set_override_action(state, format!("CLICK {button_name}:{mouse_button}"))
}

pub fn set_override_binding_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell = Option::<String>::from_stack(state, 4)?.unwrap_or_default();
    if spell.is_empty() {
        return Ok(0);
    }
    set_override_action(state, format!("SPELL {spell}"))
}

pub fn set_override_binding_item(state: &mut LuaState) -> LuaResult<u32> {
    let item = Option::<String>::from_stack(state, 4)?.unwrap_or_default();
    if item.is_empty() {
        return Ok(0);
    }
    set_override_action(state, format!("ITEM {item}"))
}

pub fn set_override_binding_macro(state: &mut LuaState) -> LuaResult<u32> {
    let macro_name = Option::<String>::from_stack(state, 4)?.unwrap_or_default();
    if macro_name.is_empty() {
        return Ok(0);
    }
    set_override_action(state, format!("MACRO {macro_name}"))
}

pub fn clear_override_bindings(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.keybindings.clear_overrides();
    Ok(0)
}

pub fn save_bindings(_state: &mut LuaState) -> LuaResult<u32> {
    // No disk persistence in the sim — values live in SimState for the
    // lifetime of the env.
    Ok(0)
}

pub fn load_bindings(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// Register all binding-related global functions on the rilua VM.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_binding_read_globals(lua)?;
    register_binding_write_globals(lua)?;
    register_override_binding_globals(lua)?;
    register_binding_persistence_globals(lua)?;
    keybindings_c_api::register_c_keybindings_namespace(lua)
}

fn register_binding_read_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetBindingKey", get_binding_key)?;
    LuaApiMut::register_function(lua, "GetBindingKeyForAction", get_binding_key_for_action)?;
    LuaApiMut::register_function(lua, "GetBindingAction", get_binding_action)?;
    LuaApiMut::register_function(lua, "GetBinding", get_binding)?;
    LuaApiMut::register_function(lua, "GetNumBindings", get_num_bindings)?;
    LuaApiMut::register_function(lua, "GetCurrentBindingSet", get_current_binding_set)?;
    LuaApiMut::register_function(lua, "GetBindingText", get_binding_text)?;
    LuaApiMut::register_function(lua, "IsBindingForGamePad", is_binding_for_game_pad)?;
    Ok(())
}

fn register_binding_write_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "SetBinding", set_binding)?;
    LuaApiMut::register_function(lua, "SetBindingClick", set_binding_click)?;
    LuaApiMut::register_function(lua, "SetBindingSpell", set_binding_spell)?;
    LuaApiMut::register_function(lua, "SetBindingItem", set_binding_item)?;
    LuaApiMut::register_function(lua, "SetBindingMacro", set_binding_macro)?;
    Ok(())
}

fn register_override_binding_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "SetOverrideBinding", set_override_binding)?;
    LuaApiMut::register_function(lua, "SetOverrideBindingClick", set_override_binding_click)?;
    LuaApiMut::register_function(lua, "SetOverrideBindingSpell", set_override_binding_spell)?;
    LuaApiMut::register_function(lua, "SetOverrideBindingItem", set_override_binding_item)?;
    LuaApiMut::register_function(lua, "SetOverrideBindingMacro", set_override_binding_macro)?;
    LuaApiMut::register_function(lua, "ClearOverrideBindings", clear_override_bindings)?;
    Ok(())
}

fn register_binding_persistence_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "SaveBindings", save_bindings)?;
    LuaApiMut::register_function(lua, "LoadBindings", load_bindings)?;
    Ok(())
}
