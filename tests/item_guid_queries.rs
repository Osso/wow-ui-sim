//! API-level coverage for C_Item GUID queries (ItemDocumentation.lua:544/:749/
//! :764/:1498 Retail, :566/:771/:786/:1535 Forever) and, on Retail, the item
//! lock requests (:1621/:1631/:1683/:1693).
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const SETUP: &str = r#"
    A_Admin.ClearBags()
    A_Admin.AddBagItem(0, 1, 6948, 1)
    A_Admin.AddBagItem(0, 2, 159, 1)
    HearthGUID = C_Item.GetItemGUID({ bagID = 0, slotIndex = 1 })
    WaterGUID = C_Item.GetItemGUID({ bagID = 0, slotIndex = 2 })
"#;

const GUID_QUERIES: &str = r#"
    assert(C_Item.GetItemIDByGUID(HearthGUID) == 6948)
    assert(C_Item.GetItemIDByGUID(WaterGUID) == 159)
    assert(C_Item.GetItemLinkByGUID(HearthGUID):find("Hitem:6948:", 1, true))
    assert(C_Item.GetItemLinkByGUID(WaterGUID):find("Hitem:159:", 1, true))
    assert(C_Item.IsItemGUIDInInventory(WaterGUID) == true)
    local location = C_Item.GetItemLocation(WaterGUID)
    assert(location.bagID == 0 and location.slotIndex == 2)
    A_Admin.RemoveBagItem(0, 1)
    assert(C_Item.IsItemGUIDInInventory(HearthGUID) == false)
    assert(C_Item.GetItemIDByGUID(HearthGUID) == nil)
    assert(C_Item.GetItemLinkByGUID(HearthGUID) == nil)
    assert(C_Item.GetItemLocation(HearthGUID) == nil)
    A_Admin.AddBagItem(0, 1, 6948, 1)
"#;

const LOCK_REQUESTS: &str = r#"
    local hearth, water = { bagID = 0, slotIndex = 1 }, { bagID = 0, slotIndex = 2 }
    assert(C_Item.IsLocked(hearth) == false)
    C_Item.LockItem(hearth)
    assert(C_Item.IsLocked(hearth) == true)
    assert(C_Item.IsLocked(water) == false)
    assert(C_Container.GetContainerItemInfo(0, 1).isLocked == true)
    C_Item.UnlockItem(hearth)
    assert(C_Item.IsLocked(hearth) == false)
    C_Item.LockItemByGUID(WaterGUID)
    assert(C_Item.IsLocked(water) == true)
    assert(C_Item.IsLocked(hearth) == false)
    C_Item.UnlockItemByGUID(WaterGUID)
    assert(C_Item.IsLocked(water) == false)
"#;

#[test]
fn item_guid_queries_resolve_the_bag_slot_that_holds_the_item() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(SETUP).expect("bag fixture");
    env.exec(GUID_QUERIES).expect("GUID queries");
    if ACTIVE == ClientProfile::Retail {
        env.exec(LOCK_REQUESTS).expect("lock requests");
    }
}
