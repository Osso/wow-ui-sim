//! C_Item.IsEquippedItem reads the supplied equipment. Retail and Forever
//! ItemDocumentation.lua declare byte-identical blocks; no selected Forever
//! FrameXML consumer calls it, so this is API-level coverage.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const EQUIPPED_ITEM_PROBE: &str = r#"
    A_Admin.EquipItem(1, 211993)
    assert(C_Item.IsEquippedItem(211993) == true)
    assert(C_Item.IsEquippedItem('item:211993') == true)
    assert(C_Item.IsEquippedItem(231268) == false)
    assert(C_Item.IsEquippedItem(0) == false)
    A_Admin.EquipItem(16, 231268)
    assert(C_Item.IsEquippedItem(231268) == true)
    A_Admin.EquipItem(1, 221125)
    assert(C_Item.IsEquippedItem(211993) == false)
    assert(C_Item.IsEquippedItem(221125) == true)
    assert(select('#', C_Item.IsEquippedItem(221125)) == 1)
"#;

#[test]
fn is_equipped_item_follows_supplied_equipment() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(EQUIPPED_ITEM_PROBE).unwrap();
}
