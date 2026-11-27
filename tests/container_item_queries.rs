//! Retail 12.1.5 / Forever 1.60.1 ContainerDocumentation.lua
//! `PlayerHasHearthstone` (:430), `GetContainerItemEquipmentSetInfo` (:157),
//! `SetBagPortraitTexture` (:459) and ItemDocumentation.lua
//! `GetItemInventorySlotKey` (:673) over the bag, equipment and set models.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const HEARTHSTONE: &str = r#"
    A_Admin.ClearBags()
    assert(C_Container.PlayerHasHearthstone() == nil)
    A_Admin.AddBagItem(2, 5, 6948, 1)
    assert(C_Container.PlayerHasHearthstone() == 6948)
    A_Admin.RemoveBagItem(2, 5)
    assert(C_Container.PlayerHasHearthstone() == nil)
"#;

const EQUIPMENT_SETS: &str = r#"
    A_Admin.ClearBags()
    local weapon = GetInventoryItemID("player", 16)
    A_Admin.AddBagItem(0, 1, weapon, 1)
    A_Admin.AddBagItem(0, 2, 6948, 1)
    local inSet, list = C_Container.GetContainerItemEquipmentSetInfo(0, 1)
    assert(inSet == false and list == "")
    C_EquipmentSet.CreateEquipmentSet("Raid")
    C_EquipmentSet.SaveEquipmentSet(C_EquipmentSet.GetEquipmentSetID("Raid"))
    C_EquipmentSet.CreateEquipmentSet("Arena")
    C_EquipmentSet.SaveEquipmentSet(C_EquipmentSet.GetEquipmentSetID("Arena"))
    inSet, list = C_Container.GetContainerItemEquipmentSetInfo(0, 1)
    assert(inSet == true and list == "Raid, Arena", list)
    inSet, list = C_Container.GetContainerItemEquipmentSetInfo(0, 2)
    assert(inSet == false and list == "")
"#;

const SLOT_KEYS: &str = r#"
    assert(C_Item.GetItemInventorySlotKey(Enum.InventoryType.IndexHeadType) == "INVTYPE_HEAD")
    assert(C_Item.GetItemInventorySlotKey(Enum.InventoryType.IndexTabardType) == "INVTYPE_TABARD")
    assert(not pcall(C_Item.GetItemInventorySlotKey, 99))
"#;

/// The original ContainerFrameMixin:UpdateMiscellaneousFrames
/// (Blizzard_UIPanels_Game/Mainline/ContainerFrame.lua:822-828) paints bag
/// portraits through `PortraitFrameMixin:SetPortraitToBag`.
const BAG_PORTRAITS: &str = r#"
    SetCVar("combinedBags", "0")
    local slot = C_Container.ContainerIDToInventoryID(1)
    for _, bagItem in ipairs({ 4500, 14156 }) do
        CloseAllBags()
        A_Admin.EquipItem(slot, bagItem)
        ToggleBag(1)
        local frame = ContainerFrameUtil_GetShownFrameForID(1)
        local icon = frame.PortraitContainer.portrait:GetTexture()
        assert(icon == C_Item.GetItemIconByID(bagItem), tostring(icon))
    end
"#;

#[cfg(feature = "client-retail")]
#[test]
fn container_frame_portrait_follows_the_equipped_bag() {
    let env = WowLuaEnv::new().expect("Lua environment");
    common::load_game_startup_inventory(&env);
    env.exec(BAG_PORTRAITS).expect("bag portrait follows the equipped bag");
}

#[test]
fn player_hearthstone_follows_the_bag_contents() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(HEARTHSTONE).expect("hearthstone follows the bags");
}

#[test]
fn container_item_set_info_follows_the_saved_sets() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(EQUIPMENT_SETS).expect("set info follows saved sets");
}

#[test]
fn item_inventory_slot_key_follows_the_inventory_type() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(SLOT_KEYS).expect("slot keys follow the inventory type");
}
