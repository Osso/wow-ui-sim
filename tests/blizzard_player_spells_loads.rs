use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn player_spells_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PlayerSpells")
}

fn player_spells_toc() -> PathBuf {
    player_spells_dir().join("Blizzard_PlayerSpells.toc")
}

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_SharedTalentUI",
    "Blizzard_PagedContent",
    "Blizzard_HelpPlate",
];

const PLAYER_SPELLS_TOC_FILES: &[&str] = &[
    "Blizzard_PlayerSpells_Bootstrap.lua",
    "ClassSpecializations/Blizzard_ClassSpecializationsFrame.xml",
    "ClassTalents/Blizzard_ClassTalentUtil.lua",
    "ClassTalents/Blizzard_ClassTalentLoadoutDialogTemplates.xml",
    "ClassTalents/Blizzard_ClassTalentImportExport.lua",
    "ClassTalents/Blizzard_ClassTalentLoadoutImportDialog.xml",
    "ClassTalents/Blizzard_ClassTalentLoadoutEditDialog.xml",
    "ClassTalents/Blizzard_ClassTalentLoadoutCreateDialog.xml",
    "ClassTalents/Blizzard_ClassTalentButtonTemplates.xml",
    "ClassTalents/Blizzard_ClassTalentEdgeTemplates.lua",
    "ClassTalents/Blizzard_ClassTalentEdgeTemplates.xml",
    "ClassTalents/Blizzard_HeroTalentsSelectionDialog.lua",
    "ClassTalents/Blizzard_HeroTalentsSelectionDialog.xml",
    "ClassTalents/Blizzard_HeroTalentsContainer.lua",
    "ClassTalents/Blizzard_HeroTalentsContainer.xml",
    "ClassTalents/Blizzard_ClassTalentSearch.lua",
    "ClassTalents/Blizzard_ClassTalentsFrame.xml",
    "PvPTalents/Blizzard_PvPTalentListTemplates.xml",
    "PvPTalents/Blizzard_PvPTalentSlotTemplates.xml",
    "PvPTalents/Blizzard_WarmodeButtonTemplate.xml",
    "SpellBook/Blizzard_SpellBookTemplates.xml",
    "SpellBook/Blizzard_SpellBookItem.xml",
    "SpellBook/Blizzard_SpellBookSearch.lua",
    "SpellBook/Blizzard_SpellBookFrame.xml",
    "Blizzard_PlayerSpellsFrame.xml",
    "Blizzard_PlayerSpellsRegistration.lua",
    "Localization.lua",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &[
    "PlayerSpellsFrame",
    "ClassTalentLoadoutCreateDialog",
    "ClassTalentLoadoutEditDialog",
    "ClassTalentLoadoutImportDialog",
    "HeroTalentsSelectionDialog",
];

const ROOT_PANEL_MIXINS: &[&str] = &[
    "PlayerSpellsFrameMixin",
    "ClassSpecFrameMixin",
    "ClassTalentsFrameMixin",
    "SpellBookFrameMixin",
];

const TALENT_BUTTON_MIXINS: &[&str] = &[
    "ClassTalentButtonBaseMixin",
    "ClassTalentButtonArtMixin",
    "ClassTalentButtonSpendMixin",
    "ClassTalentButtonSelectMixin",
    "ClassTalentButtonSplitSelectMixin",
    "ClassTalentButtonCapstonePipMixin",
    "ClassTalentButtonCapstoneWithTrackMixin",
    "ClassTalentEdgeArrowMixin",
    "ClassTalentSelectionChoiceMixin",
];

const HERO_TALENT_MIXINS: &[&str] = &[
    "HeroSpecButtonMixin",
    "HeroTalentsContainerMixin",
    "HeroTalentSpecContentMixin",
    "HeroTalentsSelectionMixin",
    "HeroTalentActivateButtonMixin",
    "HeroTalentCollapseButtonMixin",
    "HeroTalentsUnlockedAnimFrameMixin",
];

const SPELL_BOOK_MIXINS: &[&str] = &[
    "SpellBookFrameMixin",
    "SpellBookFrameTutorialsMixin",
    "SpellBookItemMixin",
    "SpellBookItemButtonMixin",
    "SpellBookHeaderMixin",
    "SpellBookSearchMixin",
    "BaseSpellBookCategoryMixin",
    "SpellBookGeneralCategoryMixin",
    "SpellBookClassCategoryMixin",
    "SpellBookPetCategoryMixin",
];

const LOADOUT_DIALOG_MIXINS: &[&str] = &[
    "ClassTalentLoadoutDialogMixin",
    "ClassTalentLoadoutCreateDialogMixin",
    "ClassTalentLoadoutEditDialogMixin",
    "ClassTalentLoadoutImportDialogMixin",
    "ClassTalentLoadoutDialogInputControlMixin",
    "ClassTalentLoadoutDialogNameControlMixin",
    "ClassTalentLoadoutCreateDialogNameControlMixin",
    "ClassTalentLoadoutEditDialogNameControlMixin",
    "ClassTalentLoadoutImportDialogNameControlMixin",
    "ClassTalentLoadoutImportDialogImportControlMixin",
];

const PVP_AND_WARMODE_MIXINS: &[&str] = &[
    "PvPTalentListMixin",
    "PvPTalentListButtonMixin",
    "PvPTalentSlotTrayMixin",
    "PvPTalentSlotButtonMixin",
    "WarmodeButtonMixin",
    "WarmodeIncentiveMixin",
];

fn load_full_game_ui_with_player_spells() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &player_spells_toc())
        .expect("explicit load_addon for Blizzard_PlayerSpells succeeds");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_player_spells_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&player_spells_dir()).expect("Blizzard_PlayerSpells TOC resolves");
    assert_eq!(
        resolved,
        player_spells_toc(),
        "Blizzard_PlayerSpells ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The PlayerSpells panel (the unified post-10.0 TabSystemOwner that hosts \
         SpellBook + ClassSpecializations + ClassTalents + PvPTalents tabs) is a \
         Mainline-only construct because its Tab system replaces the Mists/Cata/Wrath \
         per-tab separate-frame approach with a single shared portrait-frame parent"
    );

    let mainline = player_spells_dir().join("Blizzard_PlayerSpells_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_player_spells_toc_declares_load_on_demand_with_three_dependencies() {
    let toc = TocFile::from_file(&player_spells_toc()).expect("Blizzard_PlayerSpells TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true — the \
         PlayerSpells panel is the heaviest LoD addon in the simulator (~12000 lines of \
         Lua/XML across 26 source files covering 4 nested feature subdirs: \
         ClassSpecializations / ClassTalents / PvPTalents / SpellBook); lazy-loading \
         defers the cost until the player presses N or P (the talent / spellbook \
         keybinds) and the engine fires LoadAddOn('Blizzard_PlayerSpells') from the \
         keybinding handler"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_game_type_restricted());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` returns true for ScreenKind::Game when \
         AllowLoad is omitted"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — PlayerSpells is the \
             in-world talent/spellbook UI; glue screens have no character context"
        );
    }

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "TOC must declare exactly 3 deps via the `## Dependencies:` key: \
         Blizzard_SharedTalentUI (the cross-flavor talent-tree + node-button + edge \
         primitives that ClassTalents and HeroTalents extend), Blizzard_PagedContent \
         (the paged-scroll widget consumed by the SpellBook frame for spell pagination), \
         Blizzard_HelpPlate (the new-player tutorial overlay surface used by \
         SpellBookFrameTutorialsMixin to drive the `Click here to open the spellbook` \
         arrow). All three are non-LoD eager-discovery addons with `LoadOnDemand: 0` so \
         they're already loaded by the time PlayerSpells's explicit load_addon fires"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — talent loadouts persist on the server (the import/export \
         dialog uses C_Traits.GetActiveConfigID + GetTreeNodes, not local storage); \
         spellbook search history is non-persistent; class spec selection is server- \
         authoritative via C_SpecializationInfo"
    );
}

