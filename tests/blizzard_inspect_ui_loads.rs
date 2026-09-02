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

fn inspect_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_InspectUI")
}

fn inspect_ui_toc() -> PathBuf {
    find_toc_file(&inspect_ui_dir()).expect("Blizzard_InspectUI TOC should resolve")
}

const TOC_FILES: &[&str] = &[
    "Mainline/Blizzard_InspectUI_Bootstrap.lua",
    "Mainline/Blizzard_InspectUI.xml",
    "Mainline/InspectPaperDollFrame.lua",
    "Mainline/InspectPaperDollFrame.xml",
    "Mainline/InspectPVPFrame.lua",
    "Mainline/InspectPVPFrame.xml",
    "Mainline/InspectGuildFrame.lua",
    "Mainline/InspectGuildFrame.xml",
    "Mainline/Localization.lua",
];

const SUBFRAME_NAMES: &[&str] = &[
    "InspectPaperDollFrame",
    "InspectPVPFrame",
    "InspectGuildFrame",
];

const TAB_BUTTON_NAMES: &[&str] = &["InspectFrameTab1", "InspectFrameTab2", "InspectFrameTab3"];

const ITEM_SLOT_NAMES: &[&str] = &[
    "InspectHeadSlot",
    "InspectNeckSlot",
    "InspectShoulderSlot",
    "InspectBackSlot",
    "InspectChestSlot",
    "InspectShirtSlot",
    "InspectTabardSlot",
    "InspectWristSlot",
    "InspectHandsSlot",
    "InspectWaistSlot",
    "InspectLegsSlot",
    "InspectFeetSlot",
    "InspectFinger0Slot",
    "InspectFinger1Slot",
    "InspectTrinket0Slot",
    "InspectTrinket1Slot",
    "InspectMainHandSlot",
    "InspectSecondaryHandSlot",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "InspectPaperDollItemSlotButtonTemplate",
    "InspectPaperDollItemSlotButtonLeftTemplate",
    "InspectPaperDollItemSlotButtonRightTemplate",
    "InspectPaperDollItemSlotButtonBottomTemplate",
    "InspectPvpStatTemplate",
    "InspectPvpTalentSlotTemplate",
];

const PUBLIC_FUNCTIONS: &[&str] = &[
    "InspectFrame_Show",
    "InspectFrame_OnLoad",
    "InspectFrame_OnEvent",
    "InspectFrame_OnShow",
    "InspectFrame_OnHide",
    "InspectFrame_OnUpdate",
    "InspectFrame_UnitChanged",
    "InspectFrame_UpdateTabs",
    "InspectSwitchTabs",
    "InspectFrameTab_OnClick",
    "InspectPaperDollFrame_OnLoad",
    "InspectPaperDollFrame_OnEvent",
    "InspectPaperDollFrame_SetLevel",
    "InspectPaperDollFrame_UpdateButtons",
    "InspectPaperDollFrame_OnShow",
    "InspectPaperDollItemSlotButton_OnLoad",
    "InspectPaperDollItemSlotButton_OnEvent",
    "InspectPaperDollItemSlotButton_OnClick",
    "InspectPaperDollItemSlotButton_OnEnter",
    "InspectPaperDollItemSlotButton_Update",
    "InspectPVPFrame_OnLoad",
    "InspectPVPFrame_OnEvent",
    "InspectPVPFrame_OnShow",
    "InspectPVPFrame_Update",
    "InspectGuildFrame_OnLoad",
    "InspectGuildFrame_OnEvent",
    "InspectGuildFrame_OnShow",
    "InspectGuildFrame_Update",
];

fn load_inspect_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &inspect_ui_toc())
        .expect("Blizzard_InspectUI should load via explicit Rust loader call");
}

#[test]
fn blizzard_inspect_ui_find_toc_resolves_mainline_variant() {
    let resolved = inspect_ui_toc();
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some("Blizzard_InspectUI_Mainline.toc"),
        "retail resolves the mainline InspectUI TOC through find_toc_file"
    );
}

