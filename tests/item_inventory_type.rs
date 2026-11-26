//! ItemMixin:GetInventoryType reads C_Item.GetItemInventoryType(ByID).
//! Retail and Forever ItemDocumentation.lua declare both functions.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;

use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_object_api() -> WowLuaEnv {
    let env = common::env_with_shared_xml();
    let ui = common::blizzard_ui_dir();
    for addon in ["Blizzard_Colors", "Blizzard_ObjectAPI"] {
        let toc = find_toc_file(&ui.join(addon)).expect("Blizzard addon TOC should exist");
        load_addon(&env.loader_env(), &toc).expect("Blizzard addon should load");
    }
    env
}

// 212440 Devotee's Discarded Headdress (InventoryType 1) and 268208
// Strongblood's Ceremonial Cleaver (InventoryType 13) come from the item
// catalog; 1999999999 is absent from it.
const INVENTORY_TYPE_READS: &str = r#"
    local head = Enum.InventoryType.IndexHeadType
    local weapon = Enum.InventoryType.IndexWeaponType
    assert(head ~= weapon)
    local function check(actual, expected, label)
        assert(actual == expected, label .. ": " .. tostring(actual))
    end

    check(Item:CreateFromItemID(212440):GetInventoryType(), head, "head by ID")
    check(Item:CreateFromItemID(268208):GetInventoryType(), weapon, "weapon by ID")

    A_Admin.ClearBags()
    A_Admin.AddBagItem(0, 1, 268208, 1)
    A_Admin.AddBagItem(0, 3, 212440, 1)
    check(Item:CreateFromBagAndSlot(0, 1):GetInventoryType(), weapon, "weapon in bag")
    check(Item:CreateFromBagAndSlot(0, 3):GetInventoryType(), head, "head in bag")

    check(Item:CreateFromBagAndSlot(0, 2):GetInventoryType(), nil, "empty slot")
    check(C_Item.GetItemInventoryType(ItemLocation:CreateFromBagAndSlot(0, 2)), nil,
        "empty location")
    check(Item:CreateFromItemID(1999999999):GetInventoryType(), nil, "unknown ID")
"#;

#[test]
fn item_mixin_inventory_type_reads_catalog_and_bag_locations() {
    let env = env_with_object_api();
    env.exec(INVENTORY_TYPE_READS).unwrap();
}
