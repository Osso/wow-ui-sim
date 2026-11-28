#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! IsLeftMetaKeyDown / IsRightMetaKeyDown read the modeled meta modifier like the
//! other sided probes (IsLeftAltKeyDown, IsRightAltKeyDown).

use wow_ui_sim::lua_api::WowLuaEnv;

const META_SIDES: &str = r#"
    assert(IsLeftMetaKeyDown() == false and IsRightMetaKeyDown() == false)
    A_Admin.SetMetaKeyDown(true)
    assert(IsLeftMetaKeyDown() == true and IsRightMetaKeyDown() == true and IsMetaKeyDown())
    assert(IsLeftAltKeyDown() == false, "other modifiers stay independent")
    A_Admin.SetMetaKeyDown(false)
    assert(IsLeftMetaKeyDown() == false and IsRightMetaKeyDown() == false)
"#;

#[test]
fn meta_key_sides_follow_modifier_input() {
    WowLuaEnv::new().unwrap().exec(META_SIDES).unwrap();
}
