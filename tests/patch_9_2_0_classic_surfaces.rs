//! Retail 9.2.0 retirement must not alter classic lazy namespace lookup.
#![cfg(feature = "client-mists")]

#[test]
fn patch_9_2_0_mists_preserves_legacy_pvp_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
for _, name in ipairs({'GetSpecialEventDetails', 'GetSpecialEventInfo'}) do
    assert(type(C_PvP[name]) == 'function', name)
    assert(type(C_PvP[name]) == 'function', name)
end
"#,
    )
    .unwrap();
}
