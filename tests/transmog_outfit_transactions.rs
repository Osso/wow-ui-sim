#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::{WowLuaEnv, state::AppearanceSourceInfo};

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_appearance_sources.insert(190001, AppearanceSourceInfo {
        category: 1, item_appearance_id: 90001, can_have_illusion: false,
        icon: 135771, is_collected: true, item_link: String::new(), transmoglink: String::new(),
        source_type: None, item_subclass: 0, ignore_model_attachment_checks_for_illusion: false,
    });
    env
}

#[test]
fn transmog_outfit_transactions_custom_set_stages_and_prices_slots() {
    let env = env();
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid',135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Raid').outfitID)
        local list = {{appearanceID=190001, secondaryAppearanceID=0, illusionID=0}}
        local set = C_TransmogCollection.NewCustomSet('Head',135771,list)
        api.SetOutfitToCustomSet(set)
        assert(api.HasPendingOutfitTransmogs())
        assert(api.GetViewedOutfitSlotInfo(0,0,0).transmogID == 190001)
        assert(api.GetPendingTransmogCost() == 0)
        assert(api.GetItemModifiedAppearanceEffectiveCategory(190001) == 1)
        assert(api.GetItemModifiedAppearanceEffectiveCategory(999999) == 0)
        assert(api.GetUnassignedAtlasForSlot(0) == 'transmog-gearslot-unassigned-head')
        assert(api.GetUnassignedDisplayAtlasForSlot(0) == 'transmog-appearance-unassigned-head')
    "#).unwrap();
}

#[test]
fn transmog_outfit_transactions_hyperlinks_roundtrip_and_reject_invalid_data() {
    let env = env();
    env.eval::<()>(r#"
        local api = C_TransmogCollection
        local list = {{appearanceID=190001,secondaryAppearanceID=190002,illusionID=7},
                      {appearanceID=8,secondaryAppearanceID=0,illusionID=9}}
        local link = api.GetCustomSetHyperlinkFromItemTransmogInfoList(list)
        assert(type(link) == 'string' and link:find('|Htransmogset:',1,true))
        local decoded = api.GetItemTransmogInfoListFromCustomSetHyperlink(link)
        assert(#decoded == 2 and decoded[1].appearanceID == 190001)
        assert(decoded[1].secondaryAppearanceID == 190002 and decoded[1].illusionID == 7)
        assert(decoded[2].appearanceID == 8 and decoded[2].illusionID == 9)
        assert(api.GetItemTransmogInfoListFromCustomSetHyperlink('item:190001') == nil)
        assert(api.GetItemTransmogInfoListFromCustomSetHyperlink('|Htransmogset:1:bad|h[X]|h') == nil)
    "#).unwrap();
}

#[test]
fn transmog_outfit_transactions_failed_apply_preserves_pending_and_money() {
    let env = env();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.money = 49;
        sim.transmog_outfits.slot_cost = 50;
    }
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid',135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Raid').outfitID)
        api.SetPendingTransmog(0,0,0,190001,1)
        local cost, flags = api.GetPendingTransmogCost()
        assert(cost == 50 and flags == 0)
        assert(not pcall(api.CommitAndApplyAllPending,false))
        assert(GetMoney() == 49 and api.HasPendingOutfitTransmogs())
        assert(api.GetActiveOutfitID() == 0)
        api.ClearAllPendingTransmogs()
        api.SetPendingTransmog(0,0,0,999999,1)
        assert(not pcall(api.CommitAndApplyAllPending,false))
        assert(GetMoney() == 49 and api.HasPendingOutfitTransmogs())
    "#).unwrap();
    env.state().borrow_mut().player.money = 100;
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.SetPendingTransmog(0,0,0,190001,1)
        api.CommitAndApplyAllPending(false)
        assert(GetMoney() == 50 and not api.HasPendingOutfitTransmogs())
    "#).unwrap();
}
