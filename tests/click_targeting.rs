//! Integration tests for click-based targeting and spell casting.
//!
//! Verifies that:
//! - Clicking party member unit frames calls TargetUnit via SecureTemplates
//! - CastSpellBookItem starts a cast from the spellbook
//! - CastSpellByID / CastSpellByName work (used by SECURE_ACTIONS["spell"])
//! - Action bar UseAction click chain casts spells

#[path = "click_targeting/full_ui.rs"]
mod click_targeting_full_ui;
#[path = "click_targeting/forever_regressions.rs"]
mod click_targeting_forever_regressions;
use crate::common;

use click_targeting_full_ui::{drain_test_errors, env_with_full_ui, install_test_error_handler};
use wow_ui_sim::lua_api::WowLuaEnv;

/// Lightweight env — no Blizzard addons, just the Lua API.
fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create env")
}

#[test]
fn battle_pet_unit_queries_default_to_false() {
    let env = env();
    let result: (bool, bool, bool, bool) = env
        .eval(
            r#"
            return
                UnitIsBattlePet("player"),
                UnitIsBattlePetCompanion("player"),
                UnitIsOtherPlayersBattlePet("player"),
                UnitIsWildBattlePet("player")
            "#,
        )
        .unwrap();

    assert_eq!(result, (false, false, false, false));
}

// ── CastSpellBookItem ────────────────────────────────────────────────

