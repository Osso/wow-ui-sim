//! Tests for C_Collection namespaces: C_PetJournal, C_MountJournal, C_ToyBox.
//! Transmog/heirloom tests are in c_transmog_heirloom_api.rs.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_PetJournal
// ============================================================================

#[test]
fn test_pet_journal_get_num_pets_returns_two_numbers() {
    let env = env();
    let (total, owned): (i32, i32) = env.eval("return C_PetJournal.GetNumPets()").unwrap();
    assert_eq!(total, 10, "total pets in default world");
    assert_eq!(owned, 9, "collected pets (Pocopoc is not collected)");
}

#[test]
fn test_pet_journal_get_num_pet_types() {
    let env = env();
    let count: i32 = env.eval("return C_PetJournal.GetNumPetTypes()").unwrap();
    assert_eq!(count, 10, "WoW has 10 pet families");
}

#[test]
fn test_pet_journal_get_num_pet_sources() {
    let env = env();
    let count: i32 = env.eval("return C_PetJournal.GetNumPetSources()").unwrap();
    assert_eq!(count, 10, "WoW has 10 pet source types");
}

#[test]
fn test_pet_journal_get_pet_info_by_index_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_PetJournal.GetPetInfoByIndex(999) == nil")
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
        .eval("return C_PetJournal.GetPetInfoBySpeciesID(99999) == nil")
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
    assert_eq!(collected, 9);
    assert_eq!(total, 10);
}

#[test]
fn test_pet_journal_empty_loadout_slot_has_nil_pet_id() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local petID, ability1ID, ability2ID, ability3ID, locked = C_PetJournal.GetPetLoadOutInfo(1)
            if petID ~= nil then return "petID=" .. tostring(petID) end
            if ability1ID ~= 0 then return "ability1ID=" .. tostring(ability1ID) end
            if ability2ID ~= 0 then return "ability2ID=" .. tostring(ability2ID) end
            if ability3ID ~= 0 then return "ability3ID=" .. tostring(ability3ID) end
            if locked ~= false then return "locked=" .. tostring(locked) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "empty loadout slot shape: {result}");
}

// ============================================================================
// C_MountJournal
// ============================================================================

#[test]
fn test_mount_journal_get_num_mounts() {
    let env = env();
    let count: i32 = env.eval("return C_MountJournal.GetNumMounts()").unwrap();
    assert_eq!(count, 10);
}

#[test]
fn test_mount_journal_get_num_displayed_mounts() {
    let env = env();
    let count: i32 = env
        .eval("return C_MountJournal.GetNumDisplayedMounts()")
        .unwrap();
    assert_eq!(count, if cfg!(feature = "client-retail") { 0 } else { 10 });
}

#[test]
fn test_mount_journal_get_mount_info_by_id_returns_tuple() {
    let env = env();
    let is_active: bool = env
        .eval("local _,_,_, isActive = C_MountJournal.GetMountInfoByID(107); return isActive")
        .unwrap();
    assert!(!is_active);
}

