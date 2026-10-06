//! Tests for transmog and heirloom collection APIs:
//! C_TransmogCollection, C_Transmog, TransmogUtil, C_Heirloom, C_TransmogSets.

#[path = "c_transmog_heirloom_api/heirloom.rs"]
mod heirloom;

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_TransmogCollection
// ============================================================================

#[test]
fn test_transmog_collection_get_appearance_sources() {
    let env = env();
    // visual_id 1 is the first Head appearance in defaults
    let result: String = env
        .eval(
            r#"
            local sources = C_TransmogCollection.GetAppearanceSources(1)
            if #sources ~= 1 then return "count=" .. #sources end
            local s = sources[1]
            if s.sourceID ~= 1 then return "sourceID=" .. tostring(s.sourceID) end
            if s.visualID ~= 1 then return "visualID=" .. tostring(s.visualID) end
            if s.categoryID ~= 1 then return "categoryID=" .. tostring(s.categoryID) end
            if not s.isCollected then return "not collected" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_get_appearance_sources_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetAppearanceSources(99999)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_collection_get_source_info() {
    let env = env();
    // source_id 1 is the first Head appearance in defaults
    let (source_id, is_collected): (i32, bool) = env
        .eval(
            r#"
            local info = C_TransmogCollection.GetSourceInfo(1)
            return info.sourceID, info.isCollected
            "#,
        )
        .unwrap();
    assert_eq!(source_id, 1);
    assert!(is_collected);
}

#[test]
fn test_transmog_collection_player_has_transmog() {
    let env = env();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmog(1, 0)")
        .unwrap();
    assert!(!has);
}

#[test]
fn test_transmog_collection_player_has_transmog_by_item_info() {
    let env = env();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmogByItemInfo('item:1')")
        .unwrap();
    assert!(!has);
}

#[test]
fn test_transmog_collection_player_has_transmog_item_modified_appearance() {
    let env = env();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmogItemModifiedAppearance(1)")
        .unwrap();
    assert!(!has);
}

#[test]
fn test_transmog_add_then_has() {
    let env = env();
    env.exec("A_Admin.AddTransmog(42)").unwrap();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmog(42, 0)")
        .unwrap();
    assert!(has, "Should have transmog after AddTransmog");
}

#[test]
fn test_transmog_add_then_has_by_item_info() {
    let env = env();
    env.exec("A_Admin.AddTransmog(99)").unwrap();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmogByItemInfo('item:99:0')")
        .unwrap();
    assert!(has, "Should find transmog via item info link");
}

#[test]
fn test_transmog_add_then_has_modified_appearance() {
    let env = env();
    env.exec("A_Admin.AddTransmog(77)").unwrap();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmogItemModifiedAppearance(77)")
        .unwrap();
    assert!(has, "Should find transmog via modified appearance ID");
}

#[test]
fn test_transmog_remove_then_not_has() {
    let env = env();
    env.exec("A_Admin.AddTransmog(42)").unwrap();
    env.exec("A_Admin.RemoveTransmog(42)").unwrap();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmog(42, 0)")
        .unwrap();
    assert!(!has, "Should not have transmog after RemoveTransmog");
}

#[test]
fn test_admin_add_transmog_appearance() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local before = #C_TransmogCollection.GetCategoryAppearances(1, nil)
            A_Admin.AddTransmogAppearance(9999, 1, 77777)
            local after = #C_TransmogCollection.GetCategoryAppearances(1, nil)
            if after ~= before + 1 then return "count: " .. before .. " -> " .. after end
            local info = C_TransmogCollection.GetSourceInfo(9999)
            if info.sourceID ~= 9999 then return "sourceID=" .. tostring(info.sourceID) end
            if info.categoryID ~= 1 then return "categoryID=" .. tostring(info.categoryID) end
            if info.itemID ~= 77777 then return "itemID=" .. tostring(info.itemID) end
            if not info.isCollected then return "not collected" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_uncollected_returns_false() {
    let env = env();
    env.exec("A_Admin.AddTransmog(42)").unwrap();
    let has: bool = env
        .eval("return C_TransmogCollection.PlayerHasTransmog(999, 0)")
        .unwrap();
    assert!(!has, "Different ID should not match");
}

#[test]
fn test_transmog_collection_get_item_info_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_TransmogCollection.GetItemInfo(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_transmog_collection_get_num_transmog_sources() {
    let env = env();
    let count: i32 = env
        .eval("return C_TransmogCollection.GetNumTransmogSources()")
        .unwrap();
    assert_eq!(count, 7, "default source filter list has 7 source types");
}

#[test]
fn test_transmog_collection_filter_surface_supports_wardrobe_dropdowns() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_TransmogCollection.GetClassFilter() ~= 2 then return "class_filter" end
            if not C_TransmogCollection.GetCollectedShown() then return "collected" end
            if not C_TransmogCollection.GetUncollectedShown() then return "uncollected" end
            if C_TransmogCollection.GetAllFactionsShown() then return "factions" end
            if C_TransmogCollection.GetAllRacesShown() then return "races" end
            if not C_TransmogCollection.IsValidTransmogSource(1) then return "source_1" end
            if C_TransmogCollection.IsValidTransmogSource(99) then return "source_99" end
            if C_TransmogCollection.GetFilteredCategoryCollectedCount(1) ~= 4 then return "collected_count" end
            if C_TransmogCollection.GetFilteredCategoryTotal(1) ~= 5 then return "total_count" end
            local sources = C_TransmogCollection.GetValidAppearanceSourcesForClass(1, 2, 1, nil)
            if #sources ~= 1 then return "valid_sources=" .. #sources end
            if not sources[1].playerCanCollect then return "playerCanCollect" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_seeds_shirts_and_tabards() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local shirtRows = C_TransmogCollection.GetCategoryAppearances(Enum.TransmogCollectionType.Shirt, nil)
            if #shirtRows ~= 2 then return "shirts=" .. tostring(#shirtRows) end

            local tabardRows = C_TransmogCollection.GetCategoryAppearances(Enum.TransmogCollectionType.Tabard, nil)
            if #tabardRows ~= 1 then return "tabards=" .. tostring(#tabardRows) end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_source_type_filter_changes_category_results() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if #C_TransmogCollection.GetCategoryAppearances(1, nil) ~= 5 then return "initial" end
            C_TransmogCollection.SetSourceTypeFilter(1, false)
            if C_TransmogCollection.IsUsingDefaultFilters() then return "still_default" end
            if C_TransmogCollection.IsSourceTypeFilterChecked(1) then return "source_1_checked" end
            local filtered = C_TransmogCollection.GetCategoryAppearances(1, nil)
            if #filtered ~= 2 then return "filtered_count=" .. #filtered end
            for _, appearance in ipairs(filtered) do
                if appearance.sourceType == 1 then return "source_1_present" end
            end
            C_TransmogCollection.SetDefaultFilters()
            if not C_TransmogCollection.IsUsingDefaultFilters() then return "not_default" end
            if #C_TransmogCollection.GetCategoryAppearances(1, nil) ~= 5 then return "restored" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_collected_filters_change_counts_and_results() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            C_TransmogCollection.SetUncollectedShown(false)
            if C_TransmogCollection.GetUncollectedShown() then return "uncollected_still_shown" end
            if C_TransmogCollection.GetFilteredCategoryTotal(1) ~= 4 then return "collected_total" end
            if #C_TransmogCollection.GetCategoryAppearances(1, nil) ~= 4 then return "collected_rows" end
            C_TransmogCollection.SetCollectedShown(false)
            C_TransmogCollection.SetUncollectedShown(true)
            if C_TransmogCollection.GetCollectedShown() then return "collected_still_shown" end
            if C_TransmogCollection.GetFilteredCategoryTotal(1) ~= 1 then return "uncollected_total" end
            local rows = C_TransmogCollection.GetCategoryAppearances(1, nil)
            if #rows ~= 1 then return "uncollected_rows=" .. #rows end
            if rows[1].isCollected then return "row_collected" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_search_filters_category_results() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local searchType = Enum.TransmogSearchType.Items
            if not C_TransmogCollection.SetSearch(searchType, "99999") then return "not_finished" end
            if C_TransmogCollection.IsSearchInProgress(searchType) then return "in_progress" end
            if C_TransmogCollection.IsSearchDBLoading() then return "db_loading" end
            if C_TransmogCollection.SearchSize(searchType) ~= 1 then return "size" end
            if C_TransmogCollection.SearchProgress(searchType) ~= 1 then return "progress" end
            local rows = C_TransmogCollection.GetCategoryAppearances(1, nil)
            if #rows ~= 1 then return "rows=" .. #rows end
            if rows[1].itemID ~= 99999 then return "item=" .. tostring(rows[1].itemID) end
            C_TransmogCollection.ClearSearch(searchType)
            if #C_TransmogCollection.GetCategoryAppearances(1, nil) ~= 5 then return "cleared" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_get_all_appearance_sources_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetAllAppearanceSources(1)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_collection_get_illusions_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetIllusions()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_collection_get_appearance_camera_id() {
    let env = env();
    let id: i32 = env
        .eval("return C_TransmogCollection.GetAppearanceCameraID(1)")
        .unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_transmog_collection_get_category_appearances() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local appearances = C_TransmogCollection.GetCategoryAppearances(1, nil)
            if #appearances ~= 5 then return "count=" .. #appearances end
            local a = appearances[1]
            if not a.visualID then return "no visualID" end
            if a.isCollected == nil then return "no isCollected" end
            if a.uiOrder ~= 1 then return "uiOrder=" .. tostring(a.uiOrder) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_get_category_appearances_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetCategoryAppearances(99, nil)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_collection_get_category_info_armor() {
    let env = env();
    let (name, is_weapon): (String, bool) = env
        .eval("return C_TransmogCollection.GetCategoryInfo(1)")
        .unwrap();
    assert_eq!(name, "Head");
    assert!(!is_weapon);
}

#[test]
fn test_transmog_collection_get_category_info_weapon() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name, isWeapon, canEnchant, canMainHand, canOffHand = C_TransmogCollection.GetCategoryInfo(14)
            if name ~= "One-Handed Swords" then return "name=" .. tostring(name) end
            if not isWeapon then return "not weapon" end
            if not canEnchant then return "not enchantable" end
            if not canMainHand then return "not mainhand" end
            if not canOffHand then return "not offhand" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_get_category_info_shield() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name, isWeapon, canEnchant, canMainHand, canOffHand = C_TransmogCollection.GetCategoryInfo(18)
            if not isWeapon then return "not weapon" end
            if canEnchant then return "enchantable" end
            if canMainHand then return "mainhand" end
            if not canOffHand then return "not offhand" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_collection_player_knows_source() {
    let env = env();
    let knows: bool = env
        .eval("return C_TransmogCollection.PlayerKnowsSource(1)")
        .unwrap();
    assert!(!knows);
}

#[test]
fn test_transmog_collection_is_appearance_hidden_visual() {
    let env = env();
    let hidden: bool = env
        .eval("return C_TransmogCollection.IsAppearanceHiddenVisual(1)")
        .unwrap();
    assert!(!hidden);
}

#[test]
fn test_transmog_collection_is_source_type_filter_checked() {
    let env = env();
    let checked: bool = env
        .eval("return C_TransmogCollection.IsSourceTypeFilterChecked(1)")
        .unwrap();
    assert!(checked);
}

#[test]
fn test_transmog_collection_get_show_missing_source_in_item_tooltips() {
    let env = env();
    let show: bool = env
        .eval("return C_TransmogCollection.GetShowMissingSourceInItemTooltips()")
        .unwrap();
    assert!(show);
}

// ============================================================================
// C_Transmog
// ============================================================================

#[test]
fn test_transmog_get_all_set_appearances_by_id_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_Transmog.GetAllSetAppearancesByID(1)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_get_applied_source_id_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Transmog.GetAppliedSourceID(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_admin_set_transmog_for_slot() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- No transmog applied initially
            if C_Transmog.GetAppliedSourceID(1) ~= nil then return "not nil initially" end
            A_Admin.SetTransmogForSlot(1, 42)
            local id = C_Transmog.GetAppliedSourceID(1)
            if id ~= 42 then return "id=" .. tostring(id) end
            -- Other slots unaffected
            if C_Transmog.GetAppliedSourceID(2) ~= nil then return "slot 2 affected" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_applied_altered_appearance() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_Transmog.GetAppliedAlteredAppearance(1) ~= nil then return "not nil initially" end
            A_Admin.SetTransmogForSlot(1, 42)
            local altered = C_Transmog.GetAppliedAlteredAppearance(1)
            if altered ~= 42 then return "altered=" .. tostring(altered) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_player_has_transmog_by_item_info_and_is_at_npc() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_Transmog.IsAtTransmogNPC() then return "at npc unexpectedly" end
            if not C_Transmog.PlayerHasTransmogByItemInfo("item:31110") then return "missing collected item" end
            if C_Transmog.PlayerHasTransmogByItemInfo("item:99989") then return "found uncollected item" end
            A_Admin.AddTransmog(77)
            if not C_Transmog.PlayerHasTransmogByItemInfo("item:77:0") then return "missing collected source id" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

// ============================================================================
// TransmogUtil
// ============================================================================

#[test]
fn test_transmog_util_get_transmog_location() {
    let env = env();
    let (slot_name, transmog_type, modification): (String, i32, i32) = env
        .eval(
            r#"
            local loc = TransmogUtil.GetTransmogLocation("HeadSlot", 0, 0)
            return loc.slotName, loc.transmogType, loc.modification
            "#,
        )
        .unwrap();
    assert_eq!(slot_name, "HeadSlot");
    assert_eq!(transmog_type, 0);
    assert_eq!(modification, 0);
}

#[test]
fn test_transmog_util_get_transmog_location_boolean_modification() {
    let env = env();
    let (slot_name, modification): (String, i32) = env
        .eval(
            r#"
            local loc = TransmogUtil.GetTransmogLocation("HEADSLOT", 0, false)
            return loc.slotName, loc.modification
            "#,
        )
        .unwrap();
    assert_eq!(slot_name, "HEADSLOT");
    assert_eq!(modification, 0);
}

#[test]
fn test_transmog_location_methods() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local loc = TransmogUtil.GetTransmogLocation("HEADSLOT", 0, false)
            if not loc:IsAppearance() then return "not appearance" end
            if loc:IsIllusion() then return "is illusion" end
            if loc:IsEitherHand() then return "is hand" end
            if loc:IsSecondary() then return "is secondary" end
            if loc:GetSlotName() ~= "HEADSLOT" then return "bad slot" end
            local loc2 = TransmogUtil.GetTransmogLocation("HEADSLOT", 0, false)
            if not loc:IsEqual(loc2) then return "not equal" end
            local loc3 = TransmogUtil.GetTransmogLocation("MAINHANDSLOT", 0, false)
            if not loc3:IsMainHand() then return "not mainhand" end
            if not loc3:IsEitherHand() then return "not either hand" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "TransmogLocation methods: {result}");
}

#[test]
fn test_transmog_util_create_transmog_location() {
    let env = env();
    let (slot_id, transmog_type, modification): (i32, i32, i32) = env
        .eval(
            r#"
            local loc = TransmogUtil.CreateTransmogLocation(1, 0, 0)
            return loc.slotID, loc.transmogType, loc.modification
            "#,
        )
        .unwrap();
    assert_eq!(slot_id, 1);
    assert_eq!(transmog_type, 0);
    assert_eq!(modification, 0);
}

#[test]
fn test_transmog_util_get_best_item_modified_appearance_id_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return TransmogUtil.GetBestItemModifiedAppearanceID(nil) == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// C_TransmogSets
// ============================================================================

#[test]
fn test_transmog_sets_get_base_set_id() {
    let env = env();
    let id: i32 = env.eval("return C_TransmogSets.GetBaseSetID(1)").unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_transmog_sets_get_variant_sets_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogSets.GetVariantSets(1)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_sets_get_set_info() {
    let env = env();
    let missing: bool = env.eval("return C_TransmogSets.GetSetInfo(1) == nil").unwrap();
    assert!(missing, "unknown catalog entry must not fabricate set zero");
}

#[test]
fn test_transmog_sets_get_set_primary_appearances_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogSets.GetSetPrimaryAppearances(1)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_sets_get_all_sets_empty() {
    let env = env();
    let count: i32 = env.eval("return #C_TransmogSets.GetAllSets()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_sets_get_base_sets_empty() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local baseSets = C_TransmogSets.GetBaseSets()
            if type(baseSets) ~= "table" then return "type=" .. type(baseSets) end
            if #baseSets ~= 0 then return "count=" .. tostring(#baseSets) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_transmog_sets_get_usable_sets_empty() {
    let env = env();
    let count: i32 = env.eval("return #C_TransmogSets.GetUsableSets()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_sets_has_available_sets_false_when_set_inventory_is_empty() {
    let env = env();
    let has_available_sets: bool = env
        .eval("return C_TransmogSets.HasAvailableSets()")
        .unwrap();
    assert!(!has_available_sets);
}

#[test]
fn test_transmog_sets_is_base_set_collected() {
    let env = env();
    let collected: bool = env
        .eval("return C_TransmogSets.IsBaseSetCollected(1)")
        .unwrap();
    assert!(!collected);
}

#[test]
fn test_transmog_sets_get_sources_for_slot_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogSets.GetSourcesForSlot(1, 1)")
        .unwrap();
    assert_eq!(count, 0);
}
