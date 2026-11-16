#![cfg(feature = "client-mists")]

use std::collections::HashMap;

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

const VISIBILITY_USAGE: &str = "Usage: local hasCustom, alwaysShowMine, showForMySpec = C_Spell.GetVisibilityInfo(spellID, visibilityType)";

fn load_mists_consumer(addon: &str) -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::Mists);
    let ui = closure::blizzard_ui_dir();
    let (env, loaded) = closure::build_blizzard_addon_closure_env(&ui, &[addon], &[]);
    assert!(loaded.iter().any(|name| name == addon), "{addon} must load");
    env
}

fn read_lua_error(env: &WowLuaEnv, call: &str) -> String {
    env.eval(&format!(
        "local ok, err = pcall(function() {call} end); assert(not ok); return err"
    ))
    .expect("pcall should return the Lua error")
}

#[test]
fn combat_audio_manager_consumer_calls_original_is_enabled() {
    let env = load_mists_consumer("Blizzard_CombatAudioAlerts");
    let selected: bool = env
        .eval(
            "local source = debug.getinfo(CombatAudioAlertManagerMixin.RefreshEvents, 'S').source; return source:sub(-#'Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlertManager.lua') == 'Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlertManager.lua'",
        )
        .expect("Mists manager consumer should be loaded");
    assert!(selected, "expected the original Mists combat-audio manager");
    env.exec(
        "CombatAudioAlertManager.initDone = true; CombatAudioAlertManager:RefreshEvents(false)",
    )
    .expect("original RefreshEvents must call C_CombatAudioAlert.IsEnabled without an error");
}

#[test]
fn combat_audio_enabled_returns_one_boolean_for_default_and_cvar_inputs() {
    for (input, expected) in [(None, false), (Some("0"), false), (Some("1"), true)] {
        let env = WowLuaEnv::new().expect("Mists C API should initialize");
        if let Some(value) = input {
            env.state().borrow().cvars.set_imported("CAAEnabled", value);
        }
        assert_eq!(
            env.state().borrow().cvars.get("CAAEnabled").as_deref(),
            Some(input.unwrap_or("0"))
        );
        let actual: (i64, String, bool) = env
            .eval(
                "local count = select('#', C_CombatAudioAlert.IsEnabled()); local value = C_CombatAudioAlert.IsEnabled(); return count, type(value), value",
            )
            .expect("IsEnabled should return one boolean");
        assert_eq!(actual, (1, "boolean".into(), expected));
    }
}

#[test]
fn aura_util_consumer_uses_visibility_rows_and_unknown_spell_miss() {
    let env = load_mists_consumer("Blizzard_FrameXMLUtil");
    {
        let mut sim = env.state().borrow_mut();
        sim.player.present = true;
        sim.player.in_combat = false;
        sim.spell_visibility_results = Some(HashMap::from([
            ((910_001, 1), (true, true, false)),
            ((910_002, 1), (true, true, false)),
            ((910_003, 2), (true, false, true)),
            ((910_004, 0), (true, false, false)),
        ]));
    }
    env.exec(
        r#"
        local source = debug.getinfo(AuraUtil.ShouldDisplayDebuff, 'S').source
        assert(source:sub(-#'Blizzard_FrameXMLUtil/AuraUtil.lua') == 'Blizzard_FrameXMLUtil/AuraUtil.lua', source)
        assert(AuraUtil.ShouldDisplayDebuff('player', 910001))
        assert(not AuraUtil.ShouldDisplayDebuff('party1', 910002))
        assert(AuraUtil.ShouldDisplayDebuff('party1', 910003))
        assert(AuraUtil.ShouldDisplayDebuff('party1', 999999999))
        assert(Enum.SpellAuraVisibilityType.RaidInCombat == 0)
        assert(Enum.SpellAuraVisibilityType.RaidOutOfCombat == 1)
        assert(Enum.SpellAuraVisibilityType.EnemyTarget == 2)
        "#,
    )
    .expect("original AuraUtil consumer should use the native visibility query");
    assert_visibility_input_conversion(&env);
    assert_visibility_misses(&env);
}

fn assert_visibility_input_conversion(env: &WowLuaEnv) {
    let values: (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "local a, b, c = C_Spell.GetVisibilityInfo('910001', '257', 'extra'); local d, e, f = C_Spell.GetVisibilityInfo(910003.9, '258.9'); local g, h, i = C_Spell.GetVisibilityInfo('910004', 256); local j, k, l = C_Spell.GetVisibilityInfo('910004', -256); return a, b, c, d, e, f, g, h, i, j, k, l",
        )
        .expect("numeric strings, truncation, modulo, and extra arguments should work");
    assert_eq!(
        values,
        (
            true, true, false, true, false, true, true, false, false, true, false, false,
        )
    );
}

fn assert_visibility_misses(env: &WowLuaEnv) {
    let missing: (i64, bool, bool, bool) = env
        .eval(
            "local count = select('#', C_Spell.GetVisibilityInfo(999999999, 1)); local a, b, c = C_Spell.GetVisibilityInfo(999999999, 1); return count, a, b, c",
        )
        .expect("unknown spell should publish three false values");
    assert_eq!(missing, (3, false, false, false));
    env.state().borrow_mut().player.present = false;
    let count: i64 = env
        .eval("return select('#', C_Spell.GetVisibilityInfo(910001, 1))")
        .expect("absent player should return no values");
    assert_eq!(count, 0);
}

#[test]
fn visibility_info_errors_match_mists_contract() {
    let env = WowLuaEnv::new().expect("Mists C API should initialize");
    let usage = VISIBILITY_USAGE;
    for (call, expected) in [
        (
            "C_Spell.GetVisibilityInfo()",
            format!("bad argument #1 to 'C_Spell.GetVisibilityInfo' ({usage})"),
        ),
        (
            "C_Spell.GetVisibilityInfo(1, nil)",
            format!("bad argument #2 to 'C_Spell.GetVisibilityInfo' ({usage})"),
        ),
        (
            "C_Spell.GetVisibilityInfo(1, 'invalid')",
            format!("bad argument #2 to 'C_Spell.GetVisibilityInfo' ({usage})"),
        ),
        (
            "C_Spell.GetVisibilityInfo(1, 3)",
            format!("bad argument #2 to 'C_Spell.GetVisibilityInfo' ({usage})"),
        ),
        (
            "C_Spell.GetVisibilityInfo(2147483648, 0)",
            format!("bad argument #1 to 'C_Spell.GetVisibilityInfo' (outside of expected range -2147483648 to 2147483647 - {usage})"),
        ),
        (
            "C_Spell.GetVisibilityInfo(1, 2147483648)",
            "bad argument #2 to 'C_Spell.GetVisibilityInfo' (integer overflow attempting to store 2147483648.000000)".into(),
        ),
    ] {
        assert_eq!(read_lua_error(&env, call), expected, "{call}");
    }
}
