//! Integration tests for click-based targeting and spell casting.
//!
//! Verifies that:
//! - Clicking party member unit frames calls TargetUnit via SecureTemplates
//! - CastSpellBookItem starts a cast from the spellbook
//! - CastSpellByID / CastSpellByName work (used by SECURE_ACTIONS["spell"])
//! - Action bar UseAction click chain casts spells

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::global_frames;

/// Lightweight env — no Blizzard addons, just the Lua API.
fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create env")
}

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Full Blizzard UI with SecureTemplates, UnitFrame, ActionBar, etc.
fn env_with_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create env");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    let _ = global_frames::hide_runtime_hidden_frames(env.lua());
    env
}

fn fire_startup_events(env: &WowLuaEnv) {
    let lua = env.lua();
    let _ = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(lua.create_string("WoWUISim").unwrap())],
    );
    for ev in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(ev);
    }
    let _ = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    );
    let _ = env.fire_edit_mode_layouts_updated();
    for ev in ["UPDATE_BINDINGS", "DISPLAY_SIZE_CHANGED", "UI_SCALE_CHANGED"] {
        let _ = env.fire_event(ev);
    }
}

fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(r#"
        __test_errors = {}
        seterrorhandler(function(msg)
            table.insert(__test_errors, tostring(msg))
        end)
    "#).expect("install error handler");
}

fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    let lua = env.lua();
    let table: mlua::Table = match lua.globals().get("__test_errors") {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut errors = Vec::new();
    for entry in table.sequence_values::<String>() {
        if let Ok(msg) = entry {
            errors.push(msg);
        }
    }
    let _ = lua.load("__test_errors = {}").exec();
    errors
}

// ── CastSpellBookItem ────────────────────────────────────────────────

#[test]
fn cast_spell_book_item_starts_cast() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target party1");

    let casting_before: bool = env
        .eval("return UnitCastingInfo('player') ~= nil")
        .unwrap();
    assert!(!casting_before, "should not be casting before CastSpellBookItem");

    // Flash of Light is in the spellbook — find its slot
    let slot: i32 = env
        .eval(r#"
            local slot = C_SpellBook.FindSpellBookSlotForSpell(19750)
            return slot or 0
        "#)
        .unwrap();
    assert!(slot > 0, "Flash of Light should be in the spellbook, slot={slot}");

    env.exec(&format!("C_SpellBook.CastSpellBookItem({slot}, 0)"))
        .expect("CastSpellBookItem");

    let casting_after: bool = env
        .eval("return UnitCastingInfo('player') ~= nil")
        .unwrap();
    assert!(casting_after, "should be casting after CastSpellBookItem");

    let spell_name: String = env
        .eval("return select(1, UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(spell_name, "Flash of Light");
}

#[test]
fn cast_spell_book_item_instant_spell() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target party1");

    // Crusader Strike (35395) is instant — find its slot
    let slot: i32 = env
        .eval(r#"
            local slot = C_SpellBook.FindSpellBookSlotForSpell(35395)
            return slot or 0
        "#)
        .unwrap();
    assert!(slot > 0, "Crusader Strike should be in the spellbook");

    env.exec(&format!("C_SpellBook.CastSpellBookItem({slot}, 0)"))
        .expect("CastSpellBookItem instant");

    // Instant spell should not show UnitCastingInfo
    let casting: bool = env
        .eval("return UnitCastingInfo('player') ~= nil")
        .unwrap();
    assert!(!casting, "instant spell should not show casting info");
}

#[test]
fn cast_spell_book_item_blocked_while_casting() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target party1");

    // Start a cast
    let slot: i32 = env
        .eval("return C_SpellBook.FindSpellBookSlotForSpell(19750) or 0")
        .unwrap();
    env.exec(&format!("C_SpellBook.CastSpellBookItem({slot}, 0)"))
        .expect("first cast");

    let cast_id_1: i64 = env
        .eval("return select(7, UnitCastingInfo('player'))")
        .unwrap();

    // Try to cast again — should be blocked (already casting)
    env.exec(&format!("C_SpellBook.CastSpellBookItem({slot}, 0)"))
        .expect("second cast attempt");

    let cast_id_2: i64 = env
        .eval("return select(7, UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(cast_id_1, cast_id_2, "second cast should be blocked, cast_id unchanged");
}

// ── CastSpellByID / CastSpellByName ──────────────────────────────────

#[test]
fn cast_spell_by_id_starts_cast() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target party1");

    // Flash of Light = 19750
    env.exec("CastSpellByID(19750)").expect("CastSpellByID");

    let casting: bool = env
        .eval("return UnitCastingInfo('player') ~= nil")
        .unwrap();
    assert!(casting, "CastSpellByID should start a cast");

    let spell_name: String = env
        .eval("return select(1, UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(spell_name, "Flash of Light");
}

#[test]
fn cast_spell_by_name_starts_cast() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target party1");

    env.exec("CastSpellByName('Flash of Light')").expect("CastSpellByName");

    let casting: bool = env
        .eval("return UnitCastingInfo('player') ~= nil")
        .unwrap();
    assert!(casting, "CastSpellByName should start a cast");
}

