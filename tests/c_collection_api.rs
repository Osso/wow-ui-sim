//! Tests for C_Collection namespaces: C_PetJournal, C_MountJournal, C_ToyBox,
//! C_TransmogCollection, C_Transmog, TransmogUtil, C_Heirloom, C_TransmogSets.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_PetJournal
// ============================================================================

#[test]
fn test_pet_journal_get_num_pets() {
    let env = env();
    let count: i32 = env.eval("return C_PetJournal.GetNumPets()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_pet_journal_get_pet_info_by_index_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_PetJournal.GetPetInfoByIndex(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_pet_journal_get_pet_info_by_pet_id_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_PetJournal.GetPetInfoByPetID('abc') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_pet_journal_get_pet_info_by_species_id_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_PetJournal.GetPetInfoBySpeciesID(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_pet_journal_pet_is_summonable() {
    let env = env();
    let summonable: bool = env
        .eval("return C_PetJournal.PetIsSummonable('abc')")
        .unwrap();
    assert!(!summonable);
}

#[test]
fn test_pet_journal_get_num_collected_info() {
    let env = env();
    let (collected, total): (i32, i32) = env
        .eval("return C_PetJournal.GetNumCollectedInfo(1)")
        .unwrap();
    assert_eq!(collected, 0);
    assert_eq!(total, 0);
}

// ============================================================================
// C_MountJournal
// ============================================================================

#[test]
fn test_mount_journal_get_num_mounts() {
    let env = env();
    let count: i32 = env.eval("return C_MountJournal.GetNumMounts()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_mount_journal_get_num_displayed_mounts() {
    let env = env();
    let count: i32 = env
        .eval("return C_MountJournal.GetNumDisplayedMounts()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_mount_journal_get_mount_info_by_id_returns_tuple() {
    let env = env();
    // Check that it returns multiple values (the 4th is isActive=false)
    let is_active: bool = env
        .eval("local _,_,_, isActive = C_MountJournal.GetMountInfoByID(1); return isActive")
        .unwrap();
    assert!(!is_active);
}

#[test]
fn test_mount_journal_get_mount_info_by_id_first_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_MountJournal.GetMountInfoByID(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_mount_journal_get_mount_ids_empty_table() {
    let env = env();
    let count: i32 = env
        .eval("return #C_MountJournal.GetMountIDs()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_mount_journal_get_collected_filter_setting() {
    let env = env();
    let val: bool = env
        .eval("return C_MountJournal.GetCollectedFilterSetting(1)")
        .unwrap();
    assert!(val);
}

#[test]
fn test_mount_journal_set_collected_filter_setting() {
    let env = env();
    env.eval::<()>("C_MountJournal.SetCollectedFilterSetting(1, false)")
        .unwrap();
}

#[test]
fn test_mount_journal_get_is_favorite() {
    let env = env();
    let (is_fav, can_fav): (bool, bool) = env
        .eval("return C_MountJournal.GetIsFavorite(1)")
        .unwrap();
    assert!(!is_fav);
    assert!(!can_fav);
}

#[test]
fn test_mount_journal_set_is_favorite() {
    let env = env();
    env.eval::<()>("C_MountJournal.SetIsFavorite(1, true)")
        .unwrap();
}

#[test]
fn test_mount_journal_summon() {
    let env = env();
    env.eval::<()>("C_MountJournal.Summon(1)").unwrap();
}

#[test]
fn test_mount_journal_dismiss() {
    let env = env();
    env.eval::<()>("C_MountJournal.Dismiss()").unwrap();
}

// ============================================================================
// C_ToyBox
// ============================================================================

#[test]
fn test_toy_box_get_toy_info() {
    let env = env();
    let (item_id, _name, _icon, is_fav, has_fanfare, quality): (
        i32,
        String,
        i32,
        bool,
        bool,
        i32,
    ) = env.eval("return C_ToyBox.GetToyInfo(1)").unwrap();
    assert_eq!(item_id, 0);
    assert!(!is_fav);
    assert!(!has_fanfare);
    assert_eq!(quality, 0);
}

#[test]
fn test_toy_box_is_toy_usable() {
    let env = env();
    let usable: bool = env.eval("return C_ToyBox.IsToyUsable(1)").unwrap();
    assert!(!usable);
}

#[test]
fn test_toy_box_get_num_toys() {
    let env = env();
    let count: i32 = env.eval("return C_ToyBox.GetNumToys()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_toy_box_get_toy_from_index() {
    let env = env();
    let id: i32 = env.eval("return C_ToyBox.GetToyFromIndex(1)").unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_toy_box_get_num_filtered_toys() {
    let env = env();
    let count: i32 = env.eval("return C_ToyBox.GetNumFilteredToys()").unwrap();
    assert_eq!(count, 0);
}

// ============================================================================
// C_TransmogCollection
// ============================================================================

#[test]
fn test_transmog_collection_get_appearance_sources_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetAppearanceSources(1)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_collection_get_source_info() {
    let env = env();
    let (source_id, is_collected): (i32, bool) = env
        .eval(
            r#"
            local info = C_TransmogCollection.GetSourceInfo(1)
            return info.sourceID, info.isCollected
            "#,
        )
        .unwrap();
    assert_eq!(source_id, 0);
    assert!(!is_collected);
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
fn test_transmog_collection_get_item_info_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_TransmogCollection.GetItemInfo(1) == nil")
        .unwrap();
    assert!(is_nil);
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
fn test_transmog_collection_get_outfits_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetOutfits()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_collection_get_num_max_outfits() {
    let env = env();
    let count: i32 = env
        .eval("return C_TransmogCollection.GetNumMaxOutfits()")
        .unwrap();
    assert_eq!(count, 20);
}

#[test]
fn test_transmog_collection_get_outfit_info_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_TransmogCollection.GetOutfitInfo(1) == nil")
        .unwrap();
    assert!(is_nil);
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
fn test_transmog_collection_get_category_appearances_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogCollection.GetCategoryAppearances(1, nil)")
        .unwrap();
    assert_eq!(count, 0);
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
fn test_transmog_get_slot_info() {
    let env = env();
    let first: bool = env
        .eval("return C_Transmog.GetSlotInfo(1)")
        .unwrap();
    assert!(!first);
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
// C_Heirloom
// ============================================================================

#[test]
fn test_heirloom_get_heirloom_info_first_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Heirloom.GetHeirloomInfo(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_heirloom_get_heirloom_info_is_pvp() {
    let env = env();
    let is_pvp: bool = env
        .eval("local _,_, isPvP = C_Heirloom.GetHeirloomInfo(1); return isPvP")
        .unwrap();
    assert!(!is_pvp);
}

#[test]
fn test_heirloom_get_heirloom_max_upgrade_level() {
    let env = env();
    let level: i32 = env
        .eval("return C_Heirloom.GetHeirloomMaxUpgradeLevel(1)")
        .unwrap();
    assert_eq!(level, 0);
}

#[test]
fn test_heirloom_get_num_heirlooms() {
    let env = env();
    let count: i32 = env.eval("return C_Heirloom.GetNumHeirlooms()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_heirloom_get_num_known_heirlooms() {
    let env = env();
    let count: i32 = env
        .eval("return C_Heirloom.GetNumKnownHeirlooms()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_heirloom_player_has_heirloom() {
    let env = env();
    let has: bool = env
        .eval("return C_Heirloom.PlayerHasHeirloom(1)")
        .unwrap();
    assert!(!has);
}

#[test]
fn test_heirloom_get_heirloom_link_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Heirloom.GetHeirloomLink(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_heirloom_can_heirloom_upgrade_from_pending() {
    let env = env();
    let can: bool = env
        .eval("return C_Heirloom.CanHeirloomUpgradeFromPending(1)")
        .unwrap();
    assert!(!can);
}

#[test]
fn test_heirloom_get_class_and_spec_filters() {
    let env = env();
    let (class_filter, spec_filter): (i32, i32) = env
        .eval("return C_Heirloom.GetClassAndSpecFilters()")
        .unwrap();
    assert_eq!(class_filter, 0);
    assert_eq!(spec_filter, 0);
}

// ============================================================================
// C_TransmogSets
// ============================================================================

#[test]
fn test_transmog_sets_get_base_set_id() {
    let env = env();
    let id: i32 = env
        .eval("return C_TransmogSets.GetBaseSetID(1)")
        .unwrap();
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
    let (set_id, name, collected): (i32, String, bool) = env
        .eval(
            r#"
            local info = C_TransmogSets.GetSetInfo(1)
            return info.setID, info.name, info.collected
            "#,
        )
        .unwrap();
    assert_eq!(set_id, 0);
    assert_eq!(name, "");
    assert!(!collected);
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
    let count: i32 = env
        .eval("return #C_TransmogSets.GetAllSets()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_transmog_sets_get_usable_sets_empty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TransmogSets.GetUsableSets()")
        .unwrap();
    assert_eq!(count, 0);
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
