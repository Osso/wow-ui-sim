#![cfg(feature = "client-ptr")]

use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

const OVERLAY_ENV: &str = "WOW_SIM_CAA_GET_SPEC_SETTING_OVERLAY";
const ADDON: &str = "Blizzard_CombatAudioAlerts";
const AURA_CONTAINER: &[&str] = &["Blizzard_AuraContainer"];

fn install_callable_overlay(env: &WowLuaEnv) {
    let Some(path) = std::env::var_os(OVERLAY_ENV) else {
        return;
    };
    let source = std::fs::read_to_string(path).expect("callable overlay should be readable");
    env.exec(&source).expect("callable overlay should execute");
}

fn load_combat_audio_manager() -> WowLuaEnv {
    let ui = closure::blizzard_ui_dir();
    let env = closure::new_blizzard_addon_env(&ui);
    env.state().borrow().cvars.set_imported("CAAEnabled", "1");
    env.state()
        .borrow()
        .cvars
        .set_imported("CAAPulsePlayerHealthPercent", "0");
    install_callable_overlay(&env);
    let overrides = [BlizzardAddonOverride {
        addon: "Blizzard_UnitFrame",
        extra_roots: AURA_CONTAINER,
    }];
    let loaded = closure::load_blizzard_addon_closure_into_env(&env, &ui, &[ADDON], &overrides);
    assert!(loaded.iter().any(|name| name == ADDON));
    env
}

#[test]
fn manager_registers_unit_target_for_say_if_targeted() {
    let env = load_combat_audio_manager();
    let actual: String = env
        .eval(
            r#"
            local expected = 'Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlertManager.lua'
            local source = debug.getinfo(CombatAudioAlertManagerMixin.RefreshEvents, 'S').source
            assert(source:sub(-#expected) == expected, source)
            local setting = Enum.CombatAudioAlertSpecSetting.SayIfTargeted
            assert(C_CombatAudioAlert.SetSpecSetting(setting, 1))
            CombatAudioAlertManager.initDone = true
            local ok, err = pcall(function() CombatAudioAlertManager:RefreshEvents(false) end)
            return string.format('ok=%s unit_target=%s err=%s', tostring(ok),
                tostring(CombatAudioAlertManager:IsEventRegistered('UNIT_TARGET')), tostring(err))
            "#,
        )
        .expect("original combat-audio manager should run");
    assert_eq!(actual, "ok=true unit_target=true err=nil");
}