#[test]
fn blizzard_inspect_ui_toc_declares_lod_with_no_dependencies() {
    let toc = TocFile::from_file(&inspect_ui_toc()).expect("Blizzard_InspectUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_InspectUI declares `## LoadOnDemand: 1` — pulled in on demand by \
         `InspectFrame_Show(unit)` (called from unit popup menus / chat hyperlinks) which \
         triggers `UIParentLoadAddOn('Blizzard_InspectUI')` before showing the InspectFrame; \
         not loaded eagerly because most play sessions never inspect another player"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_InspectUI declares zero `## Dependencies:` / `## RequiredDep` — every \
         template it consumes (ButtonFrameTemplate, PanelTabButtonTemplate, \
         PaperDollItemSocketDisplayVerticalTemplate, PvpTalentSlotTemplate) ships from \
         Blizzard_FrameXML / Blizzard_SharedXML / Blizzard_PVPUI which are guaranteed loaded by \
         the time any `## LoadOnDemand` addon can resolve"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_InspectUI declares no `## OptionalDeps` — minimal LoD module relies entirely \
         on the eager FrameXML / SharedXML pass having published its templates already"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_InspectUI declares zero saved variables — every inspect session re-fetches \
         from the server via NotifyInspect / INSPECT_READY; no client-side persistence"
    );
}

#[test]
fn blizzard_inspect_ui_toc_declares_current_game_mainline_metadata() {
    let toc = TocFile::from_file(&inspect_ui_toc()).expect("Blizzard_InspectUI TOC should parse");

    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_InspectUI's `## AllowLoadGameType: mainline` is available to the retail \
         profile"
    );

    let raw =
        std::fs::read_to_string(inspect_ui_toc()).expect("Blizzard_InspectUI TOC should read");
    assert!(
        raw.contains("## AllowLoad: game"),
        "TOC declares current game-only AllowLoad metadata"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC declares current mainline game-type metadata"
    );
}

#[test]
fn blizzard_inspect_ui_toc_lists_current_files_in_expected_order() {
    let toc = TocFile::from_file(&inspect_ui_toc()).expect("Blizzard_InspectUI TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        TOC_FILES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "current retail TOC must list its bootstrap and Mainline files in declared order"
    );
}

#[test]
fn blizzard_inspect_ui_directory_holds_seven_entries() {
    let entries = std::fs::read_dir(inspect_ui_dir())
        .expect("Blizzard_InspectUI directory should read")
        .count();
    assert_eq!(
        entries, 7,
        "Current retail cache holds seven top-level entries: the mainline TOC, root bootstrap, \
         and the Cata, Mainline, Mists, TBC, and Vanilla source directories"
    );
}

