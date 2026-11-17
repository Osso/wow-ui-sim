#![cfg(feature = "client-mists")]

use crate::common::lua_error_assertions::assert_lua_error_contains;
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

const GET_USAGE: &str = "Usage: local value = C_CombatAudioAlert.GetSpecSetting(setting)";
const SET_USAGE: &str = "Usage: local success = C_CombatAudioAlert.SetSpecSetting(setting, newVal)";
const MANAGER_SOURCE: &str = "Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlertManager.lua";

/// Spec-setting inputs (Lua), then expected `UNIT_TARGET` (SayIfTargeted > 0)
/// and `UNIT_POWER_UPDATE` (Resource1/2 Percent > 0) registrations.
const REFRESH_CASES: [(&str, bool, bool); 4] = [
    ("", true, false),
    ("return S(8, 0) and S(0, 3)", false, true),
    ("return S(8, 256) and S(4, 5)", true, true),
    ("return S(8, 2) and not S(0, 6)", true, false),
];

fn load_combat_audio_manager() -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::Mists);
    let ui = closure::blizzard_ui_dir();
    let env = closure::new_blizzard_addon_env(&ui);
    env.state().borrow().cvars.set_imported("CAAEnabled", "1");
    let addons = ["Blizzard_CombatAudioAlerts"];
    let loaded = closure::load_blizzard_addon_closure_into_env(&env, &ui, &addons, &[]);
    assert!(loaded.iter().any(|name| name == addons[0]));
    env
}

