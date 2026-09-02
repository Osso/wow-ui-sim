use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn ui_panels_game_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UIPanels_Game")
}

fn ui_panels_game_mainline_toc() -> PathBuf {
    ui_panels_game_dir().join("Blizzard_UIPanels_Game_Mainline.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &[
    "Blizzard_FrameXML",
    "Blizzard_EditMode",
    "Blizzard_GameTooltip",
    "Blizzard_UIPanelTemplates",
    "Blizzard_POIButton",
    "Blizzard_MoneyFrame",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_Colors",
    "Blizzard_HelpPlate",
    "Blizzard_TransmogShared",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_GameMenuEsc",
];

const REPRESENTATIVE_MIXINS: &[&str] = &[
    "CharacterFrameMixin",
    "CharacterFrameTabButtonMixin",
    "PaperDollItemSlotButtonMixin",
    "LootFrameMixin",
    "FogOfWarFrameMixin",
    "GossipFrameMixin",
    "CampaignHeaderMixin",
    "PlayerCastingBarMixin",
    "PlayerInteractionFrameManagerMixin",
    "AzeritePaperDollItemOverlayMixin",
    "WorldMapPOIQuantizerMixin",
    "RoleSelectionMixin",
    "ItemRefTooltipMixin",
    "MapLegendMixin",
    "UnitPositionFrameMixin",
];

const REPRESENTATIVE_NAMED_FRAMES: &[&str] = &[
    "CharacterFrame",
    "PaperDollFrame",
    "WorldMapFrame",
    "QuestFrame",
    "MerchantFrame",
    "BankFrame",
    "TabardFrame",
    "GossipFrame",
    "MailFrame",
];

const REPRESENTATIVE_GLOBAL_TABLES: &[&str] =
    &["HUNTER_PET_BONUS", "WARLOCK_PET_BONUS", "TaxiButtonTypes"];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved = find_toc_file(&ui_panels_game_dir()).expect("UIPanels_Game TOC resolves");
    assert_eq!(
        resolved,
        ui_panels_game_mainline_toc(),
        "find_toc_file at src/loader/mod.rs:65-95 prefers \
         `<addon>_Mainline.toc` over the bare TOC. The directory \
         carries TWO flavor-suffixed TOCs (Blizzard_UIPanels_Game_Mainline.toc \
         and Blizzard_UIPanels_Game_Classic.toc) — _Classic is in the \
         fallback ladder but _Mainline wins on the first variants pass"
    );
}

#[test]
fn toc_is_eager_with_thirteen_dependencies() {
    let toc = TocFile::from_file(&ui_panels_game_mainline_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded on Game. The \
         in-world UI panels (CharacterFrame, MerchantFrame, BankFrame, \
         WorldMapFrame, etc.) must be live at world entry — most are \
         registered with the panel-manager via UIPanelWindows assignments \
         that fire at module-load"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        TOC_DEPENDENCIES.len(),
        "Mainline TOC must declare exactly {} hard deps. Got {}: {:?}",
        TOC_DEPENDENCIES.len(),
        deps.len(),
        deps
    );
    for expected in TOC_DEPENDENCIES {
        assert!(
            deps.iter().any(|d| d == expected),
            "TOC must declare `{expected}` — UIPanels_Game leans on \
             FrameXML/EditMode for the panel-manager + edit-mode hooks, \
             GameTooltip + MoneyFrame + UIPanelTemplates for shared \
             widgets, POIButton/Colors/HelpPlate for map+quest UI, \
             AccessibilityTemplates for keyboard-nav scaffolding, and \
             TransmogShared so DressUpFrames can link out to the \
             wardrobe. Got: {deps:?}"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_game_restricts_to_in_world() {
    let toc = TocFile::from_file(&ui_panels_game_mainline_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) hits the `eq_ignore_ascii_case` \
         branch at toc.rs:308 → Game-only. Glue screens have no quest \
         log, no character sheet, no world map — the addon would be \
         meaningless"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: \
             game` matches only the Game variant via toc.rs:308"
        );
    }
}

#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&ui_panels_game_mainline_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` lists `mainline` which is \
         recognised as a non-restricting flavor at toc.rs:294-302 \
         (standard|mainline). The companion _Classic.toc has \
         `AllowLoadGameType: classic` which IS restricted out on a \
         mainline build, but find_toc_file already picked _Mainline.toc"
    );
}

#[test]
fn classic_toc_uses_load_first_directive() {
    let classic_toc_path = ui_panels_game_dir().join("Blizzard_UIPanels_Game_Classic.toc");
    let classic = TocFile::from_file(&classic_toc_path).expect("Classic TOC parses");

    assert!(
        classic.is_load_first(),
        "The _Classic variant carries `## LoadFirst: 1` (mainline does \
         NOT) — Classic builds need the in-world panels available before \
         everything else because the Classic dependency graph differs \
         (e.g. Blizzard_ActionBar declares Blizzard_UIPanels_Game as a \
         hard dep on Classic but not on Mainline)"
    );
    assert!(
        classic.is_game_type_restricted(),
        "_Classic.toc has `AllowLoadGameType: classic` which IS restricted out — toc.rs:294-302 only treats `mainline`/`standard` as non-restricting"
    );
}