#[test]
fn blizzard_player_spells_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(player_spells_toc())
        .expect("Blizzard_PlayerSpells TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Player Spells"),
        "TOC must declare `## Title: Blizzard Player Spells` exactly — UNUSUAL: the \
         title uses the space-and-prose form rather than the underscore-namespace \
         spelling. Matches the Blizzard_PartyPoseUI / Blizzard_TimeManager pattern"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly"
    );
    assert!(
        raw.contains(
            "## Dependencies: Blizzard_SharedTalentUI, Blizzard_PagedContent, Blizzard_HelpPlate"
        ),
        "TOC must declare the 3-entry comma-separated `## Dependencies:` list exactly"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad:` — Game-only is the default behavior when \
         the key is omitted"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless mirror"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare `## RequiredDep:` or `## RequiredDeps:` — uses the \
         singular-`Dependencies` key only, unlike Blizzard_PlayerChoice which ships both \
         keys redundantly"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:` — talent / spellbook UI runs \
         in normal globals; spell-cast actions go through the secure ActionButton path \
         which is gated by other addons"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — minimal-metadata pattern"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
}

#[test]
fn blizzard_player_spells_toc_lists_bootstrap_then_twenty_six_files_in_canonical_order() {
    let toc = TocFile::from_file(&player_spells_toc()).expect("Blizzard_PlayerSpells TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PLAYER_SPELLS_TOC_FILES,
        "Retail 12.1.0.69497 lists the PlayerSpells bootstrap first, then twenty-six source \
         files across ClassSpecializations, ClassTalents, PvPTalents, SpellBook, and the \
         root-level frame, registration, and localization entries"
    );
}

