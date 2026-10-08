//! Removed conduit getter cannot be fabricated by repeated namespace lookup.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
assert(rawget(C_Soulbinds, "GetConduitItemLevel") == nil)
assert(C_Soulbinds.GetConduitItemLevel == nil)
assert(C_Soulbinds.GetConduitItemLevel == nil)
"#;

#[test]
fn patch_9_0_5_unused_member_stays_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
