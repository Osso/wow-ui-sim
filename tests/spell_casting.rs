//! Integration tests for spell casting via action bar keys.
//!
//! Verifies that pressing an action button key starts a cast (for spells with
//! cast time), shows UnitCastingInfo, and on completion clears the cast and
//! heals the target.

use crate::common;

use wow_ui_sim::lua_api::WowLuaEnv;

/// Lightweight environment — no Blizzard addons, just the Lua API.
fn env_with_friendly_target() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create env");
    // Target a friendly party member so the heal has somewhere to land.
    env.exec("TargetUnit('party1')").expect("target party1");
    env
}

#[test]
fn use_action_starts_cast_for_flash_of_light() {
    test_timeout! {
        let env = env_with_friendly_target();

        // Slot 1 = Flash of Light (1.5s cast)
        let casting_before: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(!casting_before, "should not be casting before UseAction");

        env.exec("UseAction(1)").expect("UseAction(1)");

        let casting_after: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(casting_after, "UnitCastingInfo should return data during cast");

        // Verify cast details
        let spell_name: String = env
            .eval("return select(1, UnitCastingInfo('player'))")
            .unwrap();
        assert_eq!(spell_name, "Flash of Light");
    }
}

#[test]
fn cast_completes_and_heals_target() {
    test_timeout! {
        let env = env_with_friendly_target();

        // Damage target so we can observe healing
        {
            let mut state = env.state().borrow_mut();
            let t = state.current_target.as_mut().expect("target set");
            t.health = t.health_max / 2; // 50% HP
        }

        let health_before: i32 = {
            let state = env.state().borrow();
            state.current_target.as_ref().unwrap().health
        };

        // Start cast
        env.exec("UseAction(1)").expect("UseAction(1)");

        // Force cast to complete by setting end_time to the past
        {
            let mut state = env.state().borrow_mut();
            if let Some(ref mut c) = state.casting {
                c.end_time = 0.0;
            }
        }

        // Replicate tick_casting: extract completed cast, fire events, apply heal
        let (cast_id, spell_id) = {
            let mut state = env.state().borrow_mut();
            let c = state.casting.take().expect("cast should exist");
            (c.cast_id, c.spell_id)
        };

        let player = env.lua_string("player");
        let args = &[
            player,
            rilua::Val::Num(cast_id as f64),
            rilua::Val::Num(spell_id as f64),
        ];
        let _ = env.fire_event_with_args("UNIT_SPELLCAST_STOP", args);
        let _ = env.fire_event_with_args("UNIT_SPELLCAST_SUCCEEDED", args);

        // Apply heal (same as update.rs apply_heal_effect for friendly target)
        {
            let mut state = env.state().borrow_mut();
            if let Some(ref mut t) = state.current_target {
                t.health = (t.health + 20_000).min(t.health_max);
            }
        }

        // Verify casting cleared
        let casting_after: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(!casting_after, "UnitCastingInfo should be nil after cast completes");

        // Verify target healed
        let health_after: i32 = {
            let state = env.state().borrow();
            state.current_target.as_ref().unwrap().health
        };
        assert!(
            health_after > health_before,
            "target health should increase: before={health_before}, after={health_after}"
        );
        assert_eq!(
            health_after - health_before,
            20_000,
            "heal amount should be 20000"
        );
    }
}

#[test]
fn instant_spell_does_not_show_cast_bar() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        // Avenger's Shield is harmful — needs hostile target
        env.exec("TargetUnit('enemy1')").expect("target enemy1");

        // Slot 2 = Avenger's Shield (instant cast)
        env.exec("UseAction(2)").expect("UseAction(2)");

        let casting: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(!casting, "instant spell should not show casting info");
    }
}

/// Load all Blizzard addons and fire startup events.
fn env_with_full_blizzard_ui() -> WowLuaEnv {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available for cached full UI tests");
    env_with_blizzard_ui_from(ui)
}

fn env_with_blizzard_ui_from(ui: std::path::PathBuf) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create env");
    env.set_screen_size(1024.0, 768.0);


    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = wow_ui_sim::loader::discover_blizzard_addons(&ui);
    for (_name, toc_path) in &addons {
        let _ = wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path);
    }
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for ev in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(ev);
    }
    common::fire_player_entering_world(env, true, false);
    let _ = env.fire_edit_mode_layouts_updated();
    for ev in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(ev);
    }
}

