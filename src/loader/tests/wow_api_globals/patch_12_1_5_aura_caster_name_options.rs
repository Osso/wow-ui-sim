//! Ordinary option processing only; no rendering or security claims.

use crate::lua_api::WowLuaEnv;

#[cfg(feature = "retail-12-1-5")]
#[test]
fn patch_12_1_5_aura_caster_name_options_defaults_and_booleans() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.eval::<()>(
        r#"
        local process = C_AuraContainerUtil.ProcessCustomAuraButtonCasterNameOptions
        assert(type(process) == "function", "caster-name processor missing")
        local function check(options, realm, colors)
            local result = process(options)
            assert(result.showRealmName == realm)
            assert(result.useClassColors == colors)
        end
        check(nil, false, false)
        check({}, false, false)
        local omitted = process()
        assert(omitted.showRealmName == false and omitted.useClassColors == false)
        check({showRealmName = true}, true, false)
        check({useClassColors = true}, false, true)
        for _, realm in ipairs({false, true}) do
            for _, colors in ipairs({false, true}) do
                check({showRealmName = realm, useClassColors = colors}, realm, colors)
            end
        end
        "#,
    )
    .expect("caster-name defaults and explicit booleans should normalize");
}

#[cfg(not(feature = "retail-12-1-5"))]
#[test]
fn patch_12_1_5_aura_caster_name_options_absent_on_earlier_profiles() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let absent: bool = env
        .eval("return C_AuraContainerUtil.ProcessCustomAuraButtonCasterNameOptions == nil")
        .expect("earlier-profile namespace should be readable");
    assert!(absent);
}
