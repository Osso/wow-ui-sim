#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogCategoryRecord, HousingCatalogEntryID, HousingCatalogEntryRecord,
    HousingCatalogSubcategoryRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_deprecated_housing_catalog_is_absent_from_retail_source() {
    assert!(
        !blizzard_ui_dir()
            .join("Blizzard_DeprecatedHousingCatalog")
            .exists(),
        "Retail 12.1.0.69497 no longer ships Blizzard_DeprecatedHousingCatalog"
    );
}

#[test]
fn blizzard_deprecated_housing_catalog_is_absent_from_discovery() {
    for screen in [ScreenKind::Game, ScreenKind::Login] {
        assert!(
            !discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen)
                .iter()
                .any(|(name, _)| name == "Blizzard_DeprecatedHousingCatalog"),
            "Retail 12.1.0.69497 must not discover removed Blizzard_DeprecatedHousingCatalog"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_deprecated_housing_catalog_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedHousingCatalog")
                || message.contains("Deprecated_HousingCatalog")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedHousingCatalog emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_housing_catalog_seeds_entry_subtype_enum(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(Enum.HousingCatalogEntrySubtype) == 'table' \
                and Enum.HousingCatalogEntrySubtype.Invalid == 0 \
                and Enum.HousingCatalogEntrySubtype.Unowned == 1 \
                and Enum.HousingCatalogEntrySubtype.OwnedModifiedStack == 2 \
                and Enum.HousingCatalogEntrySubtype.OwnedUnmodifiedStack == 3",
        )
        .expect("Enum.HousingCatalogEntrySubtype query should succeed");
    assert!(
        installed,
        "Deprecated_HousingCatalog.lua line 43-50 publishes the legacy \
         `Enum.HousingCatalogEntrySubtype` table (Invalid=0, Unowned=1, \
         OwnedModifiedStack=2, OwnedUnmodifiedStack=3) gated by `if not \
         Enum.HousingCatalogEntrySubtype then ... end`. The same table is also seeded \
         pre-shim in src/lua_api/globals/enum_data/missing_enums.lua:6797 — when the shim \
         runs, that entry already exists so the inner branch is skipped, but the published \
         values must still match the documented contract"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_catalog_entry_info_preserves_current_fields(env: &WowLuaEnv) {
    let entry_type: i32 = env.eval("return Enum.HousingCatalogEntryType.Decor").unwrap();
    let id = HousingCatalogEntryID { record_id: 1001, entry_type };
    env.state().borrow_mut().housing.catalog.entries.insert(id, HousingCatalogEntryRecord {
        item_id: Some(6948),
        name: "Full UI catalog chair".into(),
        is_unique_trophy: false,
        total_num_stored: Some(7),
        total_num_placed: Some(3),
    });
    env.exec(r#"
        assert(GetCVarBool('loadDeprecationFallbacks'))
        local info = C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Decor, 1001, true)
        assert(info, 'explicit entry must survive full UI startup')
        assert(info.recordID == 1001 and info.entryType == Enum.HousingCatalogEntryType.Decor)
        assert(info.itemID == 6948 and info.name == 'Full UI catalog chair')
        assert(info.totalNumStored == 7 and info.totalNumPlaced == 3)
        assert(info.isUniqueTrophy == false)
        for _, field in ipairs({'entryID', 'quantity', 'numPlaced', 'showQuantity', 'dyeSlots'}) do
            assert(rawget(info, field) == nil, 'removed housing wrapper field: '..field)
        end
        info.totalNumStored = 999
        assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Decor, 1001).totalNumStored == 7)
    "#).expect("current entry DTO must not regain removed legacy wrapper fields");
    env.state().borrow_mut().housing.catalog.entries.remove(&id);
    env.exec("assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Decor, 1001) == nil)")
        .expect("removed host entry must not fall back to seeded catalog data");
}
}