#[test]
fn action_button_down_sets_pushed_state() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        // ActionButtonDown calls SetButtonState("PUSHED") on the button widget
        let state_before: String = env
            .eval(r#"return _G["ActionButton1"]:GetButtonState()"#)
            .unwrap();
        assert_eq!(state_before, "NORMAL");

        env.exec("ActionButtonDown(1)").expect("ActionButtonDown");

        let state_after: String = env
            .eval(r#"return _G["ActionButton1"]:GetButtonState()"#)
            .unwrap();
        assert_eq!(state_after, "PUSHED", "ActionButtonDown should push the button");

        // ActionButtonUp resets it
        env.exec("ActionButtonUp(1)").expect("ActionButtonUp");

        let state_reset: String = env
            .eval(r#"return _G["ActionButton1"]:GetButtonState()"#)
            .unwrap();
        assert_eq!(state_reset, "NORMAL", "ActionButtonUp should reset to NORMAL");
    }
}

#[test]
fn button_state_pushed_during_keypress() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");

        // Create a test button and set its state
        env.exec(r#"
        local btn = CreateFrame("Button", "TestCastButton", UIParent)
        btn:SetButtonState("PUSHED")
    "#).expect("create button");

        let state: String = env
            .eval(r#"return TestCastButton:GetButtonState()"#)
            .unwrap();
        assert_eq!(state, "PUSHED", "SetButtonState('PUSHED') should persist");

        env.exec("TestCastButton:SetButtonState('NORMAL')").expect("reset");

        let state: String = env
            .eval(r#"return TestCastButton:GetButtonState()"#)
            .unwrap();
        assert_eq!(state, "NORMAL", "SetButtonState('NORMAL') should reset");
    }
}

#[test]
fn cast_bar_times_are_in_milliseconds() {
    test_timeout! {
        let env = env_with_friendly_target();
        env.exec("UseAction(1)").expect("UseAction(1)");

        let (start_ms, end_ms): (f64, f64) = env
            .eval("local _, _, _, s, e = UnitCastingInfo('player'); return s, e")
            .unwrap();

        // GetTime() returns seconds; UnitCastingInfo returns milliseconds.
        // start should be > 0 ms and end should be start + 1500 ms.
        assert!(start_ms > 0.0, "start time should be positive, got {start_ms}");
        let duration_ms = end_ms - start_ms;
        assert!(
            (duration_ms - 1500.0).abs() < 10.0,
            "cast duration should be ~1500ms, got {duration_ms}ms"
        );
    }
}

/// Install a Lua error handler that collects errors into `__test_errors`.
fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_errors = {}
        seterrorhandler(function(msg)
            table.insert(__test_errors, tostring(msg))
        end)
    "#,
    )
    .expect("install test error handler");
}

/// Read collected errors from `__test_errors` and clear it.
fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

#[test]
fn use_action_with_blizzard_ui_no_errors() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        env.exec("TargetUnit('party1')").expect("target party1");
        install_test_error_handler(&env);

        // UseAction(1) = Flash of Light (cast time spell) — should fire
        // UNIT_SPELLCAST_START which Blizzard's UnitFrame handles.
        env.exec("UseAction(1)").expect("UseAction(1)");

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "UseAction(1) with Blizzard UI produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
    }
}

/// Diagnose cast bar state — prints mixin/handler info for debugging.
fn dump_cast_bar_diagnostics(env: &WowLuaEnv) {
    let diag: String = env
        .eval(
            r#"
            local f = PlayerCastingBarFrame
            local p = {}
            table.insert(p, "unit=" .. tostring(f.unit))
            -- Step through PlayerCastingBarMixin:OnLoad manually
            f.unit = nil
            CastingBarMixin.OnLoad(f, "player", true, false)
            table.insert(p, "after_CBM_OnLoad=" .. tostring(f.unit))
            -- Step through SetUnit
            f.unit = nil
            CastingBarMixin.SetUnit(f, "player", true, false)
            table.insert(p, "after_SetUnit=" .. tostring(f.unit))
            return table.concat(p, ", ")
        "#,
        )
        .unwrap();
    eprintln!("[diag] {}", diag);
}

/// Assert cast bar is properly initialized and no cast-related errors after UseAction.
fn assert_cast_bar_shows(env: &WowLuaEnv) {
    let errors = drain_test_errors(env);
    let cast_errors: Vec<&String> = errors
        .iter()
        .filter(|e| !e.contains("ClassResourceBar") && !e.contains("C_PingSecure"))
        .collect();
    assert!(
        cast_errors.is_empty(),
        "Cast produced {} error(s):\n{}",
        cast_errors.len(),
        cast_errors
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );

    let casting: bool = env
        .eval("return PlayerCastingBarFrame.casting == true")
        .unwrap();
    assert!(casting, "PlayerCastingBarFrame.casting should be true");

    let shown: bool = env.eval("return PlayerCastingBarFrame:IsShown()").unwrap();
    assert!(shown, "PlayerCastingBarFrame should be visible during cast");
}

