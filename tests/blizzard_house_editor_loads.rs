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

fn house_editor_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HouseEditor")
}

fn house_editor_toc() -> PathBuf {
    house_editor_dir().join("Blizzard_HouseEditor.toc")
}

fn load_house_editor(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &house_editor_toc())
        .expect("Blizzard_HouseEditor should load via explicit Rust loader call");
}

#[test]
fn blizzard_house_editor_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&house_editor_dir()).expect("Blizzard_HouseEditor TOC should resolve");
    assert_eq!(
        resolved,
        house_editor_toc(),
        "Blizzard_HouseEditor ships exactly one bare TOC (`Blizzard_HouseEditor.toc`) — no \
         flavor variants. The Midnight housing editor only ships on retail (`## \
         AllowLoadGameType: standard`) and uses the bare TOC suffix that `find_toc_file` \
         (src/loader/mod.rs:65) falls through to"
    );
}

#[test]
fn blizzard_house_editor_toc_declares_lod_with_seven_dependencies() {
    let toc = TocFile::from_file(&house_editor_toc()).expect("HouseEditor TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HouseEditor declares `## LoadOnDemand: 1` — the housing editor surface only \
         loads when the player enters a player-owned residence and the housing C_API signals \
         the editor open path"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_HouseEditor does not declare `## LoadFirst: 1` — LoadOnDemand precludes any \
         load-order priority since the addon only loads on demand"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_HouseEditor does not declare `## UseSecureEnvironment` — runs in the standard \
         Lua environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_HousingTemplates".to_string(),
            "Blizzard_CustomizationUI".to_string(),
            "Blizzard_HousingMarketCart".to_string(),
            "Blizzard_CatalogShopSharedTemplates".to_string(),
            "Blizzard_HousingControls".to_string(),
            "Blizzard_HousingBlueprint".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Blizzard_HouseEditor declares seven `## Dependencies:` in published order, adding \
         Blizzard_HousingBlueprint and Blizzard_GameMenuEsc after Blizzard_HousingControls."
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HouseEditor declares NO `## SavedVariables*` — placed-decor state, room \
         layout, and exterior customization are server-authoritative (synced via \
         C_HousingDecor / C_HousingLayout / C_HousingExteriorCustomization), no \
         per-installation persistence is needed"
    );
}

#[test]
fn blizzard_house_editor_toc_is_retail_only_and_omits_allow_load() {
    let toc = TocFile::from_file(&house_editor_toc()).expect("HouseEditor TOC should parse");
    let toc_text = std::fs::read_to_string(house_editor_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Blizzard_HouseEditor declares `## AllowLoadGameType: standard` — the housing editor \
         is a Midnight expansion feature that only ships on retail. \
         `is_game_type_restricted()` (src/toc.rs:294) treats `standard` and `mainline` as the \
         unrestricted retail flavor, so this addon is NOT considered game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HouseEditor must NOT be game-type restricted — `## AllowLoadGameType: \
         standard` matches the retail flavor that the simulator runs as"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_HouseEditor omits `## AllowLoad:` — LoadOnDemand precludes auto-discovery \
         gating, so the AllowLoad value would be inert. The addon is pulled exclusively via the \
         Lua-side LoadAddOn(\"Blizzard_HouseEditor\") path"
    );
    assert!(
        !toc_text.contains("## DefaultState:"),
        "Blizzard_HouseEditor omits `## DefaultState:` — relies on the loader's implicit-enabled \
         default for Blizzard prefix LoD addons"
    );
}

#[test]
fn blizzard_house_editor_toc_lists_thirty_five_files() {
    let toc = TocFile::from_file(&house_editor_toc()).expect("HouseEditor TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files.len(),
        35,
        "Blizzard_HouseEditor TOC body lists exactly 35 source files in the active retail \
         cache. Got: {:?}",
        files
    );
    assert_eq!(
        files.first().map(String::as_str),
        Some("Blizzard_HouseEditorTemplates.lua"),
        "First TOC body file should be `Blizzard_HouseEditorTemplates.lua` — base mode mixin + \
         instructions container + budget/decor/room count helpers must register before any \
         CreateFromMixins(BaseHouseEditorModeMixin) call in the per-mode files"
    );
    assert_eq!(
        files.last().map(String::as_str),
        Some("Blizzard_HouseEditorRegistration.lua"),
        "Last TOC body file should be `Blizzard_HouseEditorRegistration.lua` — calls \
         RegisterUIPanel(HouseEditorFrame, {{area=\"full\", pushable=0}}) and must run AFTER the \
         HouseEditorFrame XML instantiates (Blizzard_HouseEditor.xml is the second-to-last pair)"
    );
}

