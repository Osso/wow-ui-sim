//! WoW keybinding system: action definitions, key→action mapping, and dispatch.
//!
//! Bindings are stored in two Lua tables on the registry:
//! - `__wow_binding_actions`: ordered list of `{ action, lua_code }` entries
//! - `__wow_key_bindings`: key name → action name mapping
//!
//! Key dispatch checks `__wow_key_bindings` for the pressed key, looks up
//! the action's Lua code in `__wow_binding_actions`, and executes it.

use mlua::{Lua, Result, Value};

/// A binding action definition: action name and the Lua code to execute.
struct BindingAction {
    action: &'static str,
    lua_code: &'static str,
}

/// A default key assignment: key name → action name.
struct DefaultKey {
    key: &'static str,
    action: &'static str,
}

/// Binding actions from Bindings_Standard.xml (action name → Lua code).
const BINDING_ACTIONS: &[BindingAction] = &[
    BindingAction { action: "TOGGLEGAMEMENU", lua_code: "ToggleGameMenu()" },
    BindingAction { action: "TOGGLEBACKPACK", lua_code: "ToggleBackpack()" },
    BindingAction { action: "TOGGLEBAG1", lua_code: "ToggleBag(4)" },
    BindingAction { action: "TOGGLEBAG2", lua_code: "ToggleBag(3)" },
    BindingAction { action: "TOGGLEBAG3", lua_code: "ToggleBag(2)" },
    BindingAction { action: "TOGGLEBAG4", lua_code: "ToggleBag(1)" },
    BindingAction { action: "OPENALLBAGS", lua_code: "ToggleAllBags()" },
    BindingAction { action: "TOGGLECHARACTER0", lua_code: "ToggleCharacter(\"PaperDollFrame\")" },
    BindingAction { action: "TOGGLECHARACTER2", lua_code: "ToggleCharacter(\"ReputationFrame\")" },
    BindingAction { action: "TOGGLESPELLBOOK", lua_code: "PlayerSpellsUtil.ToggleSpellBookFrame()" },
    BindingAction { action: "TOGGLETALENTS", lua_code: "PlayerSpellsUtil.ToggleClassTalentFrame()" },
    BindingAction { action: "TOGGLEACHIEVEMENT", lua_code: "ToggleAchievementFrame()" },
    BindingAction { action: "TOGGLEGROUPFINDER", lua_code: "if not PVEFrame_ToggleFrame then LoadAddOn('Blizzard_GroupFinder') end if PVEFrame_ToggleFrame then PVEFrame_ToggleFrame() end" },
    BindingAction { action: "TOGGLECOLLECTIONS", lua_code: "ToggleCollectionsJournal()" },
    BindingAction { action: "TOGGLEENCOUNTERJOURNAL", lua_code: "ToggleEncounterJournal()" },
    BindingAction { action: "TOGGLEWORLDMAP", lua_code: "ToggleWorldMap()" },
    BindingAction { action: "TOGGLESOCIAL", lua_code: "if not ToggleFriendsFrame then LoadAddOn('Blizzard_FriendsFrame') end if ToggleFriendsFrame then ToggleFriendsFrame() end" },
    BindingAction { action: "TOGGLEGUILDTAB", lua_code: "ToggleGuildFrame()" },
    BindingAction { action: "TOGGLEQUESTLOG", lua_code: "ToggleQuestLog()" },
    BindingAction { action: "TARGETSELF", lua_code: "TargetUnit('player')" },
    BindingAction { action: "TARGETPARTYMEMBER1", lua_code: "TargetUnit('party1')" },
    BindingAction { action: "TARGETPARTYMEMBER2", lua_code: "TargetUnit('party2')" },
    BindingAction { action: "TARGETPARTYMEMBER3", lua_code: "TargetUnit('party3')" },
    BindingAction { action: "TARGETPARTYMEMBER4", lua_code: "TargetUnit('party4')" },
    BindingAction { action: "TARGETNEARESTENEMY", lua_code: "TargetUnit('enemy1')" },
    BindingAction { action: "ACTIONBUTTON1", lua_code: "ActionButtonDown(1) UseAction(1) ActionButtonUp(1)" },
    BindingAction { action: "ACTIONBUTTON2", lua_code: "ActionButtonDown(2) UseAction(2) ActionButtonUp(2)" },
    BindingAction { action: "ACTIONBUTTON3", lua_code: "ActionButtonDown(3) UseAction(3) ActionButtonUp(3)" },
    BindingAction { action: "ACTIONBUTTON4", lua_code: "ActionButtonDown(4) UseAction(4) ActionButtonUp(4)" },
    BindingAction { action: "ACTIONBUTTON5", lua_code: "ActionButtonDown(5) UseAction(5) ActionButtonUp(5)" },
    BindingAction { action: "ACTIONBUTTON6", lua_code: "ActionButtonDown(6) UseAction(6) ActionButtonUp(6)" },
    BindingAction { action: "ACTIONBUTTON7", lua_code: "ActionButtonDown(7) UseAction(7) ActionButtonUp(7)" },
    BindingAction { action: "ACTIONBUTTON8", lua_code: "ActionButtonDown(8) UseAction(8) ActionButtonUp(8)" },
    BindingAction { action: "ACTIONBUTTON9", lua_code: "ActionButtonDown(9) UseAction(9) ActionButtonUp(9)" },
    BindingAction { action: "ACTIONBUTTON10", lua_code: "ActionButtonDown(10) UseAction(10) ActionButtonUp(10)" },
    BindingAction { action: "ACTIONBUTTON11", lua_code: "ActionButtonDown(11) UseAction(11) ActionButtonUp(11)" },
    BindingAction { action: "ACTIONBUTTON12", lua_code: "ActionButtonDown(12) UseAction(12) ActionButtonUp(12)" },
];

