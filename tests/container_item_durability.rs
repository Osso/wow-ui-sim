//! `C_Container.GetContainerItemDurability` (ContainerDocumentation.lua:140,
//! Retail and Forever) reads the durability pair supplied for a bag item.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const DURABILITY: &str = r##"
    A_Admin.ClearBags()
    A_Admin.AddBagItem(0, 1, 268208)
    A_Admin.AddBagItem(1, 3, 268205)
    A_Admin.SetBagItemDurability(0, 1, 40, 80)
    A_Admin.SetBagItemDurability(1, 3, 7, 55)
    local a, b = C_Container.GetContainerItemDurability(0, 1)
    local c, d = C_Container.GetContainerItemDurability(1, 3)
    assert(a == 40 and b == 80, "first pair")
    assert(c == 7 and d == 55, "second pair")
    assert(select("#", C_Container.GetContainerItemDurability(0, 2)) == 0)
    A_Admin.SetBagItemDurability(0, 1, nil, nil)
    assert(select("#", C_Container.GetContainerItemDurability(0, 1)) == 0)
    assert(not pcall(A_Admin.SetBagItemDurability, 0, 2, 1, 2))
    assert(not pcall(A_Admin.SetBagItemDurability, 1, 3, 9, 5))
"##;

#[test]
fn container_item_durability_follows_the_supplied_pair() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(DURABILITY).expect("durability reads follow their inputs");
}
