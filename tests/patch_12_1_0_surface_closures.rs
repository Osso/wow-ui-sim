//! Lua-visible behavior for 12.1.0 surface added or retired after the publication sweep.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua environment")
}

#[test]
fn removed_random_training_ground_join_is_not_fabricated() {
    let (raw, lookup): (String, String) = env()
        .eval(
            r#"return type(rawget(C_PvP, "JoinRandomTrainingGround")),
                type(C_PvP.JoinRandomTrainingGround)"#,
        )
        .unwrap();
    assert_eq!((raw.as_str(), lookup.as_str()), ("nil", "nil"));
}
