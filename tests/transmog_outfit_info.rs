#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_transmog_outfit_info::OutfitEntry;
use wow_ui_sim::lua_api::WowLuaEnv;

const TRANSMOG_OUTFIT_INFO_SCRIPT: &str = r#"
    if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 0 or C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 0 then
        return "wrong_initial_outfit_state"
    end

    local categoryInfo = C_TransmogOutfitInfo.GetAllTransmogOutfitOptionSheatheCategoryInfo(190001)
    if not categoryInfo or #categoryInfo ~= 4 then
        return "wrong_category_count"
    end

    if categoryInfo[1].sheatheCategory ~= Enum.TransmogOutfitSlotOptionSheatheCategory.Default or categoryInfo[1].categoryName ~= "Default" then
        return "wrong_default_category"
    end

    if categoryInfo[4].sheatheCategory ~= Enum.TransmogOutfitSlotOptionSheatheCategory.Hide or categoryInfo[4].categoryName ~= "Hide" then
        return "wrong_hide_category"
    end

    if C_TransmogOutfitInfo.GetAllTransmogOutfitOptionSheatheCategoryInfo(0) ~= nil then
        return "expected_nil_category_info"
    end

    C_TransmogOutfitInfo.ChangeToOutfit(1, false)
    if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 7 or C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 0 then
        return "change_to_outfit_failed"
    end

    C_TransmogOutfitInfo.ChangeViewedOutfit(7)
    assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 7, "viewed metadata fixture should be queryable")

    C_TransmogOutfitInfo.SetPendingTransmogSheatheCategory(16, 2, Enum.TransmogOutfitSlotOptionSheatheCategory.Side)
    local pendingSheatheCategories = rawget(C_TransmogOutfitInfo, "__pendingSheatheCategories")
    if not pendingSheatheCategories or pendingSheatheCategories["16:2"] ~= Enum.TransmogOutfitSlotOptionSheatheCategory.Side then
        return "pending_sheathe_not_recorded"
    end

    C_TransmogOutfitInfo.ChangeToOutfit(1, true)
    if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 0 or C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 7 then
        return "toggle_clear_failed"
    end

    local retainedPending = rawget(C_TransmogOutfitInfo, "__pendingSheatheCategories")
    if retainedPending ~= pendingSheatheCategories or retainedPending["16:2"] ~= Enum.TransmogOutfitSlotOptionSheatheCategory.Side then
        return "pending_sheathe_not_preserved"
    end

    C_TransmogOutfitInfo.ChangeToOutfit(2, false)
    assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 9, "second catalog index should select outfit 9 before clearing")
    C_TransmogOutfitInfo.ClearOutfit()
    if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 0 or C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 7 then
        return "clear_outfit_failed"
    end
    retainedPending = rawget(C_TransmogOutfitInfo, "__pendingSheatheCategories")
    assert(retainedPending == pendingSheatheCategories and retainedPending["16:2"] == Enum.TransmogOutfitSlotOptionSheatheCategory.Side,
        "ClearOutfit should preserve pending sheathe metadata")

    return "ok"
"#;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn transmog_outfit_info_methods_track_outfit_and_sheathe_state() {
    let env = env();
    env.state().borrow_mut().transmog_outfit_catalog.entries = vec![
        OutfitEntry {
            outfit_id: 7,
            name: "Fixture outfit".into(),
            situation_categories: vec![],
            icon: 135_771,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 1,
        },
        OutfitEntry {
            outfit_id: 9,
            name: "Second fixture outfit".into(),
            situation_categories: vec![],
            icon: 132_489,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 2,
        },
    ];
    let result: String = env
        .eval(TRANSMOG_OUTFIT_INFO_SCRIPT)
        .expect("C_TransmogOutfitInfo methods should be queryable");
    assert_eq!(result, "ok");
}
