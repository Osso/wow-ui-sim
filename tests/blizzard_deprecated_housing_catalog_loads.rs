#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
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
fn blizzard_deprecated_housing_catalog_wraps_get_catalog_entry_info_with_legacy_fields(env: &WowLuaEnv) {

    let has_legacy_fields: bool = env
        .eval(
            "do \
                local info = C_HousingCatalog.GetCatalogEntryInfoByRecordID(1, 1001, true) \
                if not info then return false end \
                return type(info.entryID) == 'table' \
                    and info.entryID.entrySubtype == Enum.HousingCatalogEntrySubtype.Unowned \
                    and info.entryID.subtypeIdentifier == 0 \
                    and info.entryID.variantIdentifier == 0 \
                    and info.quantity == info.totalNumStored \
                    and info.numPlaced == info.totalNumPlaced \
                    and type(info.showQuantity) == 'boolean' \
                    and type(info.dyeSlots) == 'table' \
            end",
        )
        .expect("GetCatalogEntryInfoByRecordID legacy-fields query should succeed");
    assert!(
        has_legacy_fields,
        "Deprecated_HousingCatalog.lua line 130-133 wraps `C_HousingCatalog.\
         GetCatalogEntryInfoByRecordID` so the returned info table carries all 5 \
         backward-compat fields populated by `AddBackwardCompatEntryInfoFields` at \
         line 76-91: `entryID` (compound table with entrySubtype=Enum.\
         HousingCatalogEntrySubtype.Unowned + subtypeIdentifier=0 + variantIdentifier=0), \
         `quantity` (mirroring totalNumStored), `numPlaced` (mirroring totalNumPlaced), \
         `showQuantity` (boolean), `dyeSlots` (empty table)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_housing_catalog_wraps_category_info_with_legacy_field(env: &WowLuaEnv) {

    let any_owned_mirrors: bool = env
        .eval(
            "do \
                local info = C_HousingCatalog.GetCatalogCategoryInfo(101) \
                if not info then return false end \
                return info.anyOwnedEntries == info.anyStoredEntries \
            end",
        )
        .expect("GetCatalogCategoryInfo legacy-field query should succeed");
    assert!(
        any_owned_mirrors,
        "Deprecated_HousingCatalog.lua line 184-187 wraps `C_HousingCatalog.\
         GetCatalogCategoryInfo` to mirror the new `anyStoredEntries` field onto the legacy \
         `anyOwnedEntries` field via `AddBackwardCompatCategoryInfoFields` (line 96-101)"
    );

    let subcategory_any_owned_mirrors: bool = env
        .eval(
            "do \
                local info = C_HousingCatalog.GetCatalogSubcategoryInfo(1001) \
                if not info then return false end \
                return info.anyOwnedEntries == info.anyStoredEntries \
            end",
        )
        .expect("GetCatalogSubcategoryInfo legacy-field query should succeed");
    assert!(
        subcategory_any_owned_mirrors,
        "Deprecated_HousingCatalog.lua line 191-194 wraps `C_HousingCatalog.\
         GetCatalogSubcategoryInfo` similarly via `AddBackwardCompatSubcategoryInfoFields` \
         (line 106-111)"
    );
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
