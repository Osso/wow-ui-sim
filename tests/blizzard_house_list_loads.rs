#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const SUPPLY_NEIGHBORHOODS: &str = r#"
A_Admin.SetHouseFinderNeighborhoods({
    { neighborhoodGUID="Neighborhood-A", neighborhoodName="Amber Vale", neighborhoodType=Enum.NeighborhoodType.Public,
      neighborhoodOwnerType=Enum.NeighborhoodOwnerType.None, suggestionReason=Enum.HouseFinderSuggestionReason.None,
      uiMapID=41, textureSuffix="amber", factionMatches=true,
      plots={{ mapPosition={x=0.2,y=0.3}, ownerType=Enum.HousingPlotOwnerType.None, plotID=17, plotCost=1250 }} },
    { neighborhoodGUID="Neighborhood-B", neighborhoodName="Cobalt Reach", neighborhoodType=Enum.NeighborhoodType.Public,
      neighborhoodOwnerType=Enum.NeighborhoodOwnerType.None, suggestionReason=Enum.HouseFinderSuggestionReason.None,
      uiMapID=82, textureSuffix="cobalt", factionMatches=false,
      plots={{ mapPosition={x=0.7,y=0.8}, ownerType=Enum.HousingPlotOwnerType.Friend, plotID=29, ownerName="Mira" }} },
})
"#;

const SELECT_SECOND_NEIGHBORHOOD: &str = r#"
local buttons = HouseFinderFrame.neighborhoodButtonPool:EnumerateActive()
local first = buttons()
local second = buttons()
if not first or not second then return "neighborhoods_not_rendered" end
local rendered = first.NeighborhoodName:GetText() .. "/" .. second.NeighborhoodName:GetText()
local firstMap = HouseFinderFrame.mapID
second:Click()
return rendered .. ";" .. tostring(firstMap) .. "/" .. tostring(HouseFinderFrame.mapID)
"#;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn house_list_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HouseList")
}

fn house_list_toc() -> PathBuf {
    house_list_dir().join("Blizzard_HouseList.toc")
}

fn house_finder_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HousingHouseFinder/Blizzard_HousingHouseFinder.toc")
}

fn load_house_list(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &house_list_toc())
        .expect("Blizzard_HouseList should load via explicit Rust loader call");
}

#[test]
fn blizzard_house_list_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&house_list_dir()).expect("Blizzard_HouseList TOC should resolve");
    assert_eq!(
        resolved,
        house_list_toc(),
        "Blizzard_HouseList ships exactly one bare TOC (`Blizzard_HouseList.toc`) — no flavor \
         variants. The view-others-houses social menu only ships on retail (`## \
         AllowLoadGameType: standard`) and uses the bare TOC suffix that `find_toc_file` \
         (src/loader/mod.rs:65) falls through to"
    );
}

#[test]
fn blizzard_house_list_toc_declares_lod_with_single_dependency() {
    let toc = TocFile::from_file(&house_list_toc()).expect("HouseList TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HouseList declares `## LoadOnDemand: 1` — the social view-houses dialog only \
         loads when a player triggers the unit-popup `C_AddOns.LoadAddOn(\"Blizzard_HouseList\")` \
         path (UnitPopupSharedButtonMixins.lua:3519)"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_HouseList does not declare `## LoadFirst: 1` — LoadOnDemand precludes any \
         load-order priority"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_HouseList does not declare `## UseSecureEnvironment` — runs in the standard \
         Lua environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_HousingTemplates".to_string()],
        "Blizzard_HouseList declares exactly one `## Dependencies:` entry — \
         Blizzard_HousingTemplates provides the housing-themed atlas references \
         (`housing-basic-container`, `housing-basic-container-woodheader`, \
         `housing-decorative-foliage-left`, `house-list-container-open/closed`, \
         `house-list-open-divider`) and the shared C_Housing API surface needed by the entry \
         template's VisitHouse click handler"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HouseList declares NO `## SavedVariables*` — the displayed list is fetched \
         live via `C_Housing.GetOthersOwnedHouses` in `InitWithContextData` and refreshed by the \
         `VIEW_HOUSES_LIST_RECIEVED` event, so no per-installation persistence is needed"
    );
}