// ── Party targeting via TargetUnit ───────────────────────────────────

#[test]
fn target_unit_party1_sets_target() {
    let env = env();

    let has_target_before: bool = env
        .eval("return UnitExists('target')")
        .unwrap();
    assert!(!has_target_before, "should have no target initially");

    env.exec("TargetUnit('party1')").expect("TargetUnit");

    let has_target: bool = env
        .eval("return UnitExists('target')")
        .unwrap();
    assert!(has_target, "should have a target after TargetUnit('party1')");

    let name: String = env
        .eval("return UnitName('target')")
        .unwrap();
    assert_eq!(name, "Thrynn", "target should be Thrynn (party1)");
}

#[test]
fn target_unit_party_members_by_index() {
    let env = env();
    let expected = [
        ("party1", "Thrynn"),
        ("party2", "Kazzara"),
        ("party3", "Sylvanas"),
        ("party4", "Jaina"),
    ];
    for (unit, expected_name) in expected {
        env.exec(&format!("TargetUnit('{unit}')")).expect("TargetUnit");
        let name: String = env
            .eval("return UnitName('target')")
            .unwrap();
        assert_eq!(name, expected_name, "targeting {unit} should give {expected_name}");
    }
}

#[test]
fn clear_target_removes_target() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("TargetUnit");
    assert!(env.eval::<bool>("return UnitExists('target')").unwrap());

    env.exec("ClearTarget()").expect("ClearTarget");
    let has_target: bool = env
        .eval("return UnitExists('target')")
        .unwrap();
    assert!(!has_target, "ClearTarget should remove the target");
}

// ── Secure action chain simulation ───────────────────────────────────
// Simulates what SecureTemplates does: calls TargetUnit/CastSpellByID
// directly (since we don't have the full Blizzard SecureTemplates loaded
// in lightweight tests).

