//! Concrete behavioral proof for the bounded 12.0.0 mixed publication closures.
#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_mixed_string_util_integer_formatting() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local cases = {
            {12.9, '12', '13'}, {12.5, '12', '13'}, {12.49, '12', '12'},
            {-12.9, '-13', '-13'}, {-12.5, '-13', '-12'}, {-0.1, '-1', '0'},
            {0, '0', '0'}, {1000000.1, '1000000', '1000000'},
        }
        for _, c in ipairs(cases) do
            assert(C_StringUtil.FloorToNearestString(c[1]) == c[2])
            assert(C_StringUtil.RoundToNearestString(c[1]) == c[3])
        end
        for _, f in ipairs({C_StringUtil.FloorToNearestString, C_StringUtil.RoundToNearestString}) do
            assert(not pcall(f, '12'))
            assert(not pcall(f, math.huge))
            assert(not pcall(f, 0/0))
        end
    "#).unwrap();
}