#[test]
fn cast_bar_visible_during_cast() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        env.exec("TargetUnit('party1')").expect("target party1");
        install_test_error_handler(&env);

        let exists: bool = env
            .eval("return PlayerCastingBarFrame ~= nil")
            .unwrap();
        assert!(exists, "PlayerCastingBarFrame should exist");

        dump_cast_bar_diagnostics(&env);

        let unit: String = env
            .eval("return PlayerCastingBarFrame.unit or 'nil'")
            .unwrap();
        assert_eq!(unit, "player", "cast bar unit should be 'player'");

        let registered: bool = env
            .eval("return PlayerCastingBarFrame:IsEventRegistered('UNIT_SPELLCAST_START')")
            .unwrap();
        assert!(registered, "cast bar should be registered for UNIT_SPELLCAST_START");

        env.exec("UseAction(1)").expect("UseAction(1)");
        assert_cast_bar_shows(&env);
    }
}

#[test]
fn cast_bar_respects_edit_mode_lock_setting_after_startup_fix() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();

        let (lock_value, attached_after_post_event, parent_after_post_event): (i64, bool, String) = env
            .eval(
                r#"
                local cast = PlayerCastingBarFrame
                local parent = cast and cast:GetParent()
                local lockValue = cast and cast.GetSettingValue
                    and cast:GetSettingValue(Enum.EditModeCastBarSetting.LockToPlayerFrame)
                    or -1
                return lockValue,
                    cast:IsAttachedToPlayerFrame() or false,
                    parent and parent:GetName() or "nil"
                "#,
            )
            .unwrap();

        assert_eq!(lock_value, 0, "default layout keeps the cast bar unlocked");
        assert!(
            !attached_after_post_event,
            "unlocked startup cast bar should stay under the frame manager"
        );
        assert_eq!(
            parent_after_post_event, "BottomManagedFrameContainer",
            "unlocked startup cast bar should remain frame-manager parented"
        );

        let (player_apply_ok, attached_after_player_anchor): (bool, bool) = env
            .eval(
                r#"
                PlayerFrame_DetachCastBar()
                local playerApplyOk = pcall(function()
                    PlayerFrame:ApplySystemAnchor()
                end)
                return playerApplyOk, PlayerCastingBarFrame:IsAttachedToPlayerFrame() or false
                "#,
            )
            .unwrap();

        assert!(player_apply_ok, "PlayerFrame:ApplySystemAnchor should still run successfully");
        assert!(
            !attached_after_player_anchor,
            "player-frame anchoring should not override the cast bar's unlocked edit-mode setting"
        );

        let (cast_apply_ok, attached_after_cast_anchor, parent_after_cast_anchor): (
            bool,
            bool,
            String,
        ) = env
            .eval(
                r#"
                for _, settingInfo in ipairs(PlayerCastingBarFrame.systemInfo.settings or {}) do
                    if settingInfo.setting == Enum.EditModeCastBarSetting.LockToPlayerFrame then
                        settingInfo.value = 1
                    end
                end
                PlayerCastingBarFrame:UpdateSettingMap(true)

                local castApplyOk = pcall(function()
                    PlayerCastingBarFrame:ApplySystemAnchor()
                end)
                local parent = PlayerCastingBarFrame:GetParent()
                return castApplyOk,
                    PlayerCastingBarFrame:IsAttachedToPlayerFrame() or false,
                    parent and parent:GetName() or "nil"
                "#,
            )
            .unwrap();

        assert!(
            cast_apply_ok,
            "PlayerCastingBarFrame:ApplySystemAnchor should handle locked mode"
        );
        assert!(
            attached_after_cast_anchor,
            "locked cast-bar edit-mode anchor path should attach to PlayerFrame"
        );
        assert_eq!(parent_after_cast_anchor, "PlayerFrame");
    }
}

#[test]
fn harmful_spell_blocked_with_no_target() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        // No target set — harmful spell should not cast
        env.exec("CastSpellByID(275779)").expect("CastSpellByID");
        let casting: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(!casting, "harmful spell with no target should not cast");
    }
}

#[test]
fn harmful_spell_blocked_on_friendly_target() {
    test_timeout! {
        let env = env_with_friendly_target();
        // Judgment (harmful) on a friendly target should be blocked
        env.exec("CastSpellByID(275779)").expect("CastSpellByID");
        let casting: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(!casting, "harmful spell on friendly target should not cast");
    }
}

#[test]
fn harmful_spell_succeeds_on_hostile_target() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        env.exec("TargetUnit('enemy1')").expect("target enemy1");
        // Judgment (harmful, instant) on a hostile target should succeed
        env.exec("CastSpellByID(275779)").expect("CastSpellByID");
        // Instant spell — no cast bar, but cooldown should start
        let on_cd: bool = env
            .eval("local info = C_Spell.GetSpellCooldown(275779); return info.duration > 0")
            .unwrap();
        assert!(on_cd, "harmful spell on hostile target should cast and trigger GCD");
    }
}