#[test]
fn secure_action_target_calls_target_unit() {
    let env = env();

    // Simulate what SECURE_ACTIONS["target"] does
    env.exec(r#"
        local unit = "party2"
        if unit and unit ~= "none" then
            TargetUnit(unit)
        end
    "#).expect("secure target action");

    let name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(name, "Kazzara", "SECURE_ACTIONS target should call TargetUnit");
}

#[test]
fn secure_action_spell_calls_cast_spell_by_id() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target");

    // Simulate what SECURE_ACTIONS["spell"] does
    env.exec(r#"
        local spellID = 19750  -- Flash of Light
        if spellID then
            CastSpellByID(spellID, "party1")
        end
    "#).expect("secure spell action");

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(casting, "SECURE_ACTIONS spell should start a cast");
}

// ── Action bar UseAction ─────────────────────────────────────────────

#[test]
fn use_action_casts_from_action_bar() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target");

    // Slot 1 = Flash of Light (setup by default in SimState)
    env.exec("UseAction(1)").expect("UseAction(1)");

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(casting, "UseAction should start a cast");

    let spell_name: String = env
        .eval("return select(1, UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(spell_name, "Flash of Light");
}

#[test]
fn use_action_instant_spell_succeeds() {
    let env = env();
    env.exec("TargetUnit('party1')").expect("target");

    // Slot 2 = Avenger's Shield (instant)
    env.exec("UseAction(2)").expect("UseAction(2)");

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(!casting, "instant spell should not show casting info");
}

// ── Full Blizzard UI: SecureTemplates click chain ────────────────────

#[test]
fn blizzard_secure_unit_button_click_targets_party() {
    let env = env_with_full_ui();
    install_test_error_handler(&env);

    // Clear any existing target
    env.exec("ClearTarget()").expect("ClearTarget");

    // Create a button using SecureUnitButtonTemplate (like party frames do)
    // and simulate clicking it
    env.exec(r#"
        local btn = CreateFrame("Button", "TestSecureUnitBtn", UIParent, "SecureUnitButtonTemplate")
        SecureUnitButton_OnLoad(btn, "party1")
        btn:SetSize(100, 30)
    "#).expect("create SecureUnitButton");

    let errors = drain_test_errors(&env);
    let setup_errors: Vec<&String> = errors.iter()
        .filter(|e| e.contains("SecureUnitButton"))
        .collect();
    assert!(
        setup_errors.is_empty(),
        "SecureUnitButton setup errors: {setup_errors:?}"
    );

    // Verify the button has correct attributes
    let unit_attr: String = env
        .eval(r#"return TestSecureUnitBtn:GetAttribute("unit") or "none""#)
        .unwrap();
    assert_eq!(unit_attr, "party1", "unit attribute should be party1");

    let type_attr: String = env
        .eval(r#"return TestSecureUnitBtn:GetAttribute("*type1") or "none""#)
        .unwrap();
    assert_eq!(type_attr, "target", "*type1 attribute should be target");

    // Click the button via Lua — this calls SecureUnitButton_OnClick
    // which goes through OnActionButtonClick → SECURE_ACTIONS["target"]
    // → TargetUnit("party1")
    env.exec(r#"
        local handler = TestSecureUnitBtn:GetScript("OnClick")
        if handler then
            handler(TestSecureUnitBtn, "LeftButton", false)
        end
    "#).expect("click SecureUnitButton");

    let click_errors = drain_test_errors(&env);
    let fatal_errors: Vec<&String> = click_errors.iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    assert!(
        fatal_errors.is_empty(),
        "SecureUnitButton click errors:\n{}",
        fatal_errors.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
    );

    // Verify the target was set
    let has_target: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(has_target, "clicking SecureUnitButton should set a target");

    let target_name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(target_name, "Thrynn", "target should be Thrynn (party1)");
}

#[test]
fn blizzard_secure_action_button_click_casts_spell() {
    let env = env_with_full_ui();
    install_test_error_handler(&env);
    env.exec("TargetUnit('party1')").expect("target party1");

    // Create a SecureActionButton with type="spell" and spell=19750
    env.exec(r#"
        local btn = CreateFrame("Button", "TestSpellBtn", UIParent, "SecureActionButtonTemplate")
        btn:SetAttribute("type", "spell")
        btn:SetAttribute("spell", 19750)
        btn:SetSize(40, 40)
    "#).expect("create SecureActionButton");

    // Click it — SecureActionButton_OnClick handles down/up logic.
    // With down=true and useOnKeyDown, it fires on key down.
    // With down=false and no useOnKeyDown (default), it fires on key up.
    // The default for mouse clicks is useOnKeyDown=false, so down=false triggers.
    env.exec(r#"
        local handler = TestSpellBtn:GetScript("OnClick")
        if handler then
            handler(TestSpellBtn, "LeftButton", false)
        end
    "#).expect("click spell button");

    let errors = drain_test_errors(&env);
    let fatal: Vec<&String> = errors.iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    for e in &fatal {
        eprintln!("[spell click error] {e}");
    }

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    // SecureActionButton_OnClick checks: clickAction = (down and useOnKeyDown) or (not down and not useOnKeyDown)
    // With down=false and useOnKeyDown=false (default), clickAction = true.
    // But GetCVarBool("ActionButtonUseKeyDown") may return true, making useOnKeyDown=true,
    // which means clickAction = (false and true) or (true and false) = false.
    // In that case the button fires on down=true, not down=false.
    if !casting {
        // Try with down=true (key-down mode)
        env.exec(r#"
            local handler = TestSpellBtn:GetScript("OnClick")
            if handler then
                handler(TestSpellBtn, "LeftButton", true)
            end
        "#).expect("click spell button (down=true)");
        let _ = drain_test_errors(&env);
    }

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(casting, "clicking spell button should start a cast");

    let spell_name: String = env
        .eval("return select(1, UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(spell_name, "Flash of Light");
}

#[test]
fn blizzard_action_button_click_casts_via_use_action() {
    let env = env_with_full_ui();
    install_test_error_handler(&env);
    env.exec("TargetUnit('party1')").expect("target party1");

    // ActionButton1 should exist and have the "action" type
    let exists: bool = env.eval("return ActionButton1 ~= nil").unwrap();
    assert!(exists, "ActionButton1 should exist");

    // Click ActionButton1 — goes through SecureActionButton_OnClick
    // → SECURE_ACTIONS["action"] → UseAction(slot)
    env.exec(r#"
        local handler = ActionButton1:GetScript("OnClick")
        if handler then
            handler(ActionButton1, "LeftButton", false)
        end
    "#).expect("click ActionButton1");

    let errors = drain_test_errors(&env);
    let fatal: Vec<&String> = errors.iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    assert!(
        fatal.is_empty(),
        "ActionButton1 click errors:\n{}",
        fatal.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
    );

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(casting, "clicking ActionButton1 should start casting Flash of Light");
}
