//! Enum publication only; no player-data or logging behavior.

use crate::lua_api::WowLuaEnv;

fn assert_player_data_flags(env: &WowLuaEnv, ptr: bool) {
    env.exec(&format!(
        r#"
        for _, name in ipairs({{"PlayerDataElementAccountFlags", "PlayerDataElementCharacterFlags"}}) do
            local values = Enum[name]
            local meta = Enum[name .. "Meta"]
            if {ptr} then
                assert(type(values) == "table", name .. " missing")
                assert(values.Log == 1, name .. ".Log")
                local count = 0
                for key, value in pairs(values) do
                    assert(key == "Log" and value == 1, name .. " unexpected member")
                    count = count + 1
                end
                assert(count == 1, name .. " member count")
                assert(meta.MinValue == 1 and meta.MaxValue == 1 and meta.NumValues == 1,
                    name .. " metadata")
            else
                assert(values == nil, name .. " must be absent")
                assert(meta == nil, name .. " metadata must be absent")
            end
        end
        "#
    ))
    .expect("player-data flag publication");
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_player_data_flag_enums_ptr() {
    let env = WowLuaEnv::new().unwrap();
    assert_player_data_flags(&env, true);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_player_data_flags(&env, true);
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_player_data_flag_enums_retail() {
    let env = WowLuaEnv::new().unwrap();
    assert_player_data_flags(&env, false);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_player_data_flags(&env, false);
}
