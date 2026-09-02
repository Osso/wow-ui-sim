#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn finder_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HousingHouseFinder")
}

fn finder_toc() -> PathBuf {
    finder_dir().join("Blizzard_HousingHouseFinder.toc")
}

fn load_housing_house_finder(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &finder_toc())
        .expect("Blizzard_HousingHouseFinder should load via explicit Rust loader call");
}

fn assert_mixin_methods(env: &WowLuaEnv, mixin: &str, methods: &[&str], rationale: &str) {
    for method in methods {
        let exists: bool = env
            .eval(&format!("return type({mixin}['{method}']) == 'function'"))
            .unwrap_or_else(|err| panic!("{mixin}.{method} existence query failed: {err}"));
        assert!(exists, "{mixin} must expose `:{method}()` — {rationale}");
    }
}

#[test]
fn blizzard_housing_house_finder_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&finder_dir()).expect("Blizzard_HousingHouseFinder TOC should resolve");
    assert_eq!(
        resolved,
        finder_toc(),
        "Blizzard_HousingHouseFinder ships exactly one bare TOC — retail-only addon resolves \
         via `find_toc_file` fallthrough"
    );
}

#[test]
fn blizzard_housing_house_finder_toc_declares_lod_with_two_dependencies() {
    let toc = TocFile::from_file(&finder_toc()).expect("HousingHouseFinder TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HousingHouseFinder declares `## LoadOnDemand: 1` — pulled via explicit \
         LoadAddOn from PlayerInteractionFrameManager.lua:75 (when the player interacts with a \
         House Finder NPC) and from HousingDashboardHouseInfoContent.lua:240/:310 (the dashboard \
         HouseFinderButton CTA + the no-houses fallback panel)"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_HousingTemplates".to_string(),
            "Blizzard_MapCanvas".to_string(),
        ],
        "Two `## Dependencies:` entries: HousingTemplates (housing atlases, sound kits, \
         C_Housing/C_HousingNeighborhood query surface) plus Blizzard_MapCanvas (provides \
         MapCanvasDataProviderMixin and MapCanvasPinMixin which HouseFinderMapDataProviderMixin / \
         HouseFinderPlotForSalePinMixin / HouseFinderFriendsPlotPinMixin extend via \
         CreateFromMixins for the embedded for-sale/friends map panel)"
    );
}

#[test]
fn blizzard_housing_house_finder_toc_is_retail_only_and_omits_allow_load() {
    let toc = TocFile::from_file(&finder_toc()).expect("HousingHouseFinder TOC should parse");
    let toc_text = std::fs::read_to_string(finder_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Declares `## AllowLoadGameType: standard` — retail-only Midnight feature"
    );
    assert!(!toc.is_game_type_restricted());
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Omits `## AllowLoad:` — LoadOnDemand precludes auto-discovery gating"
    );
    assert!(!toc_text.contains("## DefaultState:"));
    assert!(
        toc.saved_variables().is_empty(),
        "No `## SavedVariables*` — neighborhood list / pin selection state is server-driven via \
         HOUSE_FINDER_NEIGHBORHOODS_RESULT / HOUSE_FINDER_BNET_NEIGHBORHOODS_RESULT / \
         CANCEL_HOUSE_INVITATION_RESULT events plus the C_HousingNeighborhood / C_Housing API \
         surface"
    );
}

#[test]
fn blizzard_housing_house_finder_toc_lists_bootstrap_then_five_files_in_order() {
    let toc = TocFile::from_file(&finder_toc()).expect("HousingHouseFinder TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HousingHouseFinder_Bootstrap.lua".to_string(),
            "HouseFinderMapDataProvider.lua".to_string(),
            "HouseFinderMapDataProvider.xml".to_string(),
            "Blizzard_HousingHouseFinder.lua".to_string(),
            "Blizzard_HousingHouseFinder.xml".to_string(),
            "Blizzard_HousingHouseFinderRegistration.lua".to_string(),
        ],
        "TOC body lists its bootstrap then five source files in published order."
    );
}

#[test]
fn blizzard_housing_house_finder_directory_holds_seven_entries() {
    let dir = finder_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HousingHouseFinder directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        7,
        "Directory ships exactly 7 entries (bootstrap + 5 source files + TOC). Got: {entries:?}"
    );
    assert!(entries.contains(&"Blizzard_HousingHouseFinder.toc".to_string()));
    assert!(entries.contains(&"Blizzard_HousingHouseFinder_Bootstrap.lua".to_string()));
    assert!(entries.contains(&"Blizzard_HousingHouseFinder.lua".to_string()));
    assert!(entries.contains(&"Blizzard_HousingHouseFinder.xml".to_string()));
    assert!(entries.contains(&"Blizzard_HousingHouseFinderRegistration.lua".to_string()));
    assert!(entries.contains(&"HouseFinderMapDataProvider.lua".to_string()));
    assert!(entries.contains(&"HouseFinderMapDataProvider.xml".to_string()));
}