/// Apply inputs, run the original `RefreshEvents`, and report what it registered.
fn refresh_registrations(env: &WowLuaEnv, inputs: &str) -> String {
    env.eval(&format!(
        r#"
        local source = debug.getinfo(CombatAudioAlertManagerMixin.RefreshEvents, 'S').source
        assert(source:sub(-#'{MANAGER_SOURCE}') == '{MANAGER_SOURCE}', source)
        local S = C_CombatAudioAlert.SetSpecSetting
        local applied = select(2, pcall(function() {inputs} end))
        CombatAudioAlertManager.initDone = true
        local ok, err = pcall(function() CombatAudioAlertManager:RefreshEvents(false) end)
        local frame = CombatAudioAlertManager
        return string.format('applied=%s ok=%s UNIT_TARGET=%s UNIT_POWER_UPDATE=%s err=%s',
            tostring(applied), tostring(ok), tostring(frame:IsEventRegistered('UNIT_TARGET')),
            tostring(frame:IsEventRegistered('UNIT_POWER_UPDATE')), tostring(err))
        "#
    ))
    .expect("original combat-audio manager refresh should be observable")
}

#[test]
fn combat_audio_manager_refresh_follows_spec_settings() {
    let mut observed = Vec::new();
    let mut expected = Vec::new();
    for (inputs, unit_target, unit_power) in REFRESH_CASES {
        let env = load_combat_audio_manager();
        observed.push(refresh_registrations(&env, inputs));
        let applied = if inputs.is_empty() { "nil" } else { "true" };
        expected.push(format!(
            "applied={applied} ok=true UNIT_TARGET={unit_target} UNIT_POWER_UPDATE={unit_power} err=nil"
        ));
    }
    assert_eq!(observed, expected);
}

#[test]
fn spec_settings_default_to_absent_values_and_return_one_number() {
    let env = WowLuaEnv::new().expect("Mists C API should initialize");
    let values: String = env
        .eval(
            "local out = {} for setting = 0, 8 do local count = select('#', C_CombatAudioAlert.GetSpecSetting(setting)) out[#out + 1] = count .. ':' .. C_CombatAudioAlert.GetSpecSetting(setting) end return table.concat(out, ',')",
        )
        .expect("every spec setting should read");
    assert_eq!(values, "1:0,1:1,1:0,1:100,1:0,1:1,1:0,1:100,1:1");
    let converted: (i64, i64, i64) = env
        .eval("local G = C_CombatAudioAlert.GetSpecSetting return G(264), G('3'), G(7.9, 'extra')")
        .expect("low byte, numeric strings and truncation should select settings");
    assert_eq!(converted, (1, 100, 100));
    let published: String = env
        .eval(
            "local E, M = Enum.CombatAudioAlertSpecSetting, Enum.CombatAudioAlertSpecSettingMeta local names = {} for name in pairs(E) do names[#names + 1] = name end table.sort(names, function(a, b) return E[a] < E[b] end) for i, name in ipairs(names) do names[i] = name .. '=' .. E[name] end return table.concat(names, ',') .. ';' .. M.NumValues .. ',' .. M.MinValue .. ',' .. M.MaxValue",
        )
        .expect("spec-setting enum should be published");
    assert_eq!(
        published,
        "Resource1Percent=0,Resource1Format=1,Resource1Voice=2,Resource1Volume=3,Resource2Percent=4,Resource2Format=5,Resource2Voice=6,Resource2Volume=7,SayIfTargeted=8;9,0,8"
    );
}

#[test]
fn set_spec_setting_validates_per_setting_and_stores_full_value() {
    let env = WowLuaEnv::new().expect("Mists C API should initialize");
    let results: String = env
        .eval(
            "local S, G = C_CombatAudioAlert.SetSpecSetting, C_CombatAudioAlert.GetSpecSetting local out = {} local function try(setting, value) out[#out + 1] = tostring(S(setting, value)) .. '/' .. G(setting) end try(0, 5) try(0, 6) try(1, 261) try(3, 101) try(3, -1) try(7, '42') try(2, 0) try(8, 4) try(8, 3) try(8, 256) return table.concat(out, ',')",
        )
        .expect("spec settings should validate and store");
    assert_eq!(
        results,
        "true/5,false/5,true/261,false/100,false/100,true/42,false/0,false/1,true/3,true/256"
    );
    env.state().borrow_mut().mists_combat_audio_voice_count = 2;
    let voices: String = env
        .eval("local S, G = C_CombatAudioAlert.SetSpecSetting, C_CombatAudioAlert.GetSpecSetting return tostring(S(6, 1)) .. '/' .. G(6) .. ',' .. tostring(S(6, 2)) .. '/' .. G(6)")
        .expect("voice settings should use the voice-list length");
    assert_eq!(voices, "true/1,false/1");
}

#[test]
fn unloaded_alert_settings_read_zero_and_reject_writes() {
    let env = WowLuaEnv::new().expect("Mists C API should initialize");
    env.state().borrow_mut().mists_combat_audio_alert_state_gate = false;
    let results: String = env
        .eval(
            "local out = {} for setting = 0, 8 do out[#out + 1] = C_CombatAudioAlert.GetSpecSetting(setting) end out[#out + 1] = tostring(C_CombatAudioAlert.SetSpecSetting(8, 0)) return table.concat(out, ',')",
        )
        .expect("unloaded settings should still answer");
    assert_eq!(results, "0,0,0,0,0,0,0,0,0,false");
}

const SPEC_SETTING_ERROR_CASES: &[(&str, &[&str])] = &[
    ("C_CombatAudioAlert.GetSpecSetting()", &[GET_USAGE]),
    ("C_CombatAudioAlert.GetSpecSetting(9)", &[GET_USAGE]),
    ("C_CombatAudioAlert.GetSpecSetting('x')", &[GET_USAGE]),
    (
        "C_CombatAudioAlert.GetSpecSetting(2147483648)",
        &["integer overflow attempting to store"],
    ),
    ("C_CombatAudioAlert.SetSpecSetting(9, 0)", &[SET_USAGE]),
    ("C_CombatAudioAlert.SetSpecSetting(8)", &[SET_USAGE]),
    (
        "C_CombatAudioAlert.SetSpecSetting(8, 2147483648)",
        &[
            "outside of expected range -2147483648 to 2147483647",
            SET_USAGE,
        ],
    ),
];

#[test]
fn spec_setting_errors_contain_verified_mists_text() {
    let env = WowLuaEnv::new().expect("Mists C API should initialize");
    for &(call, fragments) in SPEC_SETTING_ERROR_CASES {
        assert_lua_error_contains(&env, call, fragments);
    }
}