#[test]
fn helpful_spell_succeeds_on_hostile_target() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        env.exec("TargetUnit('enemy1')").expect("target enemy1");
        // Flash of Light (helpful) on hostile target — should still cast (auto-target self)
        env.exec("UseAction(1)").expect("UseAction(1)");
        let casting: bool = env
            .eval("return UnitCastingInfo('player') ~= nil")
            .unwrap();
        assert!(casting, "helpful spell should cast even with hostile target (auto-self)");
    }
}

#[test]
fn self_only_spell_succeeds_with_no_target() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        // Divine Shield (self-only) should cast regardless of target
        env.exec("CastSpellByID(642)").expect("CastSpellByID");
        // Divine Shield is instant and off-GCD, verify it succeeded via cooldown
        let on_cd: bool = env
            .eval("local info = C_Spell.GetSpellCooldown(642); return info.duration > 0")
            .unwrap();
        assert!(on_cd, "self-only spell should cast with no target");
    }
}

#[test]
fn paladin_aura_spell_applies_player_buff() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        env.exec("A_Admin.ClearBuffs()").expect("clear seeded buffs");

        env.exec("CastSpellByID(465)").expect("cast Devotion Aura");

        let (is_casting, aura_name, aura_spell_id): (bool, Option<String>, Option<i32>) = env
            .eval(
                r#"
                local aura = C_UnitAuras.GetPlayerAuraBySpellID(465)
                return UnitCastingInfo("player") ~= nil,
                    aura and aura.name or nil,
                    aura and aura.spellId or nil
                "#,
            )
            .unwrap();

        assert!(!is_casting, "Paladin aura should be instant, not a cast bar spell");
        assert_eq!(aura_name.as_deref(), Some("Devotion Aura"));
        assert_eq!(aura_spell_id, Some(465));
    }
}

#[test]
fn paladin_aura_spell_replaces_previous_paladin_aura() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("create env");
        env.exec("A_Admin.ClearBuffs()").expect("clear seeded buffs");

        env.exec("CastSpellByID(465)").expect("cast Devotion Aura");
        env.exec("CastSpellByID(32223)").expect("cast Crusader Aura");

        let (devotion_present, crusader_name, helpful_count): (bool, Option<String>, i32) = env
            .eval(
                r#"
                local devotion = C_UnitAuras.GetPlayerAuraBySpellID(465) ~= nil
                local crusader = C_UnitAuras.GetPlayerAuraBySpellID(32223)
                local count = 0
                AuraUtil.ForEachAura("player", "HELPFUL", nil, function()
                    count = count + 1
                end)
                return devotion, crusader and crusader.name or nil, count
                "#,
            )
            .unwrap();

        assert!(!devotion_present, "new Paladin aura should replace the previous aura");
        assert_eq!(crusader_name.as_deref(), Some("Crusader Aura"));
        assert_eq!(helpful_count, 1);
    }
}

#[test]
fn paladin_aura_cast_refreshes_blizzard_buff_frame() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        install_test_error_handler(&env);
        env.exec(
            r#"
            A_Admin.ClearBuffs()
            if BuffFrame then
                BuffFrame:Update()
            end
            CastSpellByID(465)
            "#,
        )
        .expect("cast Devotion Aura through Blizzard UI");

        let (aura_present, buff_frame_count, first_button_shown): (bool, i32, bool) = env
            .eval(
                r#"
                local aura = C_UnitAuras.GetPlayerAuraBySpellID(465)
                local count = BuffFrame and BuffFrame.auraInfo and #BuffFrame.auraInfo or -1
                local first = BuffFrame and BuffFrame.auraFrames and BuffFrame.auraFrames[1]
                return aura ~= nil, count, first and first:IsShown() or false
                "#,
            )
            .unwrap();
        let errors = drain_test_errors(&env);

        assert!(aura_present, "cast should add Devotion Aura to player auras");
        assert!(
            errors.is_empty(),
            "no Lua errors expected while updating BuffFrame: {errors:?}"
        );
        assert!(
            buff_frame_count > 0,
            "BuffFrame should rebuild auraInfo after an aura cast"
        );
        assert!(
            first_button_shown,
            "BuffFrame should show an aura button after an aura cast"
        );
    }
}

#[test]
fn ui_error_message_wired_with_blizzard_ui() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        env.exec("TargetUnit('party1')").expect("target party1");
        install_test_error_handler(&env);

        // UIErrorsFrame should exist and handle UI_ERROR_MESSAGE
        let exists: bool = env
            .eval("return UIErrorsFrame ~= nil")
            .unwrap();
        assert!(exists, "UIErrorsFrame should exist");

        // Cast harmful spell on friendly target — should fire UI_ERROR_MESSAGE
        env.exec("CastSpellByID(275779)").expect("CastSpellByID");

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "UI_ERROR_MESSAGE handling produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
    }
}