prefork_full_ui_case! {
fn blizzard_housing_catalog_category_info_preserves_current_fields(env: &WowLuaEnv) {
    for any_stored_entries in [true, false] {
        env.state().borrow_mut().housing.catalog.categories.insert(101, HousingCatalogCategoryRecord {
            order_index: 4,
            name: Some("Full UI category".into()),
            icon: None,
            subcategory_ids: vec![1001],
            any_stored_entries,
            editor_mode_contexts: vec![],
        });
        env.state().borrow_mut().housing.catalog.subcategories.insert(1001, HousingCatalogSubcategoryRecord {
            order_index: 2,
            parent_category_id: 101,
            name: Some("Full UI subcategory".into()),
            icon: None,
            any_stored_entries,
            editor_mode_contexts: vec![],
        });
        let (category_stored, subcategory_stored): (bool, bool) = env.eval(r#"
            assert(GetCVarBool('loadDeprecationFallbacks'))
            local category = C_HousingCatalog.GetCatalogCategoryInfo(101)
            local subcategory = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
            assert(category and subcategory, 'explicit category records must survive full UI startup')
            assert(category.ID == 101 and category.name == 'Full UI category' and category.orderIndex == 4)
            assert(#category.subcategoryIDs == 1 and category.subcategoryIDs[1] == 1001)
            assert(subcategory.ID == 1001 and subcategory.parentCategoryID == 101)
            assert(subcategory.name == 'Full UI subcategory' and subcategory.orderIndex == 2)
            assert(rawget(category, 'anyOwnedEntries') == nil and rawget(subcategory, 'anyOwnedEntries') == nil)
            return category.anyStoredEntries, subcategory.anyStoredEntries
        "#).expect("current category DTOs must not regain removed legacy wrapper fields");
        assert_eq!(category_stored, any_stored_entries);
        assert_eq!(subcategory_stored, any_stored_entries);
    }
    {
        let mut state = env.state().borrow_mut();
        state.housing.catalog.categories.remove(&101);
        state.housing.catalog.subcategories.remove(&1001);
    }
    env.exec("assert(C_HousingCatalog.GetCatalogCategoryInfo(101) == nil); assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001) == nil)")
        .expect("removed host categories must not fall back to seeded catalog data");
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_housing_catalog_publishes_get_featured_decor_alias(env: &WowLuaEnv) {

    let installed: bool = env
        .eval("return type(C_HousingCatalog.GetFeaturedDecor) == 'function'")
        .expect("GetFeaturedDecor function-installation query should succeed");
    assert!(
        installed,
        "Deprecated_HousingCatalog.lua line 174-180 should publish \
         `C_HousingCatalog.GetFeaturedDecor` as a wrapper around the new \
         `GetFeaturedSmallProducts`. Each result entry must carry a synthesized \
         `entryID` super-ID built from the new `entryVariantID` via \
         `ConvertVariantIDToSuperID` (line 65-71)"
    );

    let entries_have_super_id: bool = env
        .eval(
            "do \
                local results = C_HousingCatalog.GetFeaturedDecor() \
                if type(results) ~= 'table' or #results == 0 then return false end \
                for _, info in ipairs(results) do \
                    if type(info.entryID) ~= 'table' \
                        or info.entryID.entrySubtype ~= Enum.HousingCatalogEntrySubtype.Unowned \
                        or info.entryID.subtypeIdentifier == nil then \
                        return false \
                    end \
                end \
                return true \
            end",
        )
        .expect("GetFeaturedDecor super-ID query should succeed");
    assert!(
        entries_have_super_id,
        "Each GetFeaturedDecor result must carry an entryID super-ID with \
         entrySubtype=Enum.HousingCatalogEntrySubtype.Unowned and \
         subtypeIdentifier=variantIdentifier (per ConvertVariantIDToSuperID at \
         Deprecated_HousingCatalog.lua:65-71)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_housing_catalog_create_catalog_searcher_aliases_legacy_method_names(env: &WowLuaEnv) {

    let aliases_match: bool = env
        .eval(
            "do \
                local searcher = C_HousingCatalog.CreateCatalogSearcher() \
                if type(searcher) ~= 'table' then return false end \
                return searcher.IsOwnedOnlyActive == searcher.IsStoredOnlyActive \
                    and searcher.SetOwnedOnly == searcher.SetStoredOnly \
                    and searcher.ToggleOwnedOnly == searcher.ToggleStoredOnly \
                    and type(searcher.OldGetAllSearchItems) == 'function' \
                    and type(searcher.OldGetCatalogSearchResults) == 'function' \
            end",
        )
        .expect("CreateCatalogSearcher alias query should succeed");
    assert!(
        aliases_match,
        "Deprecated_HousingCatalog.lua line 247-268 wraps \
         `C_HousingCatalog.CreateCatalogSearcher` to attach 5 legacy bindings to each \
         returned searcher: 3 method-rename aliases (IsOwnedOnlyActive=IsStoredOnlyActive, \
         SetOwnedOnly=SetStoredOnly, ToggleOwnedOnly=ToggleStoredOnly — identity-equal so \
         the new and old method names point at the SAME function), and 2 fresh \
         `OldGet*` closures (OldGetAllSearchItems / OldGetCatalogSearchResults) that wrap \
         the new variant-ID-returning methods to mutate variant IDs into super-IDs via \
         `ConvertVariantIDsToEntryIDs` (line 230-235)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_housing_catalog_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_HousingCatalog.lua:37 doesn't bail \
         before any of the in-place namespace overrides + searcher wrapping run. If this \
         CVar flips to false, all C_HousingCatalog / C_HousingBasicMode method overrides \
         are skipped and any legacy housing addon depending on the old field names \
         (entryID compound, quantity, numPlaced, anyOwnedEntries, IsOwnedOnlyActive) \
         silently sees the new-API field names instead"
    );
}
}

#[test]
fn blizzard_deprecated_housing_catalog_directory_is_absent() {
    assert!(
        !blizzard_ui_dir()
            .join("Blizzard_DeprecatedHousingCatalog")
            .exists(),
        "Retail 12.1.0.69497 no longer ships the Blizzard_DeprecatedHousingCatalog directory"
    );
}