/// Default key→action assignments (WoW defaults + simulator overrides).
const DEFAULT_KEYS: &[DefaultKey] = &[
    DefaultKey { key: "BACKSPACE", action: "TOGGLEBACKPACK" },
    DefaultKey { key: "F8", action: "TOGGLEBAG1" },
    DefaultKey { key: "F9", action: "TOGGLEBAG2" },
    DefaultKey { key: "F10", action: "TOGGLEBAG3" },
    DefaultKey { key: "F11", action: "TOGGLEBAG4" },
    DefaultKey { key: "B", action: "OPENALLBAGS" },
    DefaultKey { key: "C", action: "TOGGLECHARACTER0" },
    DefaultKey { key: "U", action: "TOGGLECHARACTER2" },
    DefaultKey { key: "S", action: "TOGGLESPELLBOOK" },
    DefaultKey { key: "N", action: "TOGGLETALENTS" },
    DefaultKey { key: "A", action: "TOGGLEACHIEVEMENT" },
    DefaultKey { key: "L", action: "TOGGLEGROUPFINDER" },
    DefaultKey { key: "O", action: "TOGGLESOCIAL" },
    DefaultKey { key: "J", action: "TOGGLEGUILDTAB" },
    DefaultKey { key: "M", action: "TOGGLEWORLDMAP" },
    DefaultKey { key: "F1", action: "TARGETSELF" },
    DefaultKey { key: "F2", action: "TARGETPARTYMEMBER1" },
    DefaultKey { key: "F3", action: "TARGETPARTYMEMBER2" },
    DefaultKey { key: "F4", action: "TARGETPARTYMEMBER3" },
    DefaultKey { key: "F5", action: "TARGETPARTYMEMBER4" },
    DefaultKey { key: "F6", action: "TARGETNEARESTENEMY" },
    DefaultKey { key: "TAB", action: "TARGETNEARESTENEMY" },
    DefaultKey { key: "1", action: "ACTIONBUTTON1" },
    DefaultKey { key: "2", action: "ACTIONBUTTON2" },
    DefaultKey { key: "3", action: "ACTIONBUTTON3" },
    DefaultKey { key: "4", action: "ACTIONBUTTON4" },
    DefaultKey { key: "5", action: "ACTIONBUTTON5" },
    DefaultKey { key: "6", action: "ACTIONBUTTON6" },
    DefaultKey { key: "7", action: "ACTIONBUTTON7" },
    DefaultKey { key: "8", action: "ACTIONBUTTON8" },
    DefaultKey { key: "9", action: "ACTIONBUTTON9" },
    DefaultKey { key: "0", action: "ACTIONBUTTON10" },
    DefaultKey { key: "-", action: "ACTIONBUTTON11" },
    DefaultKey { key: "=", action: "ACTIONBUTTON12" },
];

