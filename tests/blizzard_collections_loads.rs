#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn collections_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Collections/Blizzard_Collections_Mainline.toc")
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
fn blizzard_collections_is_load_on_demand_not_in_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);

    let auto_loaded = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Collections");
    assert!(
        !auto_loaded,
        "Blizzard_Collections is `## LoadOnDemand: 1` and must NOT appear in Game-screen \
         auto-discovery (it is loaded explicitly when the player opens the journal)"
    );
}

const KNOWN_GET_NUM_EXPANSIONS_GAP: &str =
    "attempt to call global 'GetNumExpansions' (a nil value)";

fn unexpected_collections_errors(env: &WowLuaEnv) -> Vec<String> {
    env.state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_Collections")
                && !message.contains(KNOWN_GET_NUM_EXPANSIONS_GAP)
        })
        .cloned()
        .collect()
}

prefork_full_ui_case! {
fn blizzard_collections_loads_via_explicit_load(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &collections_toc())
        .expect("Blizzard_Collections should load via Rust loader");

    let collections_errors = unexpected_collections_errors(&env);
    assert!(
        collections_errors.is_empty(),
        "Blizzard_Collections emitted unexpected Lua errors during load:\n  {}",
        collections_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_collections_top_level_frames_are_defined(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &collections_toc()).expect("Blizzard_Collections should load");

    let frames_present: bool = env
        .eval(
            "return CollectionsJournal ~= nil \
                and MountJournal ~= nil \
                and PetJournal ~= nil \
                and ToyBox ~= nil \
                and HeirloomsJournal ~= nil \
                and WardrobeCollectionFrame ~= nil \
                and WarbandSceneJournal ~= nil",
        )
        .expect("frame query should succeed");
    assert!(
        frames_present,
        "All six Collections journal frames (CollectionsJournal, MountJournal, PetJournal, \
         ToyBox, HeirloomsJournal, WardrobeCollectionFrame, WarbandSceneJournal) should be \
         defined after load"
    );

    let tabs_present: bool = env
        .eval(
            "return CollectionsJournal.MountsTab ~= nil \
                and CollectionsJournal.PetsTab ~= nil \
                and CollectionsJournal.ToysTab ~= nil \
                and CollectionsJournal.HeirloomsTab ~= nil \
                and CollectionsJournal.WardrobeTab ~= nil \
                and CollectionsJournal.WarbandScenesTab ~= nil",
        )
        .expect("tab query should succeed");
    assert!(
        tabs_present,
        "CollectionsJournal should expose its six tab parentKeys after XML load"
    );
}
}

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
prefork_full_ui_case! {
fn retail_original_mount_flyout_consumer_does_not_suppress_missing_popup_error(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &collections_toc())
        .expect("Blizzard_Collections should load before its Retail consumer runs");
    env.exec("MountJournal.ToggleDynamicFlightFlyoutButton.popup = nil")
        .expect("fixture should remove only the flyout popup reference");

    let error = env
        .exec("MountJournal.ToggleDynamicFlightFlyoutButton:OnPopupToggled()")
        .expect_err("the original Blizzard consumer dereferences its required popup child");
    assert!(
        error.to_string().contains("popup"),
        "the original consumer should report its missing popup access: {error}"
    );
}
}

