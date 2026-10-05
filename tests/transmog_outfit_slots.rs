#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn transmog_outfit_slots_secondary_and_weapon_options_deliver_changes() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid', 135771)
        local id = api.GetOutfitInfoByName('Raid').outfitID
        api.ChangeViewedOutfit(id)
        local events = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_SECONDARY_SLOTS_CHANGED')
        frame:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_SLOT_WEAPON_OPTION_CHANGED')
        frame:SetScript('OnEvent', function(_, event, slot, option) events[#events+1] = {event,slot,option} end)
        assert(api.SlotHasSecondary(1) and not api.SlotHasSecondary(0))
        assert(not api.GetSecondarySlotState(1))
        api.SetSecondarySlotState(1, true)
        assert(api.GetSecondarySlotState(1) and api.GetSecondarySlotState(2))
        assert(events[1][1] == 'VIEWED_TRANSMOG_OUTFIT_SECONDARY_SLOTS_CHANGED')
        assert(api.IsSlotWeaponSlot(12) and not api.IsSlotWeaponSlot(0))
        api.SetViewedWeaponOptionForSlot(12, 2)
        assert(events[2][2] == 12 and events[2][3] == 2)
        local repeatedEvents = 0
        local refresh = CreateFrame('Frame')
        refresh:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_SLOT_WEAPON_OPTION_CHANGED')
        refresh:SetScript('OnEvent', function(_, _, slot, option)
            repeatedEvents = repeatedEvents + 1
            if repeatedEvents < 3 then api.SetViewedWeaponOptionForSlot(slot,option) end
        end)
        api.SetViewedWeaponOptionForSlot(12, 1)
        assert(repeatedEvents == 1, 'same-option refresh must not recursively redispatch')
        local groups = api.GetSlotGroupInfo()
        local seen = {}
        for _, group in ipairs(groups) do
            for _, slot in ipairs(group.appearanceSlotInfo) do
                seen[slot.slot] = slot
                assert(type(slot.slotName)=='string' and slot.type == 0)
            end
            assert(type(group.illusionSlotInfo)=='table')
        end
        assert(seen[0].collectionType == 1 and seen[2].isSecondary)
        assert(api.GetTransmogOutfitSlotForInventoryType(1) == 0)
        assert(api.GetTransmogOutfitSlotForInventoryType(22) == 13)
        assert(api.GetTransmogOutfitSlotForInventoryType(2) == nil)
        local options = api.GetWeaponOptionsForSlot(0)
        assert(#options == 1 and options[1].weaponOption == 0)
        assert(api.GetCollectionInfoForSlotAndOption(0,0,1).name ~= '')
        assert(api.GetCollectionInfoForSlotAndOption(0,0,20) == nil)
    "#).unwrap();
}

#[test]
fn transmog_outfit_collection_names_and_capacity_are_enforced() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(r#"
        local api = C_TransmogCollection
        assert(api.IsValidCustomSetName('Raid'))
        assert(not api.IsValidCustomSetName(''))
        assert(not api.IsValidCustomSetName('bad\nname'))
        assert(api.GetNumMaxCustomSets() > 0)
        local outfits = C_TransmogOutfitInfo
        assert(outfits.GetMaxNumberOfUsableOutfits() == outfits.GetNumberOfOutfitsUnlockedForSource(2))
        assert(outfits.GetMaxNumberOfTotalOutfitsForSource(2) >= outfits.GetMaxNumberOfUsableOutfits())
        assert(outfits.GetNextOutfitCost() == 0)
        assert(C_TransmogSets.IsUsingDefaultSetsFilters())
        C_TransmogSets.SetSetsFilter(1,false)
        assert(not C_TransmogSets.IsUsingDefaultSetsFilters())
        C_TransmogSets.SetDefaultSetsFilters()
        assert(C_TransmogSets.IsUsingDefaultSetsFilters())
        assert(C_TransmogSets.GetSetsFilter(1))
    "#).unwrap();
}
