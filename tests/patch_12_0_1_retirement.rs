//! 12.0.1 removals apply from the first supported later retail epoch.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_retirement_blocks_raw_and_ordinary_lookup() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        for attempt = 1, 2 do
            for _, name in ipairs({'GetSpeakerVolume', 'SetSpeakerVolume'}) do
                assert(rawget(C_CombatAudioAlert, name) == nil)
                assert(C_CombatAudioAlert[name] == nil)
            end
            for _, name in ipairs({'GetTargetClampingInsets', 'SetTargetClampingInsets'}) do
                assert(rawget(C_NamePlate, name) == nil)
                assert(C_NamePlate[name] == nil)
            end
        end
        assert(C_CombatAudioAlert.SetSpeakerSpeed(2) == true)
        assert(C_CombatAudioAlert.GetSpeakerSpeed() == 2)
        assert(C_CombatAudioAlert.SetFormatSetting(0, 1, 3) == true)
        assert(C_CombatAudioAlert.GetFormatSetting(0, 1) == 3)
        "#,
    )
    .expect("retired members stay absent without breaking live audio settings");
}