/// Initialize the binding tables in Lua and populate with defaults.
pub fn init_keybindings(lua: &Lua) -> Result<()> {
    let actions_table = lua.create_table()?;
    let keys_table = lua.create_table()?;

    for (i, ba) in BINDING_ACTIONS.iter().enumerate() {
        let entry = lua.create_table()?;
        entry.set("action", ba.action)?;
        entry.set("lua_code", ba.lua_code)?;
        actions_table.set(ba.action, entry)?;
        // Also store by index (1-based) for GetBinding/GetNumBindings
        actions_table.set(i + 1, ba.action)?;
    }

    for dk in DEFAULT_KEYS {
        keys_table.set(dk.key, dk.action)?;
    }

    lua.set_named_registry_value("__wow_binding_actions", actions_table)?;
    lua.set_named_registry_value("__wow_key_bindings", keys_table)?;

    Ok(())
}

/// Look up a key press in the binding table and execute the bound action.
/// Returns true if a binding was found and executed.
pub fn dispatch_key_binding(lua: &Lua, key: &str) -> Result<bool> {
    let keys: mlua::Table = lua.named_registry_value("__wow_key_bindings")?;
    let action: Option<String> = keys.get(key)?;
    let Some(action) = action else {
        return Ok(false);
    };

    let actions: mlua::Table = lua.named_registry_value("__wow_binding_actions")?;
    let entry: Option<mlua::Table> = actions.get(action.as_str())?;
    let Some(entry) = entry else {
        return Ok(false);
    };

    let lua_code: String = entry.get("lua_code")?;
    eprintln!("[keybind] {} → {} → {}", key, action, lua_code);
    lua.load(&lua_code).exec()?;
    Ok(true)
}

/// Get the key(s) bound to an action. Returns up to 2 keys (WoW API contract).
pub fn get_binding_key(lua: &Lua, action: &str) -> Result<(Option<String>, Option<String>)> {
    let keys: mlua::Table = lua.named_registry_value("__wow_key_bindings")?;
    let mut found = Vec::new();
    for pair in keys.pairs::<String, String>() {
        let (k, v) = pair?;
        if v == action && found.len() < 2 {
            found.push(k);
        }
    }
    Ok((found.first().cloned(), found.get(1).cloned()))
}

/// Get the action bound to a key.
pub fn get_binding_action(lua: &Lua, key: &str) -> Result<Option<String>> {
    let keys: mlua::Table = lua.named_registry_value("__wow_key_bindings")?;
    keys.get(key)
}

/// Set or clear a key binding. If action is None, the key is unbound.
pub fn set_binding(lua: &Lua, key: &str, action: Option<&str>) -> Result<bool> {
    let keys: mlua::Table = lua.named_registry_value("__wow_key_bindings")?;
    match action {
        Some(a) => keys.set(key, a)?,
        None => keys.set(key, Value::Nil)?,
    }
    Ok(true)
}

/// Get the total number of binding actions.
pub fn get_num_bindings(_lua: &Lua) -> Result<i32> {
    Ok(BINDING_ACTIONS.len() as i32)
}

/// Get binding at index (1-based). Returns (action, header, key1, key2).
#[allow(clippy::type_complexity)]
pub fn get_binding_at(lua: &Lua, index: i32) -> Result<(Option<String>, Option<String>, Option<String>, Option<String>)> {
    if index < 1 || index as usize > BINDING_ACTIONS.len() {
        return Ok((None, None, None, None));
    }
    let actions: mlua::Table = lua.named_registry_value("__wow_binding_actions")?;
    let action_name: Option<String> = actions.get(index)?;
    let Some(ref action) = action_name else {
        return Ok((None, None, None, None));
    };
    let (key1, key2) = get_binding_key(lua, action)?;
    Ok((action_name, None, key1, key2))
}