#[test]
fn blizzard_house_editor_directory_holds_thirty_six_entries() {
    let dir = house_editor_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HouseEditor directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        36,
        "Blizzard_HouseEditor directory ships exactly 36 entries: 35 source files referenced by \
         the TOC plus its TOC file. Got: {entries:?}"
    );
    assert!(
        entries.contains(&"Blizzard_HouseEditor.toc".to_string()),
        "Blizzard_HouseEditor directory must contain the bare TOC file"
    );
    assert!(
        entries.contains(&"Blizzard_HouseEditorRegistration.lua".to_string()),
        "Blizzard_HouseEditor directory must contain the Registration tail file (8 lines, runs \
         RegisterUIPanel after the umbrella XML instantiates)"
    );
}

#[test]
fn blizzard_house_editor_excluded_from_all_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_HouseEditor");
        assert!(
            !discovered,
            "Blizzard_HouseEditor MUST NOT appear in {screen:?} auto-discovery — \
             `## LoadOnDemand: 1` keeps it out of every screen pass. Unlike GuildControlUI \
             (which is pulled via Communities' `## RequiredDep`), no non-LoD addon RequiredDeps \
             HouseEditor. The only addon that depends on it (Blizzard_HousingInspectModeUI) is \
             itself LoD with `## Dependencies` (not `## RequiredDep`), so the LoD-pull \
             promotion path in `pull_required_lod_addons` (src/loader/mod.rs:357) does not \
             escalate HouseEditor onto any auto-discovery pass"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_house_editor_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_house_editor(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HouseEditor/")
                || e.contains("Blizzard_HouseEditor\\")
                || e.contains("HouseEditorFrameMixin")
                || e.contains("BaseHouseEditorModeMixin")
                || e.contains("HouseEditorBasicDecorMode")
                || e.contains("HouseEditorCustomizeMode")
                || e.contains("HouseEditorLayoutMode")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_HouseEditor emitted addon-specific Lua errors during explicit LoD load:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_is_addon_loaded_returns_true_after_explicit_lod_load(env: &WowLuaEnv) {
    load_house_editor(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HouseEditor')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After explicit `load_addon` of Blizzard_HouseEditor.toc following Game-screen \
         auto-discovery (which loads its 5 deps but skips HouseEditor itself due to \
         LoadOnDemand), `C_AddOns.IsAddOnLoaded('Blizzard_HouseEditor')` should return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_house_editor_frame_global(env: &WowLuaEnv) {
    load_house_editor(env);

    let exists: bool = env
        .eval(
            "local f = _G['HouseEditorFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("HouseEditorFrame global lookup should succeed");
    assert!(
        exists,
        "After LoD load, `HouseEditorFrame` should publish as a global frame instance — \
         Blizzard_HouseEditor.xml line 3 declares `<Frame name=\"HouseEditorFrame\" \
         mixin=\"HouseEditorFrameMixin\" inherits=\"TopLevelParentScaleFrameTemplate\" \
         setAllPoints=\"true\" ignoreParentScale=\"true\" enableKeyboard=\"true\" \
         hidden=\"true\">`. The Registration.lua tail then calls `RegisterUIPanel(HouseEditorFrame, \
         {{area=\"full\", pushable=0}})` to register the frame as a full-area UI panel"
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_two_free_helpers(env: &WowLuaEnv) {
    load_house_editor(env);

    for helper in ["HouseEditorFrame_GetFrame", "HouseEditorFrame_IsShown"] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("HouseEditor helper existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor.lua must publish `_G['{helper}']` — these 2 free getters \
             expose the umbrella frame to other addons / slash commands without exposing the \
             mixin: GetFrame returns `HouseEditorFrame` directly, IsShown returns \
             `HouseEditorFrame and HouseEditorFrame:IsShown()` (nil-safe)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_frame_mixin_publishes_sixteen_methods(env: &WowLuaEnv) {
    load_house_editor(env);

    for method in [
        "OnLoad",
        "ShowAfterCheckout",
        "HideForCheckout",
        "OnEvent",
        "OnShow",
        "OnHide",
        "GetActiveModeFrame",
        "OnActiveModeChanged",
        "HandleEscape",
        "OnDecorSelectResponse",
        "ShowHouseStorage",
        "HideHouseStorage",
        "HouseStorageSetShown",
        "ExpandHouseStorage",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HouseEditorFrameMixin['{method}']) == 'function'"
            ))
            .expect("HouseEditorFrameMixin method existence query should succeed");
        assert!(
            exists,
            "HouseEditorFrameMixin must expose `:{method}()` — the umbrella mixin drives the \
             editor lifecycle: OnLoad registers SIMPLE_CHECKOUT_CLOSED + builds modeFramesByMode \
             keyed on Enum.HouseEditorMode; OnEvent dispatches HOUSE_EDITOR_MODE_CHANGED to \
             OnActiveModeChanged + HOUSING_DECOR_SELECT_RESPONSE to OnDecorSelectResponse; \
             OnShow/OnHide manage child-frame visibility; ShowAfterCheckout/HideForCheckout \
             toggle alpha for the simple-checkout overlay; GetActiveModeFrame / \
             OnActiveModeChanged switch the visible mode frame; HandleEscape closes the editor; \
             ShowHouseStorage / HideHouseStorage / HouseStorageSetShown / ExpandHouseStorage \
             drive the side StoragePanel visibility synced via the EventRegistry callback"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_six_mode_mixins_inheriting_base(env: &WowLuaEnv) {
    load_house_editor(env);

    for mode_mixin in [
        "BaseHouseEditorModeMixin",
        "HouseEditorBasicDecorModeMixin",
        "HouseEditorCustomizeModeMixin",
        "HouseEditorCleanupModeMixin",
        "HouseEditorLayoutModeMixin",
        "HouseEditorExpertDecorModeMixin",
        "HouseEditorExteriorCustomizationModeMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mode_mixin}']) == 'table'"))
            .expect("mode mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mode_mixin}']` — the editor exposes 6 \
             mode-specific mixins that all derive from BaseHouseEditorModeMixin via \
             CreateFromMixins (except ExteriorCustomization which is a plain `{{}}` table — \
             different lifecycle since it operates on the house exterior model rather than \
             room-by-room interior). Each mode is bound to a child Frame in HouseEditorFrame's \
             modeFramesByMode dispatch table keyed by Enum.HouseEditorMode.{{BasicDecor, \
             Customize, Cleanup, Layout, ExpertDecor, ExteriorCustomization}}"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_dye_and_room_component_mixins(env: &WowLuaEnv) {
    load_house_editor(env);

    for mixin in [
        "HousingDyePaneMixin",
        "HousingDecorDyeSlotMixin",
        "HousingDecorDyeSlotPopoutMixin",
        "HousingDecorDyeSwatchMixin",
        "HousingDyeCostIconMixin",
        "HousingRoomComponentOptionMixin",
        "HousingRoomComponentThemeMixin",
        "HousingRoomComponentWallpaperMixin",
        "HousingRoomComponentCeilingTypeMixin",
        "HousingRoomComponentDoorTypeMixin",
        "HousingRoomComponentApplyToAllButtonMixin",
        "RoomComponentPaneMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("dye/room mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mixin}']` — the Customize mode draws the \
             room-theme + wallpaper + ceiling + door + dye pickers from these mixins. \
             HousingRoomComponent{{Theme, Wallpaper, CeilingType, DoorType}}Mixin all extend \
             HousingRoomComponentOptionMixin via CreateFromMixins, and \
             HousingRoomComponentApplyToAllButtonMixin extends UIButtonMixin"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_layout_and_pin_mixins(env: &WowLuaEnv) {
    load_house_editor(env);

    for mixin in [
        "HouseEditorLayoutFloorLineMixin",
        "HouseEditorLayoutFloorSelectMixin",
        "HousingLayoutBasePinMixin",
        "HousingLayoutDoorPinMixin",
        "HousingLayoutRoomPinMixin",
        "HousingLayoutRoomOptionMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("layout/pin mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mixin}']` — the Layout mode renders the \
             house floor plan with a base pin mixin + 2 specialized pin variants \
             (DoorPin / RoomPin both extending HousingLayoutBasePinMixin via CreateFromMixins). \
             HouseEditorLayoutFloorLineMixin draws the inter-floor connection lines and \
             HouseEditorLayoutFloorSelectMixin drives the floor-picker dropdown"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_exterior_dropdown_mixins(env: &WowLuaEnv) {
    load_house_editor(env);

    for mixin in [
        "HousingExteriorFixturePointMixin",
        "HouseExteriorOptionDropdownElementMixin",
        "HouseExteriorOptionDropdownMixin",
        "HouseExteriorTypeDropdownMixin",
        "HouseExteriorSizeDropdownMixin",
        "HouseExteriorCoreFixtureDropdownMixin",
        "HouseExteriorOptionElementMixin",
        "HouseExteriorFixtureOptionListMixin",
        "HouseExteriorCheckboxOptionMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("exterior mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mixin}']` — the ExteriorCustomization mode \
             exposes the house exterior dropdown chain (Type → Size → CoreFixture) plus \
             per-fixture option lists and on/off checkboxes that drive the \
             HousingExteriorFixturePoint world clickable handles"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_modes_bar_and_button_mixins(env: &WowLuaEnv) {
    load_house_editor(env);

    for mixin in [
        "BaseHouseEditorModesBarMixin",
        "HouseEditorModesBarMixin",
        "HouseEditorSubmodesBarMixin",
        "BaseHouseEditorModeButtonMixin",
        "HouseEditorModeButtonMixin",
        "HouseEditorSubmodeButtonMixin",
        "HouseEditorOLDSubmodeButtonMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("mode bar/button mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mixin}']` — the umbrella ModeBar exposes \
             the 6 top-level mode buttons (HouseEditorModesBarMixin extends \
             BaseHouseEditorModesBarMixin), with a parallel SubmodesBar surface for \
             expert-decor + cleanup sub-modes (HouseEditorSubmodesBarMixin same parent). \
             ModeButton + SubmodeButton + the legacy OLDSubmodeButton variant all derive from \
             BaseHouseEditorModeButtonMixin via CreateFromMixins"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_storage_and_placed_decor_mixins(env: &WowLuaEnv) {
    load_house_editor(env);

    for mixin in [
        "HouseEditorStorageButtonMixin",
        "HouseEditorStorageFrameMixin",
        "HouseEditorPlacedDecorListButtonMixin",
        "HouseEditorPlacedDecorListMixin",
        "HouseEditorPlacedDecorEntryMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("storage mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mixin}']` — the StoragePanel side surface \
             surfaces the player's placed decor with HouseEditorStorageFrameMixin owning the \
             list, HouseEditorStorageButtonMixin driving the toggle, and the placed-decor list \
             with its own list mixin + per-row entry mixin + the toolbar button mixin"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_template_count_and_instruction_mixins(env: &WowLuaEnv) {
    load_house_editor(env);

    for mixin in [
        "HouseEditorInstructionsContainerMixin",
        "HouseEditorInstructionMixin",
        "HouseEditorBudgetCountMixin",
        "HouseEditorDecorCountMixin",
        "HouseEditorRoomCountMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("template count mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HouseEditor must publish `_G['{mixin}']` — the Templates.lua publishes 5 \
             shared mixins consumed across modes: InstructionsContainer + Instruction wrap the \
             top-of-frame help text strip; BudgetCount / DecorCount / RoomCount drive the \
             corner counter widgets that show remaining placement budget + total decor count + \
             total room count"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_publishes_house_exterior_color_names_table(env: &WowLuaEnv) {
    load_house_editor(env);

    let exists: bool = env
        .eval("return type(HouseExteriorColorNames) == 'table'")
        .expect("HouseExteriorColorNames lookup should succeed");
    assert!(
        exists,
        "Blizzard_HouseEditor.lua must publish `HouseExteriorColorNames` as a `_G` table — \
         line 521 of Blizzard_HouseEditorExteriorOptionTemplates.lua declares the lookup table \
         that maps Enum-driven color IDs to localized display strings (consumed by \
         HouseExteriorOptionElementMixin to render the color label in the exterior fixture \
         dropdowns)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_house_editor_house_editor_mode_enum_publishes_six_modes(env: &WowLuaEnv) {
    load_house_editor(env);

    for mode in [
        "BasicDecor",
        "Customize",
        "Cleanup",
        "Layout",
        "ExpertDecor",
        "ExteriorCustomization",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return Enum.HouseEditorMode and Enum.HouseEditorMode['{mode}'] ~= nil"
            ))
            .expect("Enum.HouseEditorMode lookup should succeed");
        assert!(
            exists,
            "`Enum.HouseEditorMode.{mode}` must publish — Blizzard_HouseEditor.lua line 26-33 \
             builds modeFramesByMode keyed on these 6 enum values, and missing any entry would \
             leave the mode-frame dispatch table sparse and break OnActiveModeChanged routing. \
             The simulator publishes Enum.HouseEditorMode via \
             src/lua_api/globals/enum_data/missing_enums.lua line 6592"
        );
    }
}
}