#[test]
fn blizzard_house_list_toc_is_retail_only_and_omits_allow_load() {
    let toc = TocFile::from_file(&house_list_toc()).expect("HouseList TOC should parse");
    let toc_text = std::fs::read_to_string(house_list_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Blizzard_HouseList declares `## AllowLoadGameType: standard` — the housing social \
         dialog is a Midnight expansion feature that only ships on retail. \
         `is_game_type_restricted()` (src/toc.rs:294) treats `standard` and `mainline` as the \
         unrestricted retail flavor, so this addon is NOT considered game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HouseList must NOT be game-type restricted — `## AllowLoadGameType: standard` \
         matches the retail flavor that the simulator runs as"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_HouseList omits `## AllowLoad:` — LoadOnDemand precludes auto-discovery \
         gating, so the AllowLoad value would be inert. The addon is pulled exclusively via the \
         Lua-side LoadAddOn(\"Blizzard_HouseList\") path"
    );
    assert!(
        !toc_text.contains("## DefaultState:"),
        "Blizzard_HouseList omits `## DefaultState:` — relies on the loader's implicit-enabled \
         default for Blizzard prefix LoD addons"
    );
}

#[test]
fn blizzard_house_list_toc_lists_three_files() {
    let toc = TocFile::from_file(&house_list_toc()).expect("HouseList TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HouseList.lua".to_string(),
            "Blizzard_HouseList.xml".to_string(),
            "Blizzard_HouseListRegistration.lua".to_string(),
        ],
        "Blizzard_HouseList TOC body lists exactly 3 source files in this exact order: \
         Blizzard_HouseList.lua (publishes HouseListFrameMixin + HouseEntryTemplateMixin and the \
         four entry-height constants), Blizzard_HouseList.xml (HouseEntryTemplate virtual \
         EventButton + the HouseListFrame named Frame instance), \
         Blizzard_HouseListRegistration.lua (calls RegisterUIPanel(HouseListFrame, \
         {{area=\"left\", pushable=1}}) — must run AFTER the XML instantiates HouseListFrame)"
    );
}

#[test]
fn blizzard_house_list_directory_holds_four_entries() {
    let dir = house_list_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HouseList directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        4,
        "Blizzard_HouseList directory ships exactly 4 entries: 3 source files referenced by the \
         TOC + 1 TOC file. No flavor subdirectory and no Localization.lua — the strings \
         (VIEW_HOUSES_TITLE, HOUSING_DASHBOARD_*_LABEL, HOUSING_PLOT_NUMBER, \
         VIEW_HOUSES_VISIT_BUTTON, VIEW_HOUSES_NO_HOUSES) are pulled from the global locale \
         table maintained by the housing dependency chain. Got: {entries:?}"
    );
    assert!(
        entries.contains(&"Blizzard_HouseList.toc".to_string()),
        "Blizzard_HouseList directory must contain the bare TOC file"
    );
    assert!(
        entries.contains(&"Blizzard_HouseListRegistration.lua".to_string()),
        "Blizzard_HouseList directory must contain the Registration tail file (8 lines, runs \
         RegisterUIPanel after the XML instantiates HouseListFrame)"
    );
}

