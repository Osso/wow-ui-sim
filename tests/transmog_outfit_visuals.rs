#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::{WowLuaEnv, state::{AppearanceSourceInfo, EquippedItem}};

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let state = env.state();
    let mut sim = state.borrow_mut();
    sim.player.equipped_items.insert(1, EquippedItem { item_id: 109984, enchant_id: 0, gem_ids: [0;3] });
    let mut source = sim.world.transmog_appearances[0].clone();
    source.source_id = 190000;
    source.visual_id = 90000;
    source.category_id = 1;
    source.item_id = 109984;
    source.is_collected = true;
    sim.world.transmog_appearances = vec![source];
    sim.transmog_appearance_sources.insert(190001, AppearanceSourceInfo {
        category: 1, item_appearance_id: 90001, can_have_illusion: false, icon: 135771,
        is_collected: true, item_link: String::new(), transmoglink: String::new(),
        source_type: None, item_subclass: 4, ignore_model_attachment_checks_for_illusion: false,
    });
    drop(sim);
    env
}

#[test]
fn transmog_outfit_visuals_shape_tracks_equipped_pending_and_applied_sources() {
    let env = env();
    env.eval::<()>(r#"
        local location = {slot=0,slotID=1,transmogType=0,isSecondary=false}
        local visual = C_Transmog.GetSlotVisualInfo(location)
        assert(visual.baseSourceID == 190000 and visual.baseVisualID == 90000)
        assert(visual.appliedSourceID == 0 and visual.pendingSourceID == 0)
        assert(visual.hasUndo == false and visual.isHideVisual == false)
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid',135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Raid').outfitID)
        api.SetPendingTransmog(0,0,0,190001,1)
        visual = C_Transmog.GetSlotVisualInfo(location)
        assert(visual.pendingSourceID == 190001 and visual.pendingVisualID == 90001)
        api.CommitAndApplyAllPending(false)
        visual = C_Transmog.GetSlotVisualInfo(location)
        assert(visual.appliedSourceID == 190001 and visual.appliedVisualID == 90001)
        assert(visual.pendingSourceID == 0)
    "#).unwrap();
}

#[test]
fn transmog_outfit_visuals_item_eligibility_returns_boolean_and_error_enum() {
    let env = env();
    env.state().borrow_mut().player.equipped_items.insert(16, EquippedItem {
        item_id: 250448, enchant_id: 0, gem_ids: [0;3],
    });
    env.state().borrow_mut().player.equipped_items.remove(&17);
    env.eval::<()>(r#"
        local can, error = C_Item.CanItemTransmogAppearance({equipmentSlotIndex=16})
        assert(can == true and error == Enum.TransmogOutfitSlotError.Ok)
        can, error = C_Item.CanItemTransmogAppearance({equipmentSlotIndex=17})
        assert(can == false and error == Enum.TransmogOutfitSlotError.NoItem)
        assert(not pcall(C_Item.CanItemTransmogAppearance, nil))
    "#).unwrap();
}
