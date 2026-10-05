#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn transmog_outfit_lifecycle_creates_renames_and_delivers_events() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        local events = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('TRANSMOG_OUTFITS_CHANGED')
        frame:RegisterEvent('TRANSMOG_DISPLAYED_OUTFIT_CHANGED')
        frame:SetScript('OnEvent', function(_, event, id) events[#events+1] = {event, id} end)
        assert(api.IsValidTransmogOutfitName('Raid'))
        assert(not api.IsValidTransmogOutfitName('  '))
        api.AddNewOutfit('Raid', 135771)
        local outfit = api.GetOutfitInfoByName('raid')
        assert(outfit and outfit.icon == 135771 and outfit.playerFacingOutfitIndex == 1)
        assert(events[1][1] == 'TRANSMOG_OUTFITS_CHANGED' and events[1][2] == outfit.outfitID)
        api.CommitOutfitInfo(outfit.outfitID, 'Dungeon', 132489)
        assert(api.GetOutfitInfoByName('Raid') == nil)
        assert(api.GetOutfitInfo(outfit.outfitID).name == 'Dungeon')
        api.ChangeDisplayedOutfit(outfit.outfitID, 0, true, false)
        assert(api.GetActiveOutfitID() == outfit.outfitID)
        assert(api.IsLockedOutfit(outfit.outfitID))
        assert(not api.IsEquippedGearOutfitDisplayed())
        api.ClearDisplayedOutfit(0, false)
        assert(api.GetActiveOutfitID() == 0 and api.IsEquippedGearOutfitDisplayed())
        assert(events[#events][1] == 'TRANSMOG_DISPLAYED_OUTFIT_CHANGED')
    "#).unwrap();
}

#[test]
fn transmog_outfit_lifecycle_pending_slots_commit_revert_and_isolate_outfits() {
    let env = WowLuaEnv::new().unwrap();
    for id in [190001, 190002, 190003] {
        env.state().borrow_mut().transmog_appearance_sources.insert(id,
            wow_ui_sim::c_api::c_transmog_collection::AppearanceSourceInfo {
                category: 1, item_appearance_id: id, can_have_illusion: false,
                icon: 135771, is_collected: true, item_link: String::new(),
                transmoglink: String::new(), source_type: None, item_subclass: 0,
                ignore_model_attachment_checks_for_illusion: false,
            });
    }
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid', 135771)
        local id = api.GetOutfitInfoByName('Raid').outfitID
        api.ChangeViewedOutfit(id)
        local saved = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_SLOT_SAVE_SUCCESS')
        frame:SetScript('OnEvent', function(_, _, slot, kind, option) saved = {slot,kind,option} end)
        api.SetPendingTransmog(0, 0, 0, 190001, 1)
        assert(api.HasPendingOutfitTransmogs())
        local slot = api.GetViewedOutfitSlotInfo(0, 0, 0)
        assert(slot.transmogID == 190001 and slot.hasPending and slot.displayType == 1)
        api.RevertPendingTransmog(0, 0, 0)
        assert(not api.HasPendingOutfitTransmogs())
        api.SetPendingTransmog(0, 0, 0, 190002, 1)
        api.CommitAndApplyAllPending(false)
        assert(not api.HasPendingOutfitTransmogs())
        slot = api.GetViewedOutfitSlotInfo(0, 0, 0)
        assert(slot.transmogID == 190002 and not slot.hasPending and slot.isTransmogrified)
        assert(saved[1] == 0 and saved[2] == 0 and saved[3] == 0)
        assert(api.GetActiveOutfitID() == id)
        api.SetPendingTransmog(0, 0, 0, 190003, 1)
        api.ClearAllPendingTransmogs()
        assert(api.GetViewedOutfitSlotInfo(0, 0, 0).transmogID == 190002)
        api.AddNewOutfit('Solo', 135772)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Solo').outfitID)
        assert(api.GetViewedOutfitSlotInfo(0, 0, 0) == nil)
        api.ChangeViewedOutfit(id)
        assert(api.GetViewedOutfitSlotInfo(0, 0, 0).transmogID == 190002)
    "#).unwrap();
}

#[test]
fn transmog_outfit_lifecycle_situations_clear_reset_and_commit() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid', 135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Raid').outfitID)
        local option = {situationID=2, specID=70, loadoutID=3, equipmentSetID=4}
        local other = {situationID=2, specID=70, loadoutID=3, equipmentSetID=5}
        api.UpdatePendingSituation(option, true)
        assert(api.HasPendingOutfitSituations() and api.GetOutfitSituation(option))
        assert(not api.GetOutfitSituation(other))
        api.ClearAllPendingSituations()
        assert(not api.HasPendingOutfitSituations() and not api.GetOutfitSituation(option))
        api.UpdatePendingSituation(option, true)
        api.CommitPendingSituations()
        assert(api.GetOutfitSituation(option) and not api.HasPendingOutfitSituations())
        api.UpdatePendingSituation(option, false)
        assert(not api.GetOutfitSituation(option))
        api.ClearAllPendingSituations()
        assert(api.GetOutfitSituation(option))
        api.ResetOutfitSituations()
        assert(not api.GetOutfitSituation(option))
    "#).unwrap();
}