#[test]
fn blizzard_house_list_excluded_from_all_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons.iter().any(|(name, _)| name == "Blizzard_HouseList");
        assert!(
            !discovered,
            "Blizzard_HouseList MUST NOT appear in {screen:?} auto-discovery — \
             `## LoadOnDemand: 1` keeps it out of every screen pass. The only consumer \
             (Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua:3519) calls \
             `C_AddOns.LoadAddOn(\"Blizzard_HouseList\")` from a unit-popup click handler — \
             never via `## RequiredDep:`, so the LoD-pull promotion path in \
             `pull_required_lod_addons` (src/loader/mod.rs:357) does not escalate HouseList onto \
             any auto-discovery pass"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_house_list_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_house_list(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HouseList/")
                || e.contains("Blizzard_HouseList\\")
                || e.contains("HouseListFrameMixin")
                || e.contains("HouseEntryTemplateMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_HouseList emitted addon-specific Lua errors during explicit LoD load:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_list_is_addon_loaded_returns_true_after_explicit_lod_load(env: &WowLuaEnv) {
    load_house_list(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HouseList')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After explicit `load_addon` of Blizzard_HouseList.toc following Game-screen \
         auto-discovery (which loads the single Blizzard_HousingTemplates dep but skips \
         HouseList itself due to LoadOnDemand), `C_AddOns.IsAddOnLoaded('Blizzard_HouseList')` \
         should return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_list_publishes_house_list_frame_global(env: &WowLuaEnv) {
    load_house_list(env);

    let exists: bool = env
        .eval(
            "local f = _G['HouseListFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("HouseListFrame global lookup should succeed");
    assert!(
        exists,
        "After LoD load, `HouseListFrame` should publish as a global frame instance — \
         Blizzard_HouseList.xml line 87 declares `<Frame name=\"HouseListFrame\" \
         mixin=\"HouseListFrameMixin\" toplevel=\"true\" parent=\"UIParent\" movable=\"true\" \
         enableMouse=\"true\" hidden=\"true\">` and the Registration.lua tail then calls \
         `RegisterUIPanel(HouseListFrame, {{area=\"left\", pushable=1}})` to register the frame \
         as a left-area pushable UI panel"
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_list_frame_mixin_publishes_nine_methods(env: &WowLuaEnv) {
    load_house_list(env);

    for method in [
        "OnLoad",
        "InitWithContextData",
        "UpdateHeight",
        "SetSelectedHouse",
        "OnEvent",
        "OnShow",
        "OnHide",
        "OnHouseListUpdated",
        "SelectedFirstHouse",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HouseListFrameMixin['{method}']) == 'function'"
            ))
            .expect("HouseListFrameMixin method existence query should succeed");
        assert!(
            exists,
            "HouseListFrameMixin must expose `:{method}()` — the mixin drives the social \
             view-houses dialog: OnLoad wires the WowScrollBoxList view + selection behavior + \
             resizable-children behavior; InitWithContextData (called by the unit-popup handler) \
             sets the title from VIEW_HOUSES_TITLE + name, kicks off \
             C_Housing.GetOthersOwnedHouses, shows the loading spinner; UpdateHeight clamps the \
             frame between HOUSE_LIST_MIN_HEIGHT (200) and HOUSE_LIST_MAX_HEIGHT (350); \
             SetSelectedHouse is the public selection hook (intentionally a no-op stub for \
             external override); OnEvent dispatches VIEW_HOUSES_LIST_RECIEVED [sic] to \
             OnHouseListUpdated; OnShow / OnHide register/unregister the event and play the \
             social-menu open/close sounds; OnHouseListUpdated rebuilds the data provider and \
             updates height; SelectedFirstHouse selects the first entry in the new data provider"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_entry_template_mixin_publishes_seven_methods(env: &WowLuaEnv) {
    load_house_list(env);

    for method in [
        "Init",
        "OnVisitHouseClicked",
        "SetSelected",
        "Expand",
        "Collapse",
        "OnClick",
        "UpdatePlusMinusTexture",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HouseEntryTemplateMixin['{method}']) == 'function'"
            ))
            .expect("HouseEntryTemplateMixin method existence query should succeed");
        assert!(
            exists,
            "HouseEntryTemplateMixin must expose `:{method}()` — the per-row mixin drives each \
             house entry in the scroll box: Init copies houseInfo into self + populates the four \
             label FontStrings + wires the VisitHouseButton OnClick via GenerateClosure + sets \
             the initial Expand/Collapse state from the selection behavior; OnVisitHouseClicked \
             plays the visit-house sound + calls C_Housing.VisitHouse(neighborhoodGUID, \
             houseGUID, plotID); SetSelected forwards to Expand/Collapse; Expand sets height to \
             HOUSE_ENTRY_EXPANDED_HEIGHT (145) + swaps to `house-list-container-open` atlas + \
             shows the 8-element expandedDetails group; Collapse mirrors with \
             HOUSE_ENTRY_COLLAPSED_HEIGHT (40) + `house-list-container-closed` atlas; OnClick \
             toggles selection via houseEntrySelectionBehavior and plays the minimize/maximize \
             sound; UpdatePlusMinusTexture swaps `common-icon-plus`/`common-icon-minus` based on \
             collapsed state"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_list_does_not_publish_house_entry_template_global(env: &WowLuaEnv) {
    load_house_list(env);

    let entry_template_published: bool = env
        .eval("return _G['HouseEntryTemplate'] ~= nil")
        .expect("HouseEntryTemplate global lookup should succeed");
    assert!(
        !entry_template_published,
        "HouseEntryTemplate is declared `virtual=\"true\"` (Blizzard_HouseList.xml line 4) — \
         virtual XML templates are NOT instantiated as global frames at load time. They only \
         materialize when a parent frame inherits them. The HouseListFrame's ScrollBox \
         instantiates HouseEntryTemplate per row via the WowScrollBoxList view's \
         SetElementInitializer call, but the template name itself stays out of `_G`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_list_frame_publishes_all_named_children(env: &WowLuaEnv) {
    load_house_list(env);

    let close_button_exists: bool = env
        .eval(
            "local b = _G['HouseListFrameCloseButton']; return type(b) == 'table' and type(b.GetName) == 'function'",
        )
        .expect("HouseListFrameCloseButton global lookup should succeed");
    assert!(
        close_button_exists,
        "HouseListFrameCloseButton must publish — the XML declares the close button with \
         `name=\"$parentCloseButton\"` (Blizzard_HouseList.xml line 126) which expands to \
         `HouseListFrameCloseButton` against the parent's name. UIPanelCloseButtonDefaultAnchors \
         provides the standard top-right anchoring"
    );

    for parent_key in ["ScrollBox", "ScrollBar", "LoadingSpinner", "Background"] {
        let exists: bool = env
            .eval(&format!(
                "return type(HouseListFrame['{parent_key}']) == 'table' and type(HouseListFrame['{parent_key}'].GetName) == 'function'"
            ))
            .expect("HouseListFrame parentKey child lookup should succeed");
        assert!(
            exists,
            "HouseListFrame.{parent_key} must publish via `parentKey` — the XML wires four \
             non-named children with `parentKey=` so that the mixin can address them without \
             touching `_G`: ScrollBox (WowScrollBoxList), ScrollBar (MinimalScrollBar EventFrame \
             at HIGH strata), LoadingSpinner (SpinnerTemplate at frameLevel=2000, hidden until \
             VIEW_HOUSES_LIST_RECIEVED fires), Background (housing-basic-container atlas)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_list_registers_view_houses_list_recieved_event_on_show(env: &WowLuaEnv) {
    load_house_list(env);

    env.eval::<()>("HouseListFrame:Show()")
        .expect("HouseListFrame:Show should succeed");

    let registered: bool = env
        .eval("return HouseListFrame:IsEventRegistered('VIEW_HOUSES_LIST_RECIEVED')")
        .expect("IsEventRegistered query should succeed");
    assert!(
        registered,
        "After `HouseListFrame:Show()`, the `VIEW_HOUSES_LIST_RECIEVED` event must be \
         registered — HouseListFrameMixin:OnShow calls \
         `FrameUtil.RegisterFrameForEvents(self, HouseListFrameShowingEvents)` where \
         HouseListFrameShowingEvents is the local table `{{ \"VIEW_HOUSES_LIST_RECIEVED\" }}` \
         (note Blizzard's typo — RECIEVED, not RECEIVED — preserved verbatim from the source)"
    );

    env.eval::<()>("HouseListFrame:Hide()")
        .expect("HouseListFrame:Hide should succeed");

    let still_registered: bool = env
        .eval("return HouseListFrame:IsEventRegistered('VIEW_HOUSES_LIST_RECIEVED')")
        .expect("IsEventRegistered query should succeed");
    assert!(
        !still_registered,
        "After `HouseListFrame:Hide()`, the `VIEW_HOUSES_LIST_RECIEVED` event must be \
         unregistered — HouseListFrameMixin:OnHide calls \
         `FrameUtil.UnregisterFrameForEvents(self, HouseListFrameShowingEvents)`. This \
         show/hide-scoped registration ensures the social dialog only consumes the event while \
         visible, avoiding stale list updates against a hidden frame"
    );
}
}

fn verify_house_finder_scenario(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &house_finder_toc()).expect("original House Finder addon loads");
    env.exec(SUPPLY_NEIGHBORHOODS).expect("supply distinct neighborhoods");
    env.exec("C_Housing.StartTutorial()")
        .expect("start the inferred tutorial flow through C_Housing");
    env.process_timers().expect("neighborhood reply dispatches asynchronously");
    let selected: String = env.eval(SELECT_SECOND_NEIGHBORHOOD)
        .expect("original frame selects supplied neighborhood");
    assert_eq!(selected, "Amber Vale/Cobalt Reach;41/82");
    let faction_match: bool = env
        .eval("return C_Housing.DoesFactionMatchNeighborhood('Neighborhood-B')")
        .expect("faction result comes from supplied neighborhood state");
    assert!(!faction_match);
    env.process_timers().expect("plot reply dispatches asynchronously");
    let supplied_plot: String = env
        .eval("local p = HouseFinderFrame.mapDataProvider.houseMapData[1]; return p.plotID .. '/' .. p.ownerName")
        .expect("original map provider stores supplied plot fields");
    assert_eq!(supplied_plot, "29/Mira");
}

prefork_full_ui_case! {
fn supplied_house_finder_neighborhoods_and_plots_reach_original_consumer(env: &WowLuaEnv) {
    load_house_list(env);
    verify_house_finder_scenario(env);
    let templates_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingTemplates')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(templates_loaded, "HouseList dependency remains loaded via Game-screen discovery");
}
}

prefork_full_ui_case! {
fn blizzard_house_list_publishes_no_event_listeners_before_show(env: &WowLuaEnv) {
    load_house_list(env);

    let registered_before_show: bool = env
        .eval("return HouseListFrame:IsEventRegistered('VIEW_HOUSES_LIST_RECIEVED')")
        .expect("IsEventRegistered query should succeed");
    assert!(
        !registered_before_show,
        "Before any `HouseListFrame:Show()`, the `VIEW_HOUSES_LIST_RECIEVED` event must NOT be \
         registered — HouseListFrameMixin:OnLoad does NOT call RegisterEvent / \
         RegisterFrameForEvents at load time. Event registration is exclusively driven by \
         OnShow/OnHide so that a hidden HouseListFrame does not consume housing-list updates \
         destined for other surfaces"
    );
}
}

