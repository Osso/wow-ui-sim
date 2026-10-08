//! Classic namespace lookup is unaffected by retail retirement.
#![cfg(feature = "client-mists")]

#[test]
fn patch_9_0_5_mists_preserves_legacy_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"assert(type(C_Soulbinds.GetConduitItemLevel) == "function")"#).unwrap();
}
