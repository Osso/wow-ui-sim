//! `C_Item.IsItemConduit(itemLoc)` (ItemDocumentation.lua:1392 Retail, :1430
//! Forever) reads the supplied conduit classification of the item at a bag or
//! equipment location.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const CONDUIT_LOCATIONS: &str = r#"
    A_Admin.ClearBags()
    A_Admin.AddBagItem(0, 1, 6948, 1)
    A_Admin.AddBagItem(0, 2, 159, 1)
    local hearth, water = { bagID = 0, slotIndex = 1 }, { bagID = 0, slotIndex = 2 }
    A_Admin.SetItemOverlayFlag(159, "conduit", true)
    assert(C_Item.IsItemConduit(water) == true)
    assert(C_Item.IsItemConduit(hearth) == false)
    A_Admin.SetItemOverlayFlag(159, "conduit", false)
    A_Admin.SetItemOverlayFlag(6948, "conduit", true)
    assert(C_Item.IsItemConduit(water) == false)
    assert(C_Item.IsItemConduit(hearth) == true)
    assert(C_Item.IsItemConduit({ bagID = 0, slotIndex = 3 }) == false)
"#;

#[test]
fn item_conduit_follows_the_item_at_the_location() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(CONDUIT_LOCATIONS)
        .expect("conduit classification follows the location");
}
