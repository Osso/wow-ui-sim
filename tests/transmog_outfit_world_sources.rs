#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn transmog_outfit_world_sources_existing_collection_can_be_applied() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(r#"
        local source = C_TransmogCollection.GetAppearanceSources(1)[1]
        assert(source and source.isCollected)
        local info = C_TransmogCollection.GetAppearanceSourceInfo(source.sourceID)
        assert(info and info.isCollected and info.itemAppearanceID == source.visualID)
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Default collection',135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Default collection').outfitID)
        api.SetPendingTransmog(0,0,0,source.sourceID,1)
        local pending = api.GetViewedOutfitSlotInfo(0,0,0)
        assert(pending.isPendingCollected and pending.canTransmogrify)
        assert(api.GetItemModifiedAppearanceEffectiveCategory(source.sourceID) == source.categoryID)
        api.CommitAndApplyAllPending(false)
        assert(api.GetViewedOutfitSlotInfo(0,0,0).isTransmogrified)
    "#).unwrap();
}
