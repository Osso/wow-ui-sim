#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::c_api::c_transmog_sets::TransmogSetInfo;
use wow_ui_sim::lua_api::{WowLuaEnv, state::AppearanceSourceInfo};

#[test]
fn transmog_outfit_catalog_inputs_sets_filter_and_stage_collected_sources() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.transmog_sets.entries = vec![
            TransmogSetInfo { set_id: 71, name: "Raid Plate".into(), collected: true,
                valid_for_character: true, ..Default::default() },
            TransmogSetInfo { set_id: 72, name: "Arena Plate".into(), collected: false,
                valid_for_character: true, is_pvp: true, ..Default::default() },
        ];
        sim.transmog_sets.slot_sources.insert((71,0), vec![190001]);
        sim.transmog_appearance_sources.insert(190001, AppearanceSourceInfo {
            category: 1, item_appearance_id: 90001, can_have_illusion: false,
            icon: 135771, is_collected: true, item_link: String::new(), transmoglink: String::new(),
            source_type: None, item_subclass: 0, ignore_model_attachment_checks_for_illusion: false,
        });
    }
    env.eval::<()>(r#"
        local sets = C_TransmogSets.GetAvailableSets()
        assert(#sets == 2 and sets[1].setID == 71 and sets[1].name == 'Raid Plate')
        assert(sets[1].favorite == false and sets[1].expansionID == 0)
        C_TransmogSets.SetSetsFilter(2,false)
        assert(#C_TransmogSets.GetAvailableSets() == 1)
        C_TransmogSets.SetDefaultSetsFilters()
        C_TransmogSets.SetSetsFilter(4,false)
        assert(#C_TransmogSets.GetAvailableSets() == 1)
        local api = C_TransmogOutfitInfo
        local ids = api.GetSourceIDsForSlot(71,0)
        assert(#ids == 1 and ids[1] == 190001)
        local sources = api.GetSetSourcesForSlot(71,0)
        assert(#sources == 1 and sources[1].isCollected and sources[1].category == 1)
        api.AddNewOutfit('Raid',135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Raid').outfitID)
        api.SetOutfitToSet(71)
        assert(api.GetViewedOutfitSlotInfo(0,0,0).transmogID == 190001)
        assert(api.HasPendingOutfitTransmogs())
    "#).unwrap();
}

#[test]
fn transmog_outfit_catalog_inputs_pickup_sets_cursor_and_clear_works() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid',135771)
        local id = api.GetOutfitInfoByName('Raid').outfitID
        api.PickupOutfit(id)
        local kind, cursorID = GetCursorInfo()
        assert(kind == 'transmogoutfit' and cursorID == id)
        ClearCursor()
        assert(GetCursorInfo() == nil)
        api.PickupOutfit(0)
        kind, cursorID = GetCursorInfo()
        assert(kind == 'transmogoutfit' and cursorID == 0)
    "#).unwrap();
}