#[test]
fn cast_spell_book_item_starts_cast() {
    test_timeout! {
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
}

#[test]
fn cast_spell_book_item_instant_spell() {
    test_timeout! {
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
}

#[test]
fn cast_spell_book_item_blocked_while_casting() {
    test_timeout! {
        let env = env();
        env.exec("TargetUnit('party1')").expect("target party1");

        // Start a cast
        let slot: i32 = env
            .eval("return C_SpellBook.FindSpellBookSlotForSpell(19750) or 0")
            .unwrap();
        env.exec(&format!("C_SpellBook.CastSpellBookItem({slot}, 0)"))
            .expect("first cast");

        // Observe numeric castBarID on the new tuple, legacy castID otherwise.
        let identity_query = if cfg!(feature = "retail-12-1-0") {
            "return select(10, UnitCastingInfo('player'))"
        } else {
            "return select(7, UnitCastingInfo('player'))"
        };
        let cast_id_1: i64 = env.eval(identity_query).unwrap();

        // Try to cast again — should be blocked (already casting)
        env.exec(&format!("C_SpellBook.CastSpellBookItem({slot}, 0)"))
            .expect("second cast attempt");

        let cast_id_2: i64 = env.eval(identity_query).unwrap();
        assert_eq!(cast_id_1, cast_id_2, "second cast should be blocked, cast_id unchanged");
    }
}

// ── CastSpellByID / CastSpellByName ──────────────────────────────────

#[test]
fn cast_spell_by_id_starts_cast() {
    test_timeout! {
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
}

#[test]
fn cast_spell_by_name_starts_cast() {
    test_timeout! {
        let env = env();
        env.exec("TargetUnit('party1')").expect("target party1");

        env.exec("CastSpellByName('Flash of Light')").expect("CastSpellByName");

        let casting: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(casting, "CastSpellByName should start a cast");
    }
}

// ── Party targeting via TargetUnit ───────────────────────────────────

#[test]
fn target_unit_party1_sets_target() {
    test_timeout! {
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
}

#[test]
fn target_unit_party_members_by_index() {
    test_timeout! {
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
}

#[test]
fn clear_target_removes_target() {
    test_timeout! {
        let env = env();
        env.exec("TargetUnit('party1')").expect("TargetUnit");
        assert!(env.eval::<bool>("return UnitExists('target')").unwrap());

        env.exec("ClearTarget()").expect("ClearTarget");
        let has_target: bool = env
            .eval("return UnitExists('target')")
            .unwrap();
        assert!(!has_target, "ClearTarget should remove the target");
    }
}

// ── Secure action chain simulation ───────────────────────────────────
// Simulates what SecureTemplates does: calls TargetUnit/CastSpellByID
// directly (since we don't have the full Blizzard SecureTemplates loaded
// in lightweight tests).

#[test]
fn secure_action_target_calls_target_unit() {
    test_timeout! {
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
}

#[test]
fn click_bindings_default_profile_reports_interaction_but_execute_remains_inert() {
    test_timeout! {
        let env = env();
        env.exec("ClearTarget()").expect("ClearTarget");

        let binding_type: i32 = env
            .eval("return C_ClickBindings.GetBindingType('LeftButton', C_ClickBindings.MakeModifiers())")
            .unwrap();
        assert_eq!(binding_type, 3, "default left click should be an interaction");

        env.exec("C_ClickBindings.ExecuteBinding('party1', 'LeftButton', 0)")
            .expect("unmodeled binding execution should remain inert");
        let target_exists: bool = env.eval("return UnitExists('target')").unwrap();
        assert!(!target_exists);
    }
}

#[test]
fn secure_action_spell_calls_cast_spell_by_id() {
    test_timeout! {
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
}

// ── Action bar UseAction ─────────────────────────────────────────────

#[test]
fn use_action_casts_from_action_bar() {
    test_timeout! {
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
}

#[test]
fn use_action_instant_spell_succeeds() {
    test_timeout! {
        let env = env();
        env.exec("TargetUnit('party1')").expect("target");

        // Slot 2 = Avenger's Shield (instant)
        env.exec("UseAction(2)").expect("UseAction(2)");

        let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
        assert!(!casting, "instant spell should not show casting info");
    }
}

#[test]
fn click_binding_profile_round_trips_and_reset_restores_interactions() {
    let env = env();
    env.exec(r#"
        local api = C_ClickBindings
        local profile = api.GetProfileInfo()
        assert(#profile == 2)
        assert(profile[1].type == 3 and profile[1].actionID == 1 and profile[1].button == 'LeftButton' and profile[1].modifiers == 0)
        assert(profile[2].type == 3 and profile[2].actionID == 2 and profile[2].button == 'RightButton' and profile[2].modifiers == 0)
        assert(api.GetBindingType('RightButton', 0) == 3)
        assert(api.GetBindingType('LeftButton', 1) == 0)
        profile[1].button = 'MiddleButton'
        assert(api.GetBindingType('LeftButton', 0) == 3, 'returned profile must not alias stored state')
        api.SetProfileByInfo({
            {type = 3, actionID = 1, button = 'Button4', modifiers = 2},
            {type = 3, actionID = 2, button = 'LeftButton', modifiers = 0},
        })
        assert(api.GetBindingType('LeftButton', 0) == 3)
        assert(api.GetBindingType('Button4', 2) == 3)
        assert(api.GetBindingType('RightButton', 0) == 0)
        assert(api.GetEffectiveInteractionButton('LeftButton', 0) == 'RightButton')
        assert(api.GetEffectiveInteractionButton('Button4', 2) == 'LeftButton')
        assert(api.GetProfileInfo()[1].button == 'Button4')
        api.ResetCurrentProfile()
        assert(api.GetBindingType('LeftButton', 0) == 3)
        assert(api.GetBindingType('RightButton', 0) == 3)
        assert(api.GetBindingType('Button4', 2) == 0)
    "#).expect("profile queries, edits, and reset should share modeled state");
}

#[test]
fn unchanged_blizzard_secure_unit_handler_targets_player_and_party() {
    common::with_perf_lock(|| {
        let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
        let env = common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
        common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
            &env, &ui, &["Blizzard_AuraContainer", "Blizzard_UnitFrame"], &[],
        );
        env.exec(r#"
            assert(type(SecureUnitButton_OnClick) == 'function')
            A_Admin.SetPartySize(1)
            A_Admin.SetPartyMember(1, 'Healer', 5, 80)
            local button = CreateFrame('Button', 'ClickProfilePartyUnit', UIParent, 'SecureUnitButtonTemplate')
            SecureUnitButton_OnLoad(button, 'party1')
            ClearTarget()
            SecureUnitButton_OnClick(button, 'LeftButton', false)
            assert(UnitName('target') == 'Healer', 'unchanged SecureUnitButton_OnClick must target party1')
            ClearTarget()
            local player = CreateFrame('Button', 'ClickProfilePlayerUnit', UIParent, 'SecureUnitButtonTemplate')
            SecureUnitButton_OnLoad(player, 'player')
            SecureUnitButton_OnClick(player, 'LeftButton', false)
            assert(UnitName('target') == UnitName('player'), 'unchanged handler must target player')
        "#).expect("vendor SecureUnitButton_OnClick should target party and player");
        assert!(env.state().borrow().lua_errors.is_empty());
    });
}

// ── Full Blizzard UI: SecureTemplates click chain ────────────────────

fn assert_blizzard_secure_unit_button_click_targets_party(env: &WowLuaEnv) {
    env.exec("ClearTarget()").expect("ClearTarget");
    create_secure_unit_button(env);
    assert_secure_unit_button_attributes(env);
    click_secure_unit_button(env);
    assert_no_secure_unit_button_click_errors(env);
    assert_target_name(env, "Thrynn", "target should be Thrynn (party1)");
}

fn create_secure_unit_button(env: &WowLuaEnv) {
    {
        let mut state = env.state().borrow_mut();
        state.party_group_active = true;
    }
    env.exec(r#"
            local btn = CreateFrame("Button", "TestSecureUnitBtn", UIParent, "SecureUnitButtonTemplate")
            SecureUnitButton_OnLoad(btn, "party1")
            btn:SetSize(100, 30)
        "#).expect("create SecureUnitButton");

    let errors = drain_test_errors(env);
    let setup_errors: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("SecureUnitButton"))
        .collect();
    assert!(
        setup_errors.is_empty(),
        "SecureUnitButton setup errors: {setup_errors:?}"
    );
}

fn assert_secure_unit_button_attributes(env: &WowLuaEnv) {
    let unit_attr: String = env
        .eval(r#"return TestSecureUnitBtn:GetAttribute("unit") or "none""#)
        .unwrap();
    assert_eq!(unit_attr, "party1", "unit attribute should be party1");

    let type_attr: String = env
        .eval(r#"return TestSecureUnitBtn:GetAttribute("*type1") or "none""#)
        .unwrap();
    assert_eq!(type_attr, "target", "*type1 attribute should be target");
}

fn click_secure_unit_button(env: &WowLuaEnv) {
    env.exec(
        r#"
            local handler = TestSecureUnitBtn:GetScript("OnClick")
            if handler then
                handler(TestSecureUnitBtn, "LeftButton", false)
            end
        "#,
    )
    .expect("click SecureUnitButton");
}

fn assert_no_secure_unit_button_click_errors(env: &WowLuaEnv) {
    let click_errors = drain_test_errors(env);
    let fatal_errors: Vec<&String> = click_errors
        .iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    assert!(
        fatal_errors.is_empty(),
        "SecureUnitButton click errors:\n{}",
        fatal_errors
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn assert_target_name(env: &WowLuaEnv, expected: &str, message: &str) {
    let has_target: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(has_target, "clicking SecureUnitButton should set a target");

    let target_name: String = env.eval("return UnitName('target')").unwrap();
    assert_eq!(target_name, expected, "{message}");
}

fn assert_blizzard_player_frame_click_targets_player(env: &WowLuaEnv) {
    env.exec("ClearTarget()").expect("ClearTarget");

    env.exec(
        r#"
            assert(PlayerFrame and PlayerFrame.Click, "PlayerFrame should be clickable")
            PlayerFrame:Click("LeftButton")
        "#,
    )
    .expect("click PlayerFrame");

    let fatal_errors: Vec<String> = drain_test_errors(env)
        .into_iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    assert!(
        fatal_errors.is_empty(),
        "PlayerFrame click errors:\n{}",
        fatal_errors.join("\n")
    );

    let (target_name, player_name): (String, String) = env
        .eval("return UnitName('target'), UnitName('player')")
        .unwrap();
    assert_eq!(
        target_name,
        player_name,
        "PlayerFrame click should target the player unit; diagnostics: {}",
        env.eval::<String>(r#"return tostring(PlayerFrame:GetScript('OnClick')) .. ' unit=' .. tostring(PlayerFrame:GetAttribute('unit')) .. ' type1=' .. tostring(PlayerFrame:GetAttribute('*type1')) .. ' binding=' .. tostring(C_ClickBindings.GetBindingType('LeftButton', 0))"#).unwrap()
    );
}

fn assert_blizzard_party_member_frame_click_targets_party1(env: &WowLuaEnv) {
    env.exec("ClearTarget()").expect("ClearTarget");
    click_party_member_frame(env);
    assert_no_party_member_click_errors(env);
    assert_target_name(env, "Healer", "party member click should target party1");
}

fn click_party_member_frame(env: &WowLuaEnv) {
    env.exec(
        r#"
            A_Admin.SetPartySize(1)
            A_Admin.SetPartyMember(1, "Healer", 5, 80)
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end

            local member
            if PartyFrame and PartyFrame.PartyMemberFramePool then
                for frame in PartyFrame.PartyMemberFramePool:EnumerateActive() do
                    if frame.unitToken == "party1" then
                        member = frame
                        break
                    end
                end
            end

            assert(member, "party1 member frame should be active")
            local handler = member:GetScript("OnClick")
            assert(handler, "party1 member frame should have an OnClick handler")
            handler(member, "LeftButton", false)
        "#,
    )
    .expect("click party1 member frame");
}

fn assert_no_party_member_click_errors(env: &WowLuaEnv) {
    let fatal_errors: Vec<String> = drain_test_errors(env)
        .into_iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    assert!(
        fatal_errors.is_empty(),
        "party member click errors:\n{}",
        fatal_errors.join("\n")
    );
}

fn assert_blizzard_secure_action_button_click_casts_spell(env: &WowLuaEnv) {
    env.exec("TargetUnit('party1')").expect("target party1");
    create_secure_spell_button(env);
    click_secure_spell_button(env, false, "click spell button");
    log_fatal_spell_click_errors(env);
    retry_secure_spell_button_if_needed(env);
    assert_player_casting_spell(env, "Flash of Light");
}

fn create_secure_spell_button(env: &WowLuaEnv) {
    env.exec(r#"
            local btn = CreateFrame("Button", "TestSpellBtn", UIParent, "SecureActionButtonTemplate")
            btn:SetAttribute("type", "spell")
            btn:SetAttribute("spell", 19750)
            btn:SetSize(40, 40)
        "#).expect("create SecureActionButton");
}

fn click_secure_spell_button(env: &WowLuaEnv, is_down: bool, label: &str) {
    let is_down_lua = if is_down { "true" } else { "false" };
    env.exec(&format!(
        r#"
            local handler = TestSpellBtn:GetScript("OnClick")
            if handler then
                handler(TestSpellBtn, "LeftButton", {is_down_lua})
            end
        "#
    ))
    .expect(label);
}

fn log_fatal_spell_click_errors(env: &WowLuaEnv) {
    let errors = drain_test_errors(env);
    let fatal: Vec<&String> = errors
        .iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    for e in &fatal {
        eprintln!("[spell click error] {e}");
    }
}

fn retry_secure_spell_button_if_needed(env: &WowLuaEnv) {
    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    if casting {
        return;
    }
    click_secure_spell_button(env, true, "click spell button (down=true)");
    let _ = drain_test_errors(env);
}

fn assert_player_casting_spell(env: &WowLuaEnv, expected_spell_name: &str) {
    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(casting, "clicking spell button should start a cast");

    let spell_name: String = env
        .eval("return select(1, UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(spell_name, expected_spell_name);
}

fn assert_blizzard_action_button_click_casts_via_use_action(env: &WowLuaEnv) {
    env.exec("TargetUnit('party1')").expect("target party1");

    let exists: bool = env.eval("return ActionButton1 ~= nil").unwrap();
    assert!(exists, "ActionButton1 should exist");

    click_action_button1(env);
    assert_no_action_button_click_errors(env);

    let casting: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(
        casting,
        "clicking ActionButton1 should start casting Flash of Light"
    );
}

fn click_action_button1(env: &WowLuaEnv) {
    env.exec(
        r#"
            local handler = ActionButton1:GetScript("OnClick")
            if handler then
                handler(ActionButton1, "LeftButton", false)
            end
        "#,
    )
    .expect("click ActionButton1");
}

fn assert_no_action_button_click_errors(env: &WowLuaEnv) {
    let errors = drain_test_errors(env);
    let fatal: Vec<&String> = errors
        .iter()
        .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
        .collect();
    assert!(
        fatal.is_empty(),
        "ActionButton1 click errors:\n{}",
        fatal
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn blizzard_secure_action_button_macrotext_targets_unit() {
    test_timeout! {
        let env = env_with_full_ui();
        install_test_error_handler(&env);
        env.exec("ClearTarget()").expect("clear target");

        env.exec(r#"
            local btn = CreateFrame("Button", "TestMacroTextBtn", UIParent, "SecureActionButtonTemplate")
            btn:SetAttribute("type", "macro")
            btn:SetAttribute("macrotext", "/target party2")
            btn:SetSize(40, 40)
            local handler = btn:GetScript("OnClick")
            if handler then
                handler(btn, "LeftButton", false)
                if UnitName("target") ~= "Kazzara" then
                    handler(btn, "LeftButton", true)
                end
            end
        "#).expect("click macrotext button");

        let fatal: Vec<String> = drain_test_errors(&env)
            .into_iter()
            .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
            .collect();
        assert!(fatal.is_empty(), "macrotext click errors:\n{}", fatal.join("\n"));

        let target_name: String = env.eval("return UnitName('target') or ''").unwrap();
        assert_eq!(target_name, "Kazzara", "macrotext should target party2");
    }
}

#[test]
fn blizzard_secure_unit_button_focus_action_sets_focus_unit() {
    test_timeout! {
        let env = env_with_full_ui();
        install_test_error_handler(&env);
        env.exec("ClearFocus()").expect("clear focus");
        env.state().borrow_mut().party_group_active = true;

        env.exec(r#"
            local btn = CreateFrame("Button", "TestFocusUnitBtn", UIParent, "SecureUnitButtonTemplate")
            SecureUnitButton_OnLoad(btn, "party2")
            btn:SetAttribute("*type1", "focus")
            btn:SetSize(40, 40)
            local handler = btn:GetScript("OnClick")
            if handler then
                handler(btn, "LeftButton", false)
                if UnitName("focus") ~= "Kazzara" then
                    handler(btn, "LeftButton", true)
                end
            end
        "#).expect("click focus unit button");

        let fatal: Vec<String> = drain_test_errors(&env)
            .into_iter()
            .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
            .collect();
        assert!(fatal.is_empty(), "focus click errors:\n{}", fatal.join("\n"));

        let focus_name: String = env.eval("return UnitName('focus') or ''").unwrap();
        assert_eq!(focus_name, "Kazzara", "focus action should focus party2");
    }
}

#[test]
fn blizzard_secure_unit_button_assist_action_targets_assisted_unit_target() {
    test_timeout! {
        let env = env_with_full_ui();
        install_test_error_handler(&env);
        env.exec("ClearTarget()").expect("clear target");
        env.state().borrow_mut().party_group_active = true;

        env.exec(r#"
            local btn = CreateFrame("Button", "TestAssistUnitBtn", UIParent, "SecureUnitButtonTemplate")
            SecureUnitButton_OnLoad(btn, "party1")
            btn:SetAttribute("*type1", "assist")
            btn:SetSize(40, 40)
            local handler = btn:GetScript("OnClick")
            if handler then
                handler(btn, "LeftButton", false)
                if UnitName("target") ~= "Hogger" then
                    handler(btn, "LeftButton", true)
                end
            end
        "#).expect("click assist unit button");

        let fatal: Vec<String> = drain_test_errors(&env)
            .into_iter()
            .filter(|e| !e.contains("C_PingSecure") && !e.contains("ClassResourceBar"))
            .collect();
        assert!(fatal.is_empty(), "assist click errors:\n{}", fatal.join("\n"));

        let target_name: String = env.eval("return UnitName('target') or ''").unwrap();
        assert_eq!(target_name, "Hogger", "assist action should target party1's simulated target");
    }
}

#[test]
fn blizzard_full_ui_click_chain_targets_and_casts() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui();
            install_test_error_handler(&env);
            assert_blizzard_secure_unit_button_click_targets_party(&env);
            assert_blizzard_player_frame_click_targets_player(&env);
            assert_blizzard_party_member_frame_click_targets_party1(&env);
            assert_blizzard_secure_action_button_click_casts_spell(&env);
            assert_blizzard_action_button_click_casts_via_use_action(&env);
        }
    });
}