#[test]
fn toc_raw_bytes_pin_six_directives_and_localization_trailer() {
    let raw = std::fs::read_to_string(ui_panels_game_mainline_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_UIPanels_Game",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## Dependencies: Blizzard_FrameXML, Blizzard_EditMode, Blizzard_GameTooltip, Blizzard_UIPanelTemplates, Blizzard_POIButton, Blizzard_MoneyFrame, Blizzard_AccessibilityTemplates, Blizzard_Colors, Blizzard_HelpPlate, Blizzard_TransmogShared, Blizzard_FrameXMLUtil, Blizzard_ManagedFrameSystem, Blizzard_GameMenuEsc",
        "## AllowLoad: game",
        "## AllowLoadGameType: mainline",
        "Mainline\\Localization.lua",
        "Mainline\\CharacterFrame.lua",
        "Mainline\\WorldMapFrame.lua",
        "Mainline\\QuestFrame.lua",
        "Mainline\\MerchantFrame.lua",
        "Mainline\\BankFrame.lua",
        "Shared\\PlayerInteractionFrameManager.lua",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — body files are split across \
             two subdirs (Shared\\ for cross-flavor utilities like \
             CastingBarFrame, ExtraAbilityContainer, TaxiFrame, \
             SocialQueue, VehicleSeatIndicator, WorldMapPOIQuantizer, \
             PlayerInteractionFrameManager and Mainline\\ for \
             flavor-specific panels). The Localization.lua trailer must \
             load LAST so its localized strings overlay any defaults the \
             panels declared at module-load"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("[Family]"));
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_UIPanels_Game");
    assert!(
        found,
        "Blizzard_UIPanels_Game must appear in Game eager discovery — \
         non-LoD addon with AllowLoad:game. Many sibling addons declare \
         it as a hard dep: Blizzard_ActionBar, Blizzard_UnitFrame, \
         Blizzard_RaidFrame, Blizzard_ZoneAbility, \
         Blizzard_HUDInventoryTemplates, Blizzard_ItemBeltFrame, \
         Blizzard_EnvironmentCleanup, Blizzard_SpectateFrame, plus \
         Blizzard_GuildRename which RequiredDeps it"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_UIPanels_Game");
        assert!(
            !found,
            "Blizzard_UIPanels_Game must NOT appear on {screen:?} — \
             AllowLoad:game restricts to in-world via toc.rs:308, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_publishes_representative_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after full-game load. The \
             addon publishes ~50+ mixins across Mainline/ + Shared/ \
             body files; this probe pins a representative cross-section: \
             1 frame-level mixin per major panel (Character/PaperDoll/\
             Loot/FogOfWar/Gossip/Campaign/Casting/Interaction/\
             AzeritePaperDoll/WorldMapPOI/RoleSelection/ItemRef/MapLegend/\
             UnitPosition). GossipFrameMixin is built via \
             `CreateFromMixins(GossipFrameSharedMixin)` — that base \
             mixin lives in Blizzard_FrameXML so the dep wiring is \
             essential. CampaignHeaderMixin is \
             `CreateFromMixins(CampaignHeaderDisplayMixin)`"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_creates_representative_named_frames(env: &WowLuaEnv) {

    for frame in REPRESENTATIVE_NAMED_FRAMES {
        let exists: bool = env
            .eval(&format!("return {frame} ~= nil"))
            .unwrap_or_else(|err| panic!("{frame} probe failed: {err}"));
        assert!(
            exists,
            "{frame} must exist as a named global Frame after XML load. \
             The addon's XML body declares the major in-world panels: \
             CharacterFrame (PortraitFrame template), PaperDollFrame, \
             WorldMapFrame (the fullscreen map), QuestFrame, \
             MerchantFrame, BankFrame, TabardFrame, GossipFrame, \
             MailFrame. Each is registered via UIPanelWindows so the \
             panel-manager can position them"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_representative_globals(env: &WowLuaEnv) {

    for global in REPRESENTATIVE_GLOBAL_TABLES {
        let kind: String = env
            .eval(&format!("return type({global})"))
            .unwrap_or_else(|err| panic!("{global} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{global} must be a global table. HUNTER_PET_BONUS + \
             WARLOCK_PET_BONUS are stat-bonus lookup tables in \
             PaperDollFrame.lua. TaxiButtonTypes is the type registry \
             in Shared/TaxiFrame.lua used to switch button textures \
             between flight-master, special-master, and current-zone \
             nodes"
        );
    }
}
}

prefork_full_ui_case! {
fn world_map_frame_has_expected_methods(env: &WowLuaEnv) {

    for method in &["IsMaximized", "Show", "Hide", "OnShow", "OnHide"] {
        let exists: bool = env
            .eval(&format!(
                "return type(WorldMapFrame.{method}) == 'function'"
            ))
            .unwrap_or_else(|err| panic!("WorldMapFrame.{method} probe failed: {err}"));
        assert!(
            exists,
            "WorldMapFrame.{method} must be a function. WorldMapFrame is \
             built via CreateFromMixins on a frame-mixin chain that \
             includes the maximize-toggle behaviour, plus standard \
             frame Show/Hide. Confirms the mixin install ran"
        );
    }
}
}

prefork_full_ui_case! {
fn character_frame_has_tab_buttons(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(CharacterFrameTab1)")
        .expect("CharacterFrameTab1 probe");
    assert_eq!(
        kind, "table",
        "CharacterFrameTab1 must exist as a named tab Frame. \
         CharacterFrame.xml declares a horizontal strip of named tab \
         buttons (CharacterFrameTab1..N) using \
         CharacterFrameTabButtonMixin. Tab1 specifically is the always- \
         present Character paperdoll tab; later tabs (Reputation, Pets, \
         etc.) get added/removed by sibling addons"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UIPanels_Game/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load with UIPanels_Game in dependency order \
         must emit zero UIPanels_Game-body errors. ~85 body files \
         (lua + xml) covering every major in-world panel must register \
         cleanly; the lua files must tolerate the simulator's stubs for \
         Container/Quest/Merchant/Bank/Map/Trade/Gossip C-API surface \
         without raising. Found: {addon_specific:?}"
    );
}
}