#[test]
fn blizzard_player_spells_does_not_appear_in_eager_discovery_for_any_screen() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PlayerSpells");
        assert!(
            !found,
            "Blizzard_PlayerSpells must NOT appear in eager discovery for {screen:?} — \
             LoadOnDemand: 1 keeps the addon in the lod_pool only; deferred loading is \
             critical because PlayerSpells is the largest LoD addon and eagerly loading \
             it would dominate startup time"
        );
    }
}

#[test]
fn blizzard_player_spells_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PlayerSpells");
    assert!(
        found,
        "Blizzard_PlayerSpells must appear in `discover_all_blizzard_addons` — the full \
         inventory is a structural listing of every parseable TOC including LoD addons"
    );
}

#[test]
fn blizzard_player_spells_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_with_player_spells();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PlayerSpells')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PlayerSpells') must return true after \
         explicit load_addon — the addon is LoadOnDemand so only the explicit load \
         path makes IsAddOnLoaded report true"
    );
}

#[test]
fn blizzard_player_spells_dependencies_loaded_via_eager_discovery() {
    let env = load_full_game_ui_with_player_spells();

    for dep in REQUIRED_DEPS {
        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{dep}')"))
            .unwrap_or_else(|err| panic!("IsAddOnLoaded('{dep}') probe failed: {err}"));
        assert!(
            loaded,
            "C_AddOns.IsAddOnLoaded('{dep}') must return true — all 3 of \
             PlayerSpells's declared deps are non-LoD eager-discovery addons. \
             Blizzard_SharedTalentUI declares `LoadOnDemand: 0`; Blizzard_PagedContent \
             and Blizzard_HelpPlate omit LoadOnDemand and use `AllowLoad: Both` / \
             `AllowLoad: both` so they're picked up by the eager Game-screen sweep \
             before PlayerSpells's explicit load fires"
        );
    }
}

#[test]
fn blizzard_player_spells_creates_named_non_virtual_frames() {
    let env = load_full_game_ui_with_player_spells();

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata (FrameRef reports as `'table'` \
             via the custom __type metamethod). PlayerSpells ships exactly 5 named \
             non-virtual top-level frames (all parent=UIParent, hidden=true at load): \
             PlayerSpellsFrame (the panel root, mixin=PlayerSpellsFrameMixin, inherits \
             PortraitFrameTemplate + TabSystemOwnerTemplate, toplevel=true, \
             enableMouse=true — Blizzard_PlayerSpellsFrame.xml:5); \
             ClassTalentLoadoutCreateDialog / EditDialog / ImportDialog (the 3 modal \
             dialogs for talent loadout creation / rename / import-from-string-paste, \
             each inherits ClassTalentLoadoutDialogTemplate); \
             HeroTalentsSelectionDialog (mixin=HeroTalentsSelectionMixin, \
             frameStrata=DIALOG, inherits DefaultPanelTemplate — the modal overlay for \
             picking a hero spec subtree)"
        );

        let name: String = env
            .eval(&format!("return _G.{frame}:GetName()"))
            .unwrap_or_else(|err| panic!("_G.{frame}:GetName() probe failed: {err}"));
        assert_eq!(
            name, *frame,
            "_G.{frame}:GetName() must round-trip the same name"
        );
    }
}

