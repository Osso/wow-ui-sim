//! Import action bar and carried-bag state captured by the ServerSnapshot addon.
//!
//! The addon writes account SavedVariables from a real client. During simulator
//! startup we load that file, pick the requested/latest character snapshot, and
//! seed action bars and carried inventory before Blizzard consumers initialize.

mod bags;

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use std::collections::HashMap;

const SERVER_SNAPSHOT_ADDON: &str = "ServerSnapshot";

/// Lua prelude that resolves the snapshot to use into a local `snapshot`.
///
/// Picks `lastCharacterKey` when present, otherwise the newest capture. Leaves
/// `snapshot` as nil when no usable snapshot exists; each consumer below handles
/// that case explicitly.
const CHOOSE_SNAPSHOT_PRELUDE: &str = r#"
local db = rawget(_G, "ServerSnapshotDB")
local function chooseSnapshot()
    if type(db) ~= "table" or type(db.characters) ~= "table" then
        return nil
    end

    local key = db.lastCharacterKey
    if type(key) == "string" and type(db.characters[key]) == "table" then
        return db.characters[key]
    end

    local newestSnapshot = nil
    local newestCapturedAt = nil
    for _, snapshot in pairs(db.characters) do
        if type(snapshot) == "table" then
            local capturedAt = tonumber(snapshot.capturedAt) or 0
            if newestSnapshot == nil or capturedAt > newestCapturedAt then
                newestSnapshot = snapshot
                newestCapturedAt = capturedAt
            end
        end
    end
    return newestSnapshot
end

local snapshot = chooseSnapshot()
"#;

const APPLY_SNAPSHOT_BODY: &str = r#"
if type(snapshot) ~= "table" then
    return 0
end

local keybindings = snapshot.keybindings
if type(keybindings) == "table" then
    local keys = keybindings.keys
    if type(keys) == "table" then
        for key, action in pairs(keys) do
            if type(key) == "string" and type(action) == "string" then
                SetBinding(key, action)
            end
        end
    end
end

local actionBars = snapshot.actionBars
local slots = type(actionBars) == "table" and actionBars.slots or nil
if type(slots) ~= "table" then
    return 0
end

A_Admin.ClearActionBars()

local imported = 0
for slot, entry in pairs(slots) do
    local slotNumber = tonumber(slot)
    if slotNumber ~= nil and type(entry) == "table" and entry.empty ~= true then
        local actionType = entry.type
        local spellID = tonumber(entry.spellID or entry.id)
        if actionType == "spell" and spellID ~= nil then
            A_Admin.SetActionSlot(slotNumber, spellID)
            imported = imported + 1
        end
    end
end

return imported
"#;

const ENABLE_OVERRIDES_BODY: &str = r#"
local entries = type(snapshot) == "table"
    and type(snapshot.addons) == "table"
    and snapshot.addons.entries
    or nil
if type(entries) ~= "table" then
    return ""
end

local rows = {}
for name, entry in pairs(entries) do
    if type(name) == "string" and type(entry) == "table" and type(entry.enabled) == "boolean" then
        table.insert(rows, name .. "=" .. (entry.enabled and "1" or "0"))
    end
end
return table.concat(rows, "\n")
"#;

const EDIT_MODE_LAYOUT_BODY: &str = r#"
local editMode = type(snapshot) == "table" and snapshot.editMode or nil
if type(editMode) ~= "table" then
    return ""
end
return type(editMode.activeLayoutName) == "string" and editMode.activeLayoutName or ""
"#;

fn snapshot_chunk(body: &str) -> String {
    format!("{CHOOSE_SNAPSHOT_PRELUDE}{body}")
}

/// Apply the selected `ServerSnapshotDB` character to action bars and carried bags.
/// Returns the spell-slot count for existing callers; bag import is independent.
pub fn apply_loaded_snapshot(env: &WowLuaEnv) -> crate::Result<i64> {
    let bags = bags::read(env)?;
    let imported = env.eval(&snapshot_chunk(APPLY_SNAPSHOT_BODY))?;
    if let Some(bags) = bags {
        let changed = bags::apply(env, bags);
        bags::notify(env, &changed)?;
    }
    Ok(imported)
}

pub fn load_addon_enable_overrides(
    env: &WowLuaEnv,
    saved_vars: &mut SavedVariablesManager,
) -> crate::Result<HashMap<String, bool>> {
    env.loader_env()
        .with_state(|state| saved_vars.load_wtf_for_addon(state, SERVER_SNAPSHOT_ADDON))?;
    let text: String = env.eval(&snapshot_chunk(ENABLE_OVERRIDES_BODY))?;
    Ok(parse_addon_enable_overrides_text(&text))
}

/// Read the active EditMode layout name captured by ServerSnapshot from a live
/// client. Returns `None` when no snapshot, no captured EditMode data, or an
/// empty layout name is present.
///
/// This feeds the EditMode cache loader as the preferred active layout, so the
/// simulator picks the same layout the live client had instead of relying on
/// the (sometimes stale) WTF `edit-mode-cache-character.txt` index or a manual
/// `WOW_SIM_EDIT_MODE_LAYOUT` override.
pub fn load_edit_mode_layout(
    env: &WowLuaEnv,
    saved_vars: &mut SavedVariablesManager,
) -> crate::Result<Option<String>> {
    env.loader_env()
        .with_state(|state| saved_vars.load_wtf_for_addon(state, SERVER_SNAPSHOT_ADDON))?;
    let name: String = env.eval(&snapshot_chunk(EDIT_MODE_LAYOUT_BODY))?;
    Ok((!name.is_empty()).then_some(name))
}

