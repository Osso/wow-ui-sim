//! API-level tests for WM42 derived item-specialization records.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn derived_item_specializations_use_overrides_rules_and_level_bands() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.level = 80;
    env.exec(
        r#"
        assert(C_Item.DoesItemContainSpec(80773, 2, 65))
        assert(not C_Item.DoesItemContainSpec(80773, 2, 66))
        assert(C_Item.DoesItemContainSpec(892, 8, 62))
        assert(not C_Item.DoesItemContainSpec(892, 8, 65))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().player.level = 110;
    env.exec("assert(C_Item.DoesItemContainSpec(892, 8, 62))")
        .unwrap();
    env.state().borrow_mut().player.level = 124;
    env.exec("assert(C_Item.DoesItemContainSpec(892, 8, 62))")
        .unwrap();

    let fresh = WowLuaEnv::new().unwrap();
    fresh.state().borrow_mut().player.level = 124;
    fresh
        .exec("assert(not C_Item.DoesItemContainSpec(892, 8, 62))")
        .unwrap();
}