#[test]
fn blizzard_inspect_ui_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_InspectUI");
        assert!(
            !found,
            "Blizzard_InspectUI must NOT appear in any ScreenKind auto-discovery sweep — \
             `## LoadOnDemand: 1` excludes it from every eager pass; only `load_addon` called \
             explicitly from `InspectFrame_Show(unit)` pulls it in. (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_inspect_ui(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_InspectUI/")
                || message.contains("InspectFrame_")
                || message.contains("InspectPaperDollFrame_")
                || message.contains("InspectPVPFrame_")
                || message.contains("InspectGuildFrame_")
                || message.contains("InspectPvpTalentSlotMixin")
                || message.contains("InspectPaperDollFrameTalentsButtonMixin")
                || message.contains("LevelTextMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_InspectUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_is_addon_loaded_after_explicit_lod(env: &WowLuaEnv) {
    load_inspect_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_InspectUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_InspectUI') must return true after the explicit \
         load_addon call — proves the LoD path completes through `load_addon` and the addon \
         registers in the loaded-addon list"
    );
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_named_main_frame_publishes_with_inherits_chain(env: &WowLuaEnv) {
    load_inspect_ui(env);

    let kind: String = env
        .eval("return type(InspectFrame)")
        .expect("InspectFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "InspectFrame must publish at `_G` as a table — declared at Blizzard_InspectUI.xml:4 \
         with `name=\"InspectFrame\"` `parent=\"UIParent\"` `inherits=\"ButtonFrameTemplate\"` \
         `toplevel=\"true\"` `movable=\"true\"` `enableMouse=\"true\"` `hidden=\"true\"`"
    );

    let name: String = env
        .eval("return InspectFrame:GetName()")
        .expect("InspectFrame:GetName() probe should succeed");
    assert_eq!(name, "InspectFrame");

    let hidden: bool = env
        .eval("return InspectFrame:IsShown() == false")
        .expect("InspectFrame:IsShown() probe should succeed");
    assert!(
        hidden,
        "InspectFrame declares `hidden=\"true\"` (Blizzard_InspectUI.xml:4) — must start hidden \
         until InspectFrame_Show(unit) calls ShowUIPanel after INSPECT_READY arrives"
    );
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_main_frame_carries_three_tab_buttons(env: &WowLuaEnv) {
    load_inspect_ui(env);

    for tab_name in TAB_BUTTON_NAMES {
        let kind: String = env
            .eval(&format!("return type({tab_name})"))
            .unwrap_or_else(|err| panic!("{tab_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{tab_name} must publish at `_G` as a table — InspectFrame ships exactly 3 tabs \
             (CHARACTER / PVP / GUILD) declared at Blizzard_InspectUI.xml:6-41, each \
             inheriting PanelTabButtonTemplate. PanelTemplates_SetNumTabs(self, 3) at \
             InspectFrame_OnLoad locks the count"
        );

        let id: i32 = env
            .eval(&format!("return {tab_name}:GetID()"))
            .unwrap_or_else(|err| panic!("{tab_name}:GetID() probe failed: {err}"));
        let expected_id: i32 = tab_name
            .trim_start_matches("InspectFrameTab")
            .parse()
            .expect("tab name suffix is a digit");
        assert_eq!(
            id, expected_id,
            "{tab_name}:GetID() must return {expected_id} — XML `id=\"{expected_id}\"` \
             attribute drives InspectSwitchTabs's INSPECTFRAME_SUBFRAMES[id] lookup so each tab \
             routes to the right subframe via the file-global subframe table"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_main_frame_carries_three_subframes(env: &WowLuaEnv) {
    load_inspect_ui(env);

    for subframe_name in SUBFRAME_NAMES {
        let kind: String = env
            .eval(&format!("return type({subframe_name})"))
            .unwrap_or_else(|err| panic!("{subframe_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{subframe_name} must publish at `_G` as a table — declared with `parent=\
             \"InspectFrame\"` `setAllPoints=\"true\"` `useParentLevel=\"true\"`. The file-global \
             INSPECTFRAME_SUBFRAMES = {{\"InspectPaperDollFrame\", \"InspectPVPFrame\", \
             \"InspectGuildFrame\"}} (Blizzard_InspectUI.lua:2) drives InspectSwitchTabs's \
             newFrame = _G[INSPECTFRAME_SUBFRAMES[newID]] lookup"
        );

        let parent_name: String = env
            .eval(&format!("return {subframe_name}:GetParent():GetName()"))
            .unwrap_or_else(|err| panic!("{subframe_name}:GetParent() probe failed: {err}"));
        assert_eq!(
            parent_name, "InspectFrame",
            "{subframe_name} must reparent to InspectFrame — XML `parent=\"InspectFrame\"` \
             attribute resolves at frame-creation time"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_paperdoll_frame_publishes_eighteen_item_slot_buttons(env: &WowLuaEnv) {
    load_inspect_ui(env);

    for slot_name in ITEM_SLOT_NAMES {
        let kind: String = env
            .eval(&format!("return type({slot_name})"))
            .unwrap_or_else(|err| panic!("{slot_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{slot_name} must publish at `_G` as a table — InspectPaperDollFrame ships exactly \
             18 equipment-slot ItemButtons (Head/Neck/Shoulder/Back/Chest/Shirt/Tabard/Wrist on \
             the left column inheriting InspectPaperDollItemSlotButtonLeftTemplate; \
             Hands/Waist/Legs/Feet/Finger0/Finger1/Trinket0/Trinket1 on the right column \
             inheriting InspectPaperDollItemSlotButtonRightTemplate; MainHand/SecondaryHand \
             along the bottom inheriting InspectPaperDollItemSlotButtonBottomTemplate). Each \
             slot wires the same 5 OnLoad/OnEvent/OnClick/OnEnter/OnLeave handlers via the \
             InspectPaperDollItemSlotButtonTemplate base"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_virtual_templates_stay_nil_at_global_scope(env: &WowLuaEnv) {
    load_inspect_ui(env);

    for template_name in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G['{template_name}'])"))
            .unwrap_or_else(|err| panic!("_G[{template_name}] probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{template_name} must NOT publish at `_G` — declared as `virtual=\"true\"` so the \
             loader keeps it in the template registry only. Consumed via `inherits=` by the \
             concrete slot/talent ItemButtons / Frames inside each subframe, never resolved \
             through global scope"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_publishes_all_public_script_handler_functions(env: &WowLuaEnv) {
    load_inspect_ui(env);

    for fn_name in PUBLIC_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G['{fn_name}'])"))
            .unwrap_or_else(|err| panic!("_G[{fn_name}] probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fn_name} must publish at `_G` as a function — XML `<OnLoad function=\"{fn_name}\"/>` \
             style script bindings need every handler to resolve through global scope at \
             frame-creation time. Declared without the `local` keyword in \
             Blizzard_InspectUI.lua / InspectPaperDollFrame.lua / InspectPVPFrame.lua / \
             InspectGuildFrame.lua so each name reaches `_G`"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_publishes_inspectframe_subframes_lookup_table(env: &WowLuaEnv) {
    load_inspect_ui(env);

    let kind: String = env
        .eval("return type(INSPECTFRAME_SUBFRAMES)")
        .expect("INSPECTFRAME_SUBFRAMES probe should succeed");
    assert_eq!(
        kind, "table",
        "INSPECTFRAME_SUBFRAMES must publish at `_G` as a table — declared without `local` \
         (Blizzard_InspectUI.lua:2). Drives InspectSwitchTabs's _G[INSPECTFRAME_SUBFRAMES[newID]] \
         lookup so each tab id (1/2/3) maps to InspectPaperDollFrame / InspectPVPFrame / \
         InspectGuildFrame respectively"
    );

    for (idx, expected) in SUBFRAME_NAMES.iter().enumerate() {
        let lua_idx = idx + 1;
        let entry: String = env
            .eval(&format!("return INSPECTFRAME_SUBFRAMES[{lua_idx}]"))
            .unwrap_or_else(|err| panic!("INSPECTFRAME_SUBFRAMES[{lua_idx}] probe failed: {err}"));
        assert_eq!(
            entry, *expected,
            "INSPECTFRAME_SUBFRAMES[{lua_idx}] must equal `{expected}` — id-to-name routing \
             table"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_inspected_unit_initializes_to_nil(env: &WowLuaEnv) {
    load_inspect_ui(env);

    let kind: String = env
        .eval("return type(INSPECTED_UNIT)")
        .expect("INSPECTED_UNIT probe should succeed");
    assert_eq!(
        kind, "nil",
        "INSPECTED_UNIT must publish as a `_G` value initialized to nil — declared without \
         `local` (Blizzard_InspectUI.lua:4) so that it reaches `_G`. Set by InspectFrame_Show \
         to the unit being inspected; cleared back to nil when CanInspect returns false. \
         Initial nil state proves no inspect session is active immediately after load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_inspect_ui_uipanelwindows_registers_inspectframe_left_area(env: &WowLuaEnv) {
    load_inspect_ui(env);

    let area: String = env
        .eval("return UIPanelWindows['InspectFrame'].area")
        .expect("UIPanelWindows.InspectFrame.area probe should succeed");
    assert_eq!(
        area, "left",
        "UIPanelWindows['InspectFrame'].area must equal `left` — registered at \
         Blizzard_InspectUI.lua:6 so ShowUIPanel anchors InspectFrame on the left side of the \
         screen alongside CharacterFrame / TalentFrame / SpellBookFrame"
    );

    let pushable: i32 = env
        .eval("return UIPanelWindows['InspectFrame'].pushable")
        .expect("UIPanelWindows.InspectFrame.pushable probe should succeed");
    assert_eq!(
        pushable, 0,
        "UIPanelWindows['InspectFrame'].pushable must equal 0 — InspectFrame cannot be pushed \
         to the secondary panel slot, so opening another left-area panel will close \
         InspectFrame instead of stacking"
    );
}
}