prefork_full_ui_case! {
fn warband_scene_footer_controls_do_not_overlap(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &collections_toc()).expect("Blizzard_Collections should load");

    env.exec(
        r#"
        CollectionsJournal:Show()
        CollectionsJournal_SetTab(CollectionsJournal, 6)
        "#,
    )
    .expect("warband scene tab should open");
    env.fire_on_update(0.016)
        .expect("layout dirty OnUpdate should run");

    let layout: (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        String,
        f64,
        String,
        f64,
        bool,
        String,
    ) = env
        .eval(
            r#"
            local controls = WarbandSceneJournal.IconsFrame.Icons.Controls
            local showOwned = controls.ShowOwned
            local paging = controls.PagingControls
            local checkbox = showOwned.Checkbox
            local label = showOwned.Text

            local showPoint, _, _, showX = showOwned:GetPoint(1)
            local pagingPoint, _, _, pagingX = paging:GetPoint(1)
            return label:GetWidth(),
                   showOwned:GetWidth(),
                   checkbox:GetWidth(),
                   label.anchorSpacing or 0,
                   showOwned:GetLeft(),
                   showOwned:GetRight(),
                   paging:GetLeft(),
                   paging.PageText:GetRight(),
                   paging.PrevPageButton:GetLeft(),
                   showPoint or "",
                   showX or 0,
                   pagingPoint or "",
                   pagingX or 0,
                   controls:IsDirty() == true,
                   type(controls:GetScript("OnUpdate"))
            "#,
        )
        .expect("warband scene footer geometry query should succeed");

    let (
        label_width,
        show_owned_width,
        checkbox_width,
        anchor_spacing,
        show_owned_left,
        show_owned_right,
        paging_left,
        page_text_right,
        prev_button_left,
        show_owned_point,
        show_owned_offset,
        paging_point,
        paging_offset,
        controls_dirty,
        controls_on_update_type,
    ) = layout;

    assert!(
        label_width > 80.0,
        "Show Collected Only label should report its intrinsic text width, got {label_width}"
    );
    assert!(
        show_owned_width >= checkbox_width + label_width + anchor_spacing - 0.5,
        "ShowOwned width should include checkbox + label + spacing, got \
         showOwned={show_owned_width}, checkbox={checkbox_width}, label={label_width}, \
         spacing={anchor_spacing}"
    );
    assert!(
        paging_left >= show_owned_right + 20.0,
        "Paging controls should sit after ShowOwned with layout spacing, got \
         showOwnedLeft={show_owned_left}, showOwnedRight={show_owned_right}, \
         pagingLeft={paging_left}, showOwnedPoint={show_owned_point}, \
         showOwnedOffset={show_owned_offset}, pagingPoint={paging_point}, \
         pagingOffset={paging_offset}, controlsDirty={controls_dirty}, \
         controlsOnUpdateType={controls_on_update_type}"
    );
    assert!(
        prev_button_left >= page_text_right + 4.0,
        "Paging arrow should be laid out after page text, got \
         pageTextRight={page_text_right}, prevButtonLeft={prev_button_left}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_collections_mixins_are_defined(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &collections_toc()).expect("Blizzard_Collections should load");

    let mixins_present: bool = env
        .eval(
            "return type(HeirloomsMixin) == 'table' \
                and type(WarbandSceneJounalMixin) == 'table' \
                and type(MountEquipmentButtonMixin) == 'table' \
                and type(SuppressedMountEquipmentButtonMixin) == 'table' \
                and type(MountJournalSummonRandomFavoriteSpellFrameMixin) == 'table'",
        )
        .expect("mixin query should succeed");
    assert!(
        mixins_present,
        "Top-level Collections mixins should be populated after load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_collections_journal_helpers_are_defined(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &collections_toc()).expect("Blizzard_Collections should load");

    let helpers_present: bool = env
        .eval(
            "return type(CollectionsJournal_SetTab) == 'function' \
                and type(CollectionsJournal_GetTab) == 'function' \
                and type(CollectionsJournal_ValidateTab) == 'function' \
                and type(CollectionsJournal_UpdateSelectedTab) == 'function' \
                and type(CollectionsJournal_OnShow) == 'function' \
                and type(CollectionsJournal_OnHide) == 'function'",
        )
        .expect("helper query should succeed");
    assert!(
        helpers_present,
        "Six top-level CollectionsJournal helper functions should be defined after load"
    );
}
}

// Regression: opening the Wardrobe (Appearances) tab must populate the
// items collection with at least one appearance for the active slot.
// Earlier this returned 0 because `IsUnitModelReadyForUI`,
// `SetUseTransmogSkin`, `IsSlotAllowed`, and friends were missing — the
// `ChangeModelsSlot`/`SetActiveCategory` chain bailed out before
// `RefreshVisualsList` ran.
prefork_full_ui_case! {
fn wardrobe_appearances_panel_populates_for_head_slot(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &collections_toc()).expect("Blizzard_Collections should load");

    env.eval::<()>("CollectionsJournal:Show(); CollectionsJournal_SetTab(CollectionsJournal, 5)")
        .expect("opening the Appearances tab should not error");

    let active_category: f64 = env
        .eval("return WardrobeCollectionFrame.ItemsCollectionFrame.activeCategory or -1")
        .expect("activeCategory query should succeed");
    assert!(
        active_category > 0.0,
        "ItemsCollectionFrame.activeCategory should be set (>0) after opening the wardrobe, got {active_category}"
    );

    let filtered_count: f64 = env
        .eval("return #(WardrobeCollectionFrame.ItemsCollectionFrame.filteredVisualsList or {})")
        .expect("filteredVisualsList length query should succeed");
    assert!(
        filtered_count > 0.0,
        "filteredVisualsList should contain at least one appearance for the default head slot, got {filtered_count}"
    );

    let first_visible: bool = env
        .eval(
            "local m = WardrobeCollectionFrame.ItemsCollectionFrame.Models \
             return m and m[1] and m[1]:IsShown() or false",
        )
        .expect("first model query should succeed");
    assert!(
        first_visible,
        "First appearance tile (Models[1]) should be visible after the wardrobe populates"
    );
}
}

prefork_full_ui_case! {
fn wardrobe_appearances_filter_dropdown_has_rows(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &collections_toc()).expect("Blizzard_Collections should load");

    let result: String = env
        .eval(
            r#"
            CollectionsJournal:Show()
            CollectionsJournal_SetTab(CollectionsJournal, 5)

            local dropdown = WardrobeCollectionFrame and WardrobeCollectionFrame.FilterButton
            if dropdown == nil then
                return "missing_dropdown"
            end

            dropdown:Show()
            dropdown:OpenMenu()

            local description = dropdown:GetMenuDescription()
            if description == nil or not description:HasElements() then
                return "empty_filter"
            end

            dropdown:CloseMenu()

            local classDropdown = WardrobeCollectionFrame.ClassDropdown
            if classDropdown == nil then
                return "missing_class_dropdown"
            end
            classDropdown:Show()
            classDropdown:OpenMenu()

            local classDescription = classDropdown:GetMenuDescription()
            if classDescription == nil or not classDescription:HasElements() then
                return "empty_class"
            end

            return "ok"
            "#,
        )
        .expect("opening the wardrobe filter dropdown should not error");

    assert_eq!(
        result, "ok",
        "Appearances filter and class dropdowns should expose menu rows"
    );
}
}
