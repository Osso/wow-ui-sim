#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::{WowLuaEnv, state::AppearanceSourceInfo};

#[test]
fn patch_12_0_1_transmog_copies_outfit_to_pending_selection() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_appearance_sources.insert(190001, AppearanceSourceInfo {
        category: 1, item_appearance_id: 90001, can_have_illusion: false,
        icon: 135771, is_collected: true, item_link: String::new(), transmoglink: String::new(),
        source_type: None, item_subclass: 0, ignore_model_attachment_checks_for_illusion: false,
    });
    env.exec(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Source', 135771)
        api.AddNewOutfit('Destination', 135771)
        local source = api.GetOutfitInfoByName('Source').outfitID
        local destination = api.GetOutfitInfoByName('Destination').outfitID
        api.ChangeViewedOutfit(source)
        api.SetPendingTransmog(0,0,0,190001,1)
        api.CommitAndApplyAllPending(false)
        api.ChangeViewedOutfit(destination)
        api.SetOutfitToOutfit(source)
        assert(api.HasPendingOutfitTransmogs(), 'copy must stage source contents')
        assert(api.GetViewedOutfitSlotInfo(0,0,0).transmogID == 190001)
        api.ClearAllPendingTransmogs()
        api.ChangeViewedOutfit(source)
        assert(api.GetViewedOutfitSlotInfo(0,0,0).transmogID == 190001)
        assert(api.InTransmogEvent() == false)
        assert(api.TransmogEventActive() == false)
        assert(api.IsUsableDiscountAvailable() == false)
    "#).unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.transmog_outfits.event_active = true;
        sim.transmog_outfits.in_event = false;
        sim.transmog_outfits.usable_discount_available = true;
    }
    env.exec(r#"
        assert(C_TransmogOutfitInfo.TransmogEventActive())
        assert(not C_TransmogOutfitInfo.InTransmogEvent())
        assert(C_TransmogOutfitInfo.IsUsableDiscountAvailable())
        assert(not pcall(C_TransmogOutfitInfo.SetOutfitToOutfit, 99999))
    "#).unwrap();
}