#[test]
fn class_talent_load_system_inherits_dropdown_template_mixin_methods() {
    let env = load_full_game_ui_with_player_spells();

    let get_dropdown_types: (String, String) = env
        .eval(
            "local direct = CreateFrame('Frame', 'DropdownLoadSystemProbe', UIParent, \
             'DropdownLoadSystemTemplate'); \
             return type(direct.GetDropdown), \
                    type(PlayerSpellsFrame.TalentsFrame.LoadSystem.GetDropdown)",
        )
        .expect("ClassTalents LoadSystem GetDropdown probe should succeed");
    assert_eq!(
        get_dropdown_types,
        ("function".to_string(), "function".to_string()),
        "DropdownLoadSystemTemplate must apply DropdownLoadSystemMixin both to direct \
         CreateFrame instances and nested XML children such as ClassTalentsFrame.LoadSystem"
    );
}

#[test]
fn blizzard_player_spells_panel_root_is_hidden_after_load() {
    let env = load_full_game_ui_with_player_spells();

    let visible: bool = env
        .eval("return PlayerSpellsFrame:IsShown()")
        .expect("PlayerSpellsFrame:IsShown() probe succeeds");
    assert!(
        !visible,
        "PlayerSpellsFrame must be hidden at load — `<Frame ... hidden=\"true\">` in \
         Blizzard_PlayerSpellsFrame.xml stamps the panel as hidden. The panel only \
         shows when the player presses N/P or the engine fires \
         PlayerSpellsUtil.ToggleSpellBookFrame()"
    );
}

#[test]
fn blizzard_player_spells_publishes_root_panel_mixins() {
    let env = load_full_game_ui_with_player_spells();

    for mixin in ROOT_PANEL_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — these are the 4 root-tab mixins that \
             back the major panel sections: PlayerSpellsFrameMixin (the parent panel \
             owning the TabSystem), ClassSpecFrameMixin (Specialization tab content), \
             ClassTalentsFrameMixin (Talents tab content — the talent tree), \
             SpellBookFrameMixin (SpellBook tab content). Each is referenced by an XML \
             `mixin=\"...\"` attribute on a top-level Frame inside its respective tab \
             content XML"
        );
    }
}

#[test]
fn blizzard_player_spells_publishes_class_talent_button_mixins() {
    let env = load_full_game_ui_with_player_spells();

    for mixin in TALENT_BUTTON_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — these 9 talent-button mixins back the \
             talent-tree node primitives: Base + Art (the visual node), \
             Spend / Select / SplitSelect (the 3 node-input variants depending on \
             whether the node is a single-rank, single-pick, or split-pick choice), \
             CapstonePip / CapstoneWithTrack (the per-tier capstone slot art at the \
             bottom of the tree), EdgeArrow (the directional arrow connecting prereq \
             nodes to dependents), and SelectionChoice (the choice-row used by hero \
             talent picker)"
        );
    }
}

#[test]
fn blizzard_player_spells_publishes_hero_talent_mixins() {
    let env = load_full_game_ui_with_player_spells();

    for mixin in HERO_TALENT_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — these 7 hero-talent mixins back the \
             hero-spec subtree feature added in TWW (10.2.5): HeroSpecButton (the \
             pickable per-spec button at the top of the dialog), HeroTalentsContainer \
             (the embedded sub-frame inside ClassTalentsFrame that hosts the active \
             hero spec's tree), HeroTalentSpecContent (the per-spec content view), \
             HeroTalentsSelection (the modal dialog mixin), HeroTalentActivateButton \
             (the `Activate` button at the bottom of the dialog), \
             HeroTalentCollapseButton (the chevron that collapses the embedded tree to \
             save vertical space), HeroTalentsUnlockedAnimFrame (the unlock animation \
             that plays when the player crosses level 71)"
        );
    }
}

