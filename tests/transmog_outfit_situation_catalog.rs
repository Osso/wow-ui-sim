#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::c_api::c_transmog_outfit_info::{SituationCategory, SituationGroup};
use wow_ui_sim::lua_api::{WowLuaEnv, state::EquippedItem};

#[test]
fn transmog_outfit_situation_catalog_options_reflect_pending_selection() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_outfits.situation_categories.push(SituationCategory {
        trigger_id: 2, name: "Specialization".into(), description: "Select spec".into(), is_radio_button: true,
        groups: vec![SituationGroup { group_id: 70, secondary_id: 3,
            options: vec![("Retribution".into(), (2,70,3,4))] }],
    });
    env.eval::<()>(r#"
        local api = C_TransmogOutfitInfo
        api.AddNewOutfit('Raid',135771)
        api.ChangeViewedOutfit(api.GetOutfitInfoByName('Raid').outfitID)
        local categories = api.GetUISituationCategoriesAndOptions()
        assert(#categories == 1 and categories[1].triggerID == 2)
        assert(categories[1].name == 'Specialization' and categories[1].isRadioButton)
        local group = categories[1].groupData[1]
        assert(group.groupID == 70 and group.secondaryID == 3)
        local option = group.optionData[1]
        assert(option.name == 'Retribution' and not option.value)
        assert(option.option.specID == 70 and option.option.equipmentSetID == 4)
        api.UpdatePendingSituation(option.option, true)
        assert(api.GetUISituationCategoriesAndOptions()[1].groupData[1].optionData[1].value)
        api.ClearAllPendingSituations()
        assert(not api.GetUISituationCategoriesAndOptions()[1].groupData[1].optionData[1].value)
    "#).unwrap();
}

#[test]
fn transmog_outfit_situation_catalog_equipped_option_reads_actual_item_inventory_type() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.equipped_items.clear();
    assert_eq!(env.eval::<Option<i32>>("return C_TransmogOutfitInfo.GetEquippedSlotOptionFromTransmogSlot(12)").unwrap(), None);
    env.state().borrow_mut().player.equipped_items.insert(16, EquippedItem {
        item_id: 250448, enchant_id: 0, gem_ids: [0;3],
    });
    assert_eq!(env.eval::<i32>("return C_TransmogOutfitInfo.GetEquippedSlotOptionFromTransmogSlot(12)").unwrap(), 2);
}