#[test]
fn test_mount_journal_get_mount_info_by_id_first_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_MountJournal.GetMountInfoByID(99999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_mount_journal_get_mount_ids_empty_table() {
    let env = env();
    let count: i32 = env.eval("return #C_MountJournal.GetMountIDs()").unwrap();
    assert_eq!(count, 10);
}

#[test]
fn test_mount_journal_get_collected_filter_setting() {
    let env = env();
    let val: bool = env
        .eval("return C_MountJournal.GetCollectedFilterSetting(1)")
        .unwrap();
    assert_eq!(val, !cfg!(feature = "client-retail"));
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
    env.exec("C_MountJournal.SetDefaultFilters()").unwrap();
    let (is_fav, can_fav): (bool, bool) =
        env.eval("return C_MountJournal.GetIsFavorite(1)").unwrap();
    assert!(!is_fav);
    assert!(can_fav);

    let (is_fav, can_fav): (bool, bool) =
        env.eval("return C_MountJournal.GetIsFavorite(0)").unwrap();
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

#[test]
fn test_mount_journal_get_applied_mount_equipment_id_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_MountJournal.GetAppliedMountEquipmentID() == nil")
        .unwrap();
    assert!(is_nil, "should return nil when no mount equipment applied");
}

// ============================================================================
// C_ToyBox
// ============================================================================

#[test]
fn test_toy_box_get_toy_info() {
    let env = env();
    let (item_id, _name, _icon, is_fav, has_fanfare, quality): (i32, String, i32, bool, bool, i32) =
        env.eval("return C_ToyBox.GetToyInfo(1)").unwrap();
    assert_eq!(item_id, 0);
    assert!(!is_fav);
    assert!(!has_fanfare);
    assert_eq!(quality, 0);
}

#[test]
fn test_toy_box_is_toy_usable_unknown() {
    let env = env();
    let usable: bool = env.eval("return C_ToyBox.IsToyUsable(1)").unwrap();
    assert!(!usable, "unknown toy should not be usable");
}

#[test]
fn test_toy_box_is_toy_usable_collected() {
    let env = env();
    let usable: bool = env.eval("return C_ToyBox.IsToyUsable(166779)").unwrap();
    assert!(usable, "collected toy should be usable");
}

#[test]
fn test_toy_box_is_toy_usable_uncollected() {
    let env = env();
    let usable: bool = env.eval("return C_ToyBox.IsToyUsable(187421)").unwrap();
    assert!(!usable, "uncollected toy should not be usable");
}

#[test]
fn test_toy_box_info_filter_helpers_exist() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            return type(C_ToyBoxInfo) == "table"
                and C_ToyBoxInfo.IsUsingDefaultFilters() == true
                and C_ToyBoxInfo.IsToySourceValid(1) == true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
fn test_player_has_toy_reads_collected_toy_state() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if not PlayerHasToy(166779) then return "missing_collected" end
            if PlayerHasToy(187421) then return "uncollected_present" end
            if PlayerHasToy(1) then return "unknown_present" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_toy_box_get_set_is_favorite() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_ToyBox.GetIsFavorite(166779) then return "already fav" end
            C_ToyBox.SetIsFavorite(166779, true)
            if not C_ToyBox.GetIsFavorite(166779) then return "not fav after set" end
            C_ToyBox.SetIsFavorite(166779, false)
            if C_ToyBox.GetIsFavorite(166779) then return "still fav after unset" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_toy_box_filter_stubs() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if not C_ToyBox.GetCollectedShown() then return "collected" end
            if not C_ToyBox.GetUncollectedShown() then return "uncollected" end
            if not C_ToyBox.GetUnusableShown() then return "unusable" end
            if not C_ToyBox.IsExpansionTypeFilterChecked(1) then return "expansion" end
            if not C_ToyBox.IsSourceTypeFilterChecked(1) then return "source" end
            C_ToyBox.SetCollectedShown(false)
            C_ToyBox.SetExpansionTypeFilter(1, false)
            C_ToyBox.SetSourceTypeFilter(1, false)
            C_ToyBox.SetUncollectedShown(false)
            C_ToyBox.SetUnusableShown(false)
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_toy_box_get_toy_link() {
    let env = env();
    let link: String = env.eval("return C_ToyBox.GetToyLink(166779)").unwrap();
    assert!(link.contains("Hearthstone Game Table"));
    assert!(link.contains("|Hitem:166779"));
}

#[test]
fn test_toy_box_get_toy_link_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_ToyBox.GetToyLink(1) == nil").unwrap();
    assert!(is_nil, "unknown toy should return nil");
}

#[test]
fn test_toy_box_force_toy_refilter() {
    let env = env();
    env.eval::<()>("C_ToyBox.ForceToyRefilter()").unwrap();
}

#[test]
fn test_toy_box_has_favorites() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_ToyBox.HasFavorites() then return "has favs initially" end
            C_ToyBox.SetIsFavorite(166779, true)
            if not C_ToyBox.HasFavorites() then return "no favs after set" end
            C_ToyBox.SetIsFavorite(166779, false)
            if C_ToyBox.HasFavorites() then return "has favs after unset" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_toy_box_get_num_toys() {
    let env = env();
    let count: i32 = env.eval("return C_ToyBox.GetNumToys()").unwrap();
    assert_eq!(count, 10);
}

#[test]
fn test_toy_box_get_toy_from_index() {
    let env = env();
    let id: i32 = env.eval("return C_ToyBox.GetToyFromIndex(1)").unwrap();
    assert_eq!(id, 166779);
}

#[test]
fn test_toy_box_get_toy_from_index_out_of_range() {
    let env = env();
    let id: i32 = env.eval("return C_ToyBox.GetToyFromIndex(9999)").unwrap();
    assert_eq!(
        id, -1,
        "out-of-range index must return -1 so ToySpellButton_UpdateButton hides the slot"
    );
}

#[test]
fn test_toy_box_get_num_filtered_toys() {
    let env = env();
    let count: i32 = env.eval("return C_ToyBox.GetNumFilteredToys()").unwrap();
    assert_eq!(count, 10);
}