#[test]
fn blizzard_player_spells_publishes_spell_book_mixins() {
    let env = load_full_game_ui_with_player_spells();

    for mixin in SPELL_BOOK_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — these 11 spellbook mixins back the \
             post-10.0 redesigned SpellBook tab: Frame + Tutorials root, Item + \
             ItemButton (the per-spell entry), Header (the per-category section \
             header), CategoryTab (the left-side spec/general/pet selector tabs), \
             Search (the inline search box at the top), and 4 category-classification \
             mixins (Base + General + Class + Pet) that drive what content shows up in \
             each tab"
        );
    }
}

#[test]
fn blizzard_player_spells_publishes_loadout_dialog_mixins() {
    let env = load_full_game_ui_with_player_spells();

    for mixin in LOADOUT_DIALOG_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — these 10 loadout-dialog mixins back \
             the talent loadout management dialogs: 1 base \
             (ClassTalentLoadoutDialogMixin), 3 dialog variants (Create/Edit/Import), \
             4 name-control variants (Base + Create/Edit/Import each with their own \
             validation rules), 1 input-control base, and 1 import-control \
             (the import-string paste box on the Import dialog). The 4-name-control \
             split exists because each dialog applies different name validation \
             (Create: must be unique; Edit: must be unique except current; Import: \
             must be unique vs server state)"
        );
    }
}

#[test]
fn blizzard_player_spells_publishes_pvp_and_warmode_mixins() {
    let env = load_full_game_ui_with_player_spells();

    for mixin in PVP_AND_WARMODE_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — these 6 mixins back the PvP talents \
             selection UI (List + ListButton + SlotTray + SlotButton — the 3-slot PvP \
             talent picker shown in the lower portion of the Talents tab) and the \
             Warmode toggle (WarmodeButton + WarmodeIncentive — the `War Mode` opt-in \
             checkbox + the `+10% bonus rewards` incentive label that shows when \
             Warmode is OFF in the player's current PvP region)"
        );
    }
}

#[test]
fn blizzard_player_spells_publishes_player_spells_util_namespace() {
    let env = load_full_game_ui_with_player_spells();

    let kind: String = env
        .eval("return type(_G.PlayerSpellsUtil)")
        .expect("type(_G.PlayerSpellsUtil) probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.PlayerSpellsUtil must publish as a table — declared by \
         Blizzard_FrameXMLUtil/Mainline/PlayerSpellsUtil.lua (which loads as part of \
         the eager FrameXML pass BEFORE Blizzard_PlayerSpells's explicit LoD load) AND \
         by runtime_surface_bootstrap.lua as a fallback. The namespace exposes the \
         keybind-driven entry-point helpers (ToggleSpellBookFrame, ToggleTalentFrame, \
         OpenToSpec, OpenToHeroSpec, etc.) that the `KEYBINDING_HEADER_TALENTS` / \
         `OPENSPELLBOOK` keybind handlers call when the player presses N/P. \
         PlayerSpells's own Lua does NOT redefine PlayerSpellsUtil — it only extends \
         the existing namespace with additional methods"
    );

    let toggle_helper_present: bool = env
        .eval("return type(PlayerSpellsUtil.ToggleSpellBookFrame) == 'function'")
        .expect("ToggleSpellBookFrame probe succeeds");
    assert!(
        toggle_helper_present,
        "PlayerSpellsUtil.ToggleSpellBookFrame must be a function — the canonical \
         spellbook-keybind entry point. The existing \
         tests/test_showuipanel_lod_player_spells.rs uses it to drive the spellbook \
         open-flow"
    );
}

#[test]
fn blizzard_player_spells_loads_without_unrelated_lua_errors() {
    let env = load_full_game_ui_with_player_spells();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PlayerSpells")
                || message.contains("PlayerSpellsFrame")
                || message.contains("PlayerSpellsRegistration")
                || message.contains("ClassTalentsFrame")
                || message.contains("ClassSpecFrame")
                || message.contains("SpellBookFrame")
        })
        .filter(|message| {
            let touches_three_d_model_gap = message.contains("ScriptAnimatedModelSceneTemplate")
                || message.contains("ModelScene");
            !touches_three_d_model_gap
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PlayerSpells emitted addon-specific Lua errors during load (excluding \
         the documented 3D ModelScene permanent gap):\n  {}",
        load_errors.join("\n  ")
    );
}