#[test]
fn blizzard_housing_house_finder_excluded_from_all_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
        ScreenKind::Game,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_HousingHouseFinder");
        assert!(
            !discovered,
            "Must NOT appear in {screen:?} auto-discovery — `## LoadOnDemand: 1` excludes the \
             addon from every ScreenKind pass; it is only ever pulled via explicit LoadAddOn \
             from the Player-Interaction-Frame manager (housing NPC) or the Dashboard \
             HouseFinder CTA"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_housing_house_finder(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HousingHouseFinder/")
                || e.contains("Blizzard_HousingHouseFinder\\")
                || e.contains("HouseFinderFrameMixin")
                || e.contains("HouseFinderBNetFriendSearchBoxMixin")
                || e.contains("PlotInfoFrameBackButtonMixin")
                || e.contains("HouseFinderPlotInfoFrameMixin")
                || e.contains("HouseFinderNeighborhoodButtonMixin")
                || e.contains("DeclineInviteButtonMixin")
                || e.contains("HouseFinderMapDataProviderMixin")
                || e.contains("HouseFinderPlotForSalePinMixin")
                || e.contains("HouseFinderFriendsPlotPinMixin")
                || e.contains("SelectedPlotTooltipMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Loading Blizzard_HousingHouseFinder must not emit any addon-specific Lua errors. \
         Got {} errors: {:?}",
        related.len(),
        related
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_is_addon_loaded_via_explicit_lod_call(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingHouseFinder')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_HousingHouseFinder') must return true after explicit \
         LoD load — proves the loader registered the addon name"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_template_dependency_loads_via_game_screen_pass(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let templates_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingTemplates')")
        .expect("HousingTemplates IsAddOnLoaded query should succeed");
    assert!(
        templates_loaded,
        "Blizzard_HousingTemplates (the first declared dep) must auto-load via the Game-screen \
         discovery pass before the explicit HouseFinder LoD load runs — it is NOT \
         LoadOnDemand so the auto-discovery sweep includes it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_map_canvas_dependency_is_loaded(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let map_canvas_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MapCanvas')")
        .expect("MapCanvas IsAddOnLoaded query should succeed");
    assert!(
        map_canvas_loaded,
        "Blizzard_MapCanvas (the second declared dep) must be loaded — it provides \
         MapCanvasDataProviderMixin and MapCanvasPinMixin which 3 HouseFinder mixins extend"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_publishes_house_finder_frame_global(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let frame_type: String = env
        .eval("return type(HouseFinderFrame)")
        .expect("HouseFinderFrame type query should succeed");
    assert_eq!(
        frame_type, "table",
        "HouseFinderFrame must publish as the sole non-virtual `_G` frame of the addon — the \
         XML declares it at file scope (line 103) inheriting PortraitFrameTemplate as the \
         left-area panel, while every other XML element is `virtual=\"true\"`"
    );
    let frame_name: String = env
        .eval("return HouseFinderFrame:GetName()")
        .expect("HouseFinderFrame:GetName query should succeed");
    assert_eq!(frame_name, "HouseFinderFrame");
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_companion_tooltip_frame_publishes_to_globals(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let tooltip_type: String = env
        .eval("return type(HouseFinderHighlightedPlotTooltip)")
        .expect("HouseFinderHighlightedPlotTooltip type query should succeed");
    assert_eq!(
        tooltip_type, "table",
        "HouseFinderHighlightedPlotTooltip must publish as a `_G` frame — the data-provider XML \
         declares this single non-virtual companion frame at file scope (TOOLTIP strata, parented \
         to UIParent, hidden) inheriting SelectedPlotTooltipTemplate; the rest of the \
         data-provider XML is `virtual=\"true\"`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_virtual_templates_stay_nil_in_globals(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    for template in [
        "HouseFinderNeighborhoodButtonTemplate",
        "SelectedPlotTooltipTemplate",
        "HouseFinderPlotForSalePinTemplate",
        "HouseFinderFriendsPlotPinTemplate",
    ] {
        let value_type: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .expect("template _G lookup should succeed");
        assert_eq!(
            value_type, "nil",
            "{template} is `virtual=\"true\"` so it must NOT publish to `_G` — proves the loader \
             honors the virtual flag"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_frame_mixin_publishes_eighteen_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "HouseFinderFrameMixin",
        &[
            "OnLoad",
            "OnRefreshClicked",
            "PopulateNeighborhoodList",
            "PopulateBNetNeighborhoodList",
            "UpdateSubdivisionDropdown",
            "SelectSubdivision",
            "SelectNeighborhood",
            "OnEvent",
            "OnShow",
            "OnHide",
            "SelectPlot",
            "ShowNeighborhoodList",
            "TryBnetFriendSearch",
            "SearchBnetFriendNeighborhoods",
            "ClearBnetFriendSearch",
            "SetPendingNeighborhoodInviteToDecline",
        ],
        "HouseFinderFrameMixin owns 16 methods driving the umbrella panel: OnLoad / OnEvent / \
         OnShow / OnHide lifecycle, OnRefreshClicked (refresh the neighborhood list), \
         PopulateNeighborhoodList / PopulateBNetNeighborhoodList (build the two list views from \
         server-supplied vectors), UpdateSubdivisionDropdown / SelectSubdivision (filter the \
         list by guild subdivision), SelectNeighborhood / SelectPlot (focus the right-side plot \
         info panel from a list click or map pin click), ShowNeighborhoodList (return to list \
         view from PlotInfo view), TryBnetFriendSearch / SearchBnetFriendNeighborhoods / \
         ClearBnetFriendSearch (BNet-friend-name search flow), \
         SetPendingNeighborhoodInviteToDecline (StaticPopup decline-invite handshake)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_bnet_search_box_mixin_publishes_eleven_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "HouseFinderBNetFriendSearchBoxMixin",
        &[
            "OnLoad",
            "OnClearButtonClicked",
            "OnEnterPressed",
            "OnEscapePressed",
            "OnTextChanged",
            "OnEditFocusGained",
            "OnEditFocusLost",
            "RefreshSearch",
            "UpdateState",
            "HasStickyFocus",
            "GetSearchDisplayText",
            "GetBnetID",
        ],
        "HouseFinderBNetFriendSearchBoxMixin owns 12 methods (full EditBox lifecycle plus \
         RefreshSearch / UpdateState / HasStickyFocus / GetSearchDisplayText / GetBnetID utility \
         queries) used by the BNet-friend autocomplete search edit box",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_plot_info_back_button_mixin_publishes_four_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "PlotInfoFrameBackButtonMixin",
        &["OnEnter", "OnLeave", "OnClick", "UpdateSize"],
        "PlotInfoFrameBackButtonMixin owns 4 methods (OnEnter/OnLeave tooltip wiring, OnClick \
         returns to the neighborhood list, UpdateSize lays out the chevron)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_plot_info_frame_mixin_publishes_six_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "HouseFinderPlotInfoFrameMixin",
        &[
            "OnLoad",
            "OnEvent",
            "OnShow",
            "OnHide",
            "Init",
            "OnVisitClicked",
        ],
        "HouseFinderPlotInfoFrameMixin owns 6 methods (lifecycle quartet + Init populates the \
         right-side plot detail panel from plotInfo+neighborhoodInfo, OnVisitClicked routes \
         through C_Housing visit-plot RPCs)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_neighborhood_button_mixin_publishes_ten_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "HouseFinderNeighborhoodButtonMixin",
        &[
            "Init",
            "OnEnter",
            "OnLeave",
            "OnClick",
            "OnMouseUp",
            "ReportNeighborhood",
            "Select",
            "Deselect",
            "TryCancelInvite",
            "FailCancelInvite",
            "UpdateGuildIcon",
        ],
        "HouseFinderNeighborhoodButtonMixin owns 11 methods on the row button shared by the \
         standard list and the BNet search results: Init / OnEnter / OnLeave / OnClick / \
         OnMouseUp interaction, ReportNeighborhood (right-click report), Select / Deselect \
         (visual state), TryCancelInvite / FailCancelInvite (decline-invite request handshake), \
         UpdateGuildIcon",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_decline_invite_button_mixin_publishes_five_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "DeclineInviteButtonMixin",
        &[
            "SetNeighborhoodButton",
            "OnEnter",
            "OnLeave",
            "OnClick",
            "OnMouseDown",
            "OnMouseUp",
        ],
        "DeclineInviteButtonMixin owns 6 methods including the SetNeighborhoodButton parent-link \
         setter and the StaticPopup_Show OnClick that fires HOUSING_HOUSEFINDER_CANCEL_INVITATION",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_map_data_provider_mixin_extends_map_canvas_base(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let extends: bool = env
        .eval(
            "return type(HouseFinderMapDataProviderMixin) == 'table' \
             and type(MapCanvasDataProviderMixin) == 'table' \
             and HouseFinderMapDataProviderMixin.OnAdded == MapCanvasDataProviderMixin.OnAdded",
        )
        .expect(
            "HouseFinderMapDataProviderMixin extends-MapCanvasDataProviderMixin query should \
         succeed",
        );
    assert!(
        extends,
        "HouseFinderMapDataProviderMixin must inherit MapCanvasDataProviderMixin via \
         CreateFromMixins(MapCanvasDataProviderMixin) — confirms Blizzard_MapCanvas dep is loaded \
         and the mixin chain is preserved through the loader"
    );
    assert_mixin_methods(
        &env,
        "HouseFinderMapDataProviderMixin",
        &[
            "SetHouseMapData",
            "SetSelectedPin",
            "OnEvent",
            "RemoveAllData",
            "RefreshAllData",
        ],
        "HouseFinderMapDataProviderMixin owns 5 methods on top of the inherited \
         MapCanvasDataProviderMixin chain: SetHouseMapData (server-supplied plot list), \
         SetSelectedPin (track which pin is highlighted), OnEvent / RemoveAllData / \
         RefreshAllData driving the embedded map panel",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_for_sale_pin_mixin_publishes_nine_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "HouseFinderPlotForSalePinMixin",
        &[
            "OnLoad",
            "OnAcquired",
            "Refresh",
            "OnMouseEnter",
            "OnMouseLeave",
            "OnMouseDownAction",
            "OnMouseUpAction",
            "OnMouseClickAction",
            "StartGlow",
            "StopGlow",
        ],
        "HouseFinderPlotForSalePinMixin owns 10 methods covering the for-sale plot pin: \
         OnLoad/OnAcquired/Refresh lifecycle, OnMouseEnter/Leave/Down/Up/Click input, plus \
         StartGlow/StopGlow tying the pin's animation playback to the highlight state",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_friends_pin_mixin_publishes_five_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "HouseFinderFriendsPlotPinMixin",
        &[
            "OnLoad",
            "OnAcquired",
            "Refresh",
            "OnMouseEnter",
            "OnMouseLeave",
        ],
        "HouseFinderFriendsPlotPinMixin owns 5 methods (OnLoad/OnAcquired/Refresh + \
         OnMouseEnter/Leave) — friends pins are tooltip-only and do not own a click action",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_selected_plot_tooltip_mixin_publishes_two_methods(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    assert_mixin_methods(
        &env,
        "SelectedPlotTooltipMixin",
        &["OnLoad", "SetPlotInfo"],
        "SelectedPlotTooltipMixin owns 2 methods — OnLoad (cache the inherited tooltip-bordered \
         frame) plus SetPlotInfo (branch on Enum.HousingPlotOwnerType.None vs Friend to swap \
         atlases / header text / footer color and Show or Hide the price MoneyFrame)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_publishes_static_popup_for_cancel_invitation(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let exists: bool = env
        .eval("return type(StaticPopupDialogs['HOUSING_HOUSEFINDER_CANCEL_INVITATION']) == 'table'")
        .expect("StaticPopupDialogs lookup should succeed");
    assert!(
        exists,
        "StaticPopupDialogs['HOUSING_HOUSEFINDER_CANCEL_INVITATION'] must register as a table — \
         the .lua file declares this single popup descriptor at file scope (line 678) wrapping \
         the decline-pending-invitation confirmation flow tied to \
         DeclineInviteButtonMixin:OnClick"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_registers_ui_panel_with_left_area_and_pushable_zero(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    let area: String = env
        .eval("return UIPanelWindows['HouseFinderFrame'].area")
        .expect("UIPanelWindows area lookup should succeed");
    let pushable: i64 = env
        .eval("return UIPanelWindows['HouseFinderFrame'].pushable")
        .expect("UIPanelWindows pushable lookup should succeed");
    assert_eq!(
        area, "left",
        "Blizzard_HousingHouseFinderRegistration.lua must register HouseFinderFrame with \
         area=\"left\" — the panel docks to the left edge alongside other major housing panels"
    );
    assert_eq!(
        pushable, 0,
        "Blizzard_HousingHouseFinderRegistration.lua must register HouseFinderFrame with \
         pushable=0 — lowest displacement priority, gets pushed by every higher-rank panel"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_house_finder_frame_publishes_top_level_parent_keys(env: &WowLuaEnv) {
    load_housing_house_finder(env);
    for key in ["NeighborhoodListFrame", "HouseFinderMapCanvasFrame"] {
        let key_type: String = env
            .eval(&format!("return type(HouseFinderFrame.{key})"))
            .expect("HouseFinderFrame parentKey lookup should succeed");
        assert_eq!(
            key_type, "table",
            "HouseFinderFrame.{key} must publish as a child frame — the XML declares it as a \
             top-level parentKey under HouseFinderFrame's <Frames> element"
        );
    }
    let nested_search_box: String = env
        .eval("return type(HouseFinderFrame.NeighborhoodListFrame.BNetFriendSearchBox)")
        .expect("nested BNetFriendSearchBox lookup should succeed");
    assert_eq!(
        nested_search_box, "table",
        "HouseFinderFrame.NeighborhoodListFrame.BNetFriendSearchBox must publish as a nested \
         child — the XML declares the EditBox under NeighborhoodListFrame's <Frames> element \
         (not at HouseFinderFrame's top-level <Frames>)"
    );
}
}
