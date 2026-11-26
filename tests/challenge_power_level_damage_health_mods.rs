#![cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]

use wow_ui_sim::lua_api::WowLuaEnv;

const POWER_MOD_INPUTS_LUA: &str = r#"
local function capture(...) return {n = select('#', ...), ...} end
assert(capture(C_ChallengeMode.GetPowerLevelDamageHealthMod(11)).n == 0)
A_Admin.SetChallengePowerLevelMods(11, 13, 19)
local mods = capture(C_ChallengeMode.GetPowerLevelDamageHealthMod(11))
assert(mods.n == 2 and mods[1] == 13 and mods[2] == 19)
assert(capture(C_ChallengeMode.GetPowerLevelDamageHealthMod(12)).n == 0)
A_Admin.SetChallengePowerLevelMods(11, nil, 0)
assert(capture(C_ChallengeMode.GetPowerLevelDamageHealthMod(11)).n == 0)
"#;

#[test]
fn challenge_power_level_mods_use_only_supplied_values() {
    let env = WowLuaEnv::new().expect("Retail or Forever Lua environment should initialize");
    let error = env
        .exec("C_ChallengeMode.GetPowerLevelDamageHealthMod()")
        .expect_err("missing powerLevel must raise the documented Usage error");
    assert!(error.to_string().contains(
        "Usage: local damageMod, healthMod = C_ChallengeMode.GetPowerLevelDamageHealthMod(powerLevel)"
    ));
    env.exec(POWER_MOD_INPUTS_LUA)
        .expect("power-level scaling should use supplied rows only");
}
