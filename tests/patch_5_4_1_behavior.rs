#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_5_4_1_default_realm_cvar_is_unavailable(env: &WowLuaEnv) {
    let unavailable: bool = env
        .eval(
            r#"
            return GetCVar("realmName") == nil
                and C_CVar.GetCVar("realmName") == nil
            "#,
        )
        .expect("query default game-state realm CVar");
    assert!(unavailable, "realmName must not expose a default game-state CVar");
}
}