fn parse_addon_enable_overrides_text(text: &str) -> HashMap<String, bool> {
    text.lines()
        .filter_map(|line| {
            let (name, enabled) = line.split_once('=')?;
            match enabled {
                "1" => Some((name.to_string(), true)),
                "0" => Some((name.to_string(), false)),
                _ => None,
            }
        })
        .collect()
}

/// Load ServerSnapshot SavedVariables from the configured WTF source and apply them.
///
/// Returns the number of spell action slots imported. Missing saved variables are a
/// clean no-op. `load_wtf_for_addon` is idempotent (it tracks already-loaded
/// addons), so this is safe to call after `load_edit_mode_layout` has already
/// pulled `ServerSnapshotDB` into the environment — `apply_loaded_snapshot`
/// no-ops when no usable snapshot is present.
pub fn load_from_saved_variables(
    env: &WowLuaEnv,
    saved_vars: &mut SavedVariablesManager,
) -> crate::Result<i64> {
    env.loader_env()
        .with_state(|state| saved_vars.load_wtf_for_addon(state, SERVER_SNAPSHOT_ADDON))?;
    apply_loaded_snapshot(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_addon_enable_overrides_reads_snapshot_addon_entries() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            ServerSnapshotDB = {
                lastCharacterKey = "Realm/Character",
                characters = {
                    ["Realm/Character"] = {
                        capturedAt = 1,
                        addons = {
                            entries = {
                                EllesmereUI = { enabled = false },
                                BetterBlizzFrames = { enabled = true },
                            },
                        },
                    },
                },
            }
            "#,
        )
        .expect("seed snapshot");
        let mut saved_vars = SavedVariablesManager::new();

        let overrides = load_addon_enable_overrides(&env, &mut saved_vars).expect("overrides");

        assert_eq!(overrides.get("EllesmereUI"), Some(&false));
        assert_eq!(overrides.get("BetterBlizzFrames"), Some(&true));
    }

    #[test]
    fn load_addon_enable_overrides_tolerates_missing_snapshot() {
        let env = WowLuaEnv::new().expect("env");
        let mut saved_vars = SavedVariablesManager::new();

        let overrides = load_addon_enable_overrides(&env, &mut saved_vars).expect("overrides");

        assert!(overrides.is_empty());
    }

    #[test]
    fn load_edit_mode_layout_reads_captured_active_layout_name() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            ServerSnapshotDB = {
                lastCharacterKey = "Realm/Character",
                characters = {
                    ["Realm/Character"] = {
                        capturedAt = 1,
                        editMode = {
                            activeLayout = 3,
                            activeLayoutName = "Ultrawide",
                        },
                    },
                },
            }
            "#,
        )
        .expect("seed snapshot");
        let mut saved_vars = SavedVariablesManager::new();

        let layout = load_edit_mode_layout(&env, &mut saved_vars).expect("layout");

        assert_eq!(layout.as_deref(), Some("Ultrawide"));
    }

    #[test]
    fn load_edit_mode_layout_tolerates_missing_edit_mode_data() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            ServerSnapshotDB = {
                lastCharacterKey = "Realm/Character",
                characters = { ["Realm/Character"] = { capturedAt = 1 } },
            }
            "#,
        )
        .expect("seed snapshot");
        let mut saved_vars = SavedVariablesManager::new();

        let layout = load_edit_mode_layout(&env, &mut saved_vars).expect("layout");

        assert_eq!(layout, None);
    }

    #[test]
    fn apply_loaded_snapshot_imports_keybindings_before_action_bars() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            ServerSnapshotDB = {
                lastCharacterKey = "Realm/Character",
                characters = {
                    ["Realm/Character"] = {
                        capturedAt = 1,
                        keybindings = {
                            keys = {
                                F1 = "",
                                ["CTRL-M"] = "TOGGLEWORLDMAP",
                            },
                        },
                        actionBars = { slots = {} },
                    },
                },
            }
            "#,
        )
        .expect("seed snapshot");

        let imported = apply_loaded_snapshot(&env).expect("apply snapshot");
        let (f1_action, map_action): (String, String) = env
            .eval(r#"return GetBindingAction("F1"), GetBindingAction("CTRL-M")"#)
            .expect("read bindings");

        assert_eq!(imported, 0);
        assert_eq!(f1_action, "");
        assert_eq!(map_action, "TOGGLEWORLDMAP");
    }

    #[test]
    fn apply_loaded_snapshot_ignores_binding_rows_for_key_actions() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            ServerSnapshotDB = {
                lastCharacterKey = "Realm/Character",
                characters = {
                    ["Realm/Character"] = {
                        capturedAt = 1,
                        keybindings = {
                            keys = {
                                C = "TOGGLECHARACTER0",
                            },
                            entries = {
                                {
                                    action = "HOUSING_TOGGLEDECORSNAPMODE",
                                    keys = { "C" },
                                },
                                {
                                    action = "TOGGLEWORLDMAP",
                                    keys = { "CTRL-M" },
                                },
                                {
                                    action = "HOUSING_LAYOUTCAMERA_DOWN",
                                    keys = { "S" },
                                },
                            },
                        },
                        actionBars = { slots = {} },
                    },
                },
            }
            "#,
        )
        .expect("seed snapshot");

        apply_loaded_snapshot(&env).expect("apply snapshot");
        let (c_action, map_action, s_action): (String, String, String) = env
            .eval(r#"return GetBindingAction("C"), GetBindingAction("CTRL-M"), GetBindingAction("S")"#)
            .expect("read bindings");

        assert_eq!(c_action, "TOGGLECHARACTER0");
        assert_eq!(
            map_action, "",
            "binding rows are metadata and must not create unsampled key overrides"
        );
        assert_eq!(
            s_action, "",
            "binding rows are metadata and must not let contextual housing actions shadow defaults"
        );
    }
}
