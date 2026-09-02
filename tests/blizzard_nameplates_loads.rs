#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addons_for_screen, find_toc_file, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn nameplates_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_NamePlates")
}

fn nameplates_toc() -> PathBuf {
    nameplates_dir().join("Blizzard_NamePlates.toc")
}

const NAMEPLATES_RETAIL_TOC_FILES: &[&str] = &[
    "Blizzard_NamePlateConstants.lua",
    "Blizzard_NamePlateFrameOptions.lua",
    "Blizzard_NamePlateComponent.lua",
    "Blizzard_NamePlateAuras.lua",
    "Blizzard_NamePlateAuras.xml",
    "Blizzard_NamePlateHealthBar.lua",
    "Blizzard_NamePlateCastingBar.lua",
    "Blizzard_NamePlateCastingBar.xml",
    "Blizzard_NamePlateClassificationFrame.lua",
    "Blizzard_NamePlateRaidTarget.lua",
    "Blizzard_NamePlateUnitFrame.lua",
    "Blizzard_NamePlateBase.lua",
    "Blizzard_NamePlates.lua",
    "Blizzard_NamePlates.xml",
    "Mainline/Blizzard_ClassNameplateBar.lua",
    "Mainline/Blizzard_ClassNameplateBar.xml",
    "Mainline/Blizzard_ClassNameplateAlternatePowerBarBase.lua",
    "Mainline/Blizzard_ClassNameplateAlternatePowerBarBase.xml",
    "Mainline/Blizzard_ClassNameplateBar_Paladin.lua",
    "Mainline/Blizzard_ClassNameplateBar_Paladin.xml",
    "Mainline/Blizzard_ClassNameplateBar_DeathKnight.lua",
    "Mainline/Blizzard_ClassNameplateBar_DeathKnight.xml",
    "Mainline/Blizzard_ClassNameplateBar_Dracthyr.lua",
    "Mainline/Blizzard_ClassNameplateBar_Dracthyr.xml",
    "Mainline/Blizzard_ClassNameplateBar_Rogue.lua",
    "Mainline/Blizzard_ClassNameplateBar_Rogue.xml",
    "Mainline/Blizzard_ClassNameplateBar_Druid.lua",
    "Mainline/Blizzard_ClassNameplateBar_Druid.xml",
    "Mainline/Blizzard_ClassNameplateBar_Mage.lua",
    "Mainline/Blizzard_ClassNameplateBar_Mage.xml",
    "Mainline/Blizzard_ClassNameplateBar_Monk.lua",
    "Mainline/Blizzard_ClassNameplateBar_Monk.xml",
    "Mainline/Blizzard_ClassNameplateBar_Warlock.lua",
    "Mainline/Blizzard_ClassNameplateBar_Warlock.xml",
];

const WOWHACK_GATED_FILES: &[&str] = &[
    "Blizzard_ClassNameplateBar_AlternatePower.lua",
    "Blizzard_ClassNameplateBar_AlternatePower.xml",
];

const PUBLIC_DRIVER_AND_BASE_MIXINS: &[&str] = &[
    "NamePlateDriverMixin",
    "NamePlateBaseMixin",
    "NamePlateComponentMixin",
    "NamePlateUnitFrameMixin",
    "NamePlateAurasMixin",
    "NamePlateAuraItemMixin",
    "NamePlateClassificationFrameMixin",
    "NamePlateCastingBarMixin",
    "NamePlateHealthBarMixin",
    "NamePlateRaidTargetMixin",
    "NamePlateBorderTemplateMixin",
    "NamePlateScriptBaseMixin",
    "ClassNameplateAlternatePowerBarBaseMixin",
];

const PUBLIC_CLASS_BAR_GLOBALS: &[&str] = &[
    "ClassNameplateBar",
    "ClassNameplateManaBar",
    "ClassNameplateBarPaladin",
    "ClassNameplateBarDeathKnight",
    "ClassNameplateBarDracthyr",
    "ClassNameplateBarRogue",
    "ClassNameplateBarFeralDruid",
    "ClassNameplateBarMage",
    "ClassNameplateBarBrewmasterMonk",
    "ClassNameplateBarWindwalkerMonk",
    "ClassNameplateBarWarlock",
];

const FILE_PRIVATE_LOCALS_THAT_MUST_NOT_LEAK: &[&str] = &[
    "BuffCompare",
    "GetAuraFrameHeight",
    "GetHealthBarHeight",
    "IsUnitNameInsideHealthBar",
];

const WOWHACK_ONLY_GLOBAL: &str = "ClassNameplateBarAlternatePowerMixin";

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
fn blizzard_nameplates_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&nameplates_dir()).expect("Blizzard_NamePlates TOC resolves");
    assert_eq!(
        resolved,
        nameplates_toc(),
        "Blizzard_NamePlates ships exactly one bare TOC — no `_Mainline.toc` and no \
         `_Classic.toc`. The unit-nameplate driver is a Game-screen-only feature whose retail \
         shape is fully described by the bare TOC plus per-file `[AllowLoadGameType wowhack]` \
         tags. `find_toc_file` resolves the bare TOC after the `_Mainline.toc` lookup misses"
    );

    let mainline = nameplates_dir().join("Blizzard_NamePlates_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — Blizzard_NamePlates expresses its \
         retail-only shape via per-file annotations, not via separate flavor TOCs",
        mainline.display()
    );
}

#[test]
fn blizzard_nameplates_toc_declares_eager_load_with_two_required_deps() {
    let toc = TocFile::from_file(&nameplates_toc()).expect("Blizzard_NamePlates TOC parses");
    assert!(
        !toc.is_load_on_demand(),
        "TOC omits `## LoadOnDemand`, so the unit-nameplate driver eager-loads. \
         NAME_PLATE_CREATED / NAME_PLATE_UNIT_ADDED events fire during world entry; the \
         driver mixin's OnLoad must be wired before PLAYER_ENTERING_WORLD"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_UIWidgets".to_string(),
            "Blizzard_UnitFrame".to_string(),
        ],
        "Repeated `## Dep:` lines declare UIWidgets and UnitFrame as hard \
         dependencies in source order"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Current retail TOC declares no optional dependencies"
    );

    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the nameplate driver mirrors live unit state and re-derives \
         every plate per-session from server-provided NAME_PLATE_UNIT_ADDED events. There is \
         no client-side persisted nameplate state worth checkpointing across login"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` at the addon level — the addon itself loads on \
         every game type; only the AlternatePower bar is gated to wowhack via per-file \
         annotations. `is_game_type_restricted()` at src/toc.rs:294 returns false when the \
         metadata key is absent"
    );
}

#[test]
fn blizzard_nameplates_toc_filters_wowhack_files_during_retail_parse() {
    let toc = TocFile::from_file(&nameplates_toc()).expect("Blizzard_NamePlates TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, NAMEPLATES_RETAIL_TOC_FILES,
        "Retail 12.1.0.69497 lists 34 retail files, including CastingBar.xml and Mainline/ \
         class-bar paths. The two WoWHack-only AlternatePower files remain filtered from the \
         retail parsed file list"
    );

    for wowhack in WOWHACK_GATED_FILES {
        let dropped = listed.iter().all(|f| !f.ends_with(wowhack));
        assert!(
            dropped,
            "TOC must NOT list `{wowhack}` after retail parse — its `[AllowLoadGameType \
             wowhack]` annotation should drop it. WoWHack is an internal-test game mode (the \
             AlternatePower bar is a WoW-Labs / hack-week experiment); on retail it must not \
             load. Listed files: {listed:?}"
        );
    }
}

#[test]
fn blizzard_nameplates_toc_omits_load_on_demand_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(nameplates_toc()).expect("Blizzard_NamePlates TOC reads as utf-8");
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must omit `## LoadOnDemand`; omission is the current retail eager-load \
         spelling for Blizzard_NamePlates"
    );
    assert!(
        raw.contains("[AllowLoadGameType wowhack]"),
        "TOC must contain at least one `[AllowLoadGameType wowhack]` annotation — the \
         AlternatePower bar's two files (.lua + .xml) carry it. The token `wowhack` is the \
         signal that drives the `is_allowed_game_type` filter at src/toc.rs:43-57 to drop the \
         pair on retail parse"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` (the colon variant — distinct from per-file \
         `[AllowLoadGameType ...]` annotations). When `## AllowLoad:` is omitted, \
         `allows_screen` at src/toc.rs:311 defaults to Game-only (`screen == \
         ScreenKind::Game`). Nameplates are a world-only surface — they have no role on \
         glue screens"
    );
}

#[test]
fn blizzard_nameplates_auto_discovers_on_game_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_NamePlates");
    assert!(
        game_found,
        "Blizzard_NamePlates must auto-discover on the Game screen — the unit-nameplate driver \
         is part of the eager retail boot set; it is not LoadOnDemand and it has no \
         AllowLoadGameType restriction at the addon level"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_NamePlates");
        assert!(
            !found,
            "Blizzard_NamePlates must NOT auto-discover on glue screens. With `## AllowLoad:` \
             omitted, `allows_screen` at src/toc.rs:311 defaults to Game-only. There are no \
             nameplates on Login / CharacterSelect / CharacterCreate — those screens render no \
             world units. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn blizzard_nameplates_appears_in_discover_all_blizzard_addons() {
    let all = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = all.iter().any(|(name, _)| name == "Blizzard_NamePlates");
    assert!(
        found,
        "Blizzard_NamePlates must appear in `discover_all_blizzard_addons` — that helper \
         enumerates every `Blizzard_*` directory regardless of LOD or screen restriction. The \
         addon-management UI relies on this exhaustive sweep to render the addon list"
    );
}

prefork_full_ui_case! {
fn blizzard_nameplates_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_NamePlates")
                || message.contains("NamePlateDriver")
                || message.contains("NamePlateBase")
                || message.contains("NamePlateAuras")
                || message.contains("ClassNameplateBar")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_NamePlates emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_nameplates_is_addon_loaded_after_auto_discovery(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_NamePlates')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_NamePlates') must return true after the eager \
         auto-discovery sweep — proves the unit-nameplate driver registers with the loaded-set \
         during the standard Game-screen boot pipeline, no explicit load_addon call required"
    );
}
}

prefork_full_ui_case! {
fn blizzard_nameplates_publishes_driver_frame_and_constants_table(env: &WowLuaEnv) {

    let driver_kind: String = env
        .eval("return type(NamePlateDriverFrame)")
        .expect("NamePlateDriverFrame probe succeeds");
    assert_eq!(
        driver_kind, "table",
        "_G.NamePlateDriverFrame must publish as a userdata-backed table — XML declares the \
         frame as `<Frame name=\"NamePlateDriverFrame\" toplevel=\"true\" \
         mixin=\"NamePlateDriverMixin\">`. Without the named frame, OnEvent / RegisterEvent \
         wiring against NAME_PLATE_CREATED has no surface to live on"
    );

    let constants_kind: String = env
        .eval("return type(NamePlateConstants)")
        .expect("NamePlateConstants probe succeeds");
    assert_eq!(
        constants_kind, "table",
        "_G.NamePlateConstants must publish as a table — Blizzard_NamePlateConstants.lua \
         declares it at file scope as `NamePlateConstants = {{ ... }}`. Other files in the \
         load order (notably Blizzard_NamePlates.lua line 1) reference \
         NamePlateConstants.SOFT_TARGET_NAMEPLATE_SIZE_CVAR before any function definition; \
         if the table is nil the file fails at module top-level"
    );

    let cvar_field: String = env
        .eval("return type(NamePlateConstants.NAME_PLATE_SCALES)")
        .expect("NAME_PLATE_SCALES probe succeeds");
    assert_eq!(
        cvar_field, "table",
        "NamePlateConstants.NAME_PLATE_SCALES must be a table keyed by Enum.NamePlateSize \
         values. Each entry holds horizontal / vertical / classification / aura / \
         aggroHighlight scale floats; the driver mixin reads these on every \
         OnNamePlateResized to compute the per-plate scale"
    );
}
}

prefork_full_ui_case! {
fn blizzard_nameplates_publishes_all_driver_and_base_mixins(env: &WowLuaEnv) {

    for mixin in PUBLIC_DRIVER_AND_BASE_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table after Blizzard_NamePlates loads. The driver / \
             component / aura / classification / casting / health / raid-target / border / \
             script-base mixins are all declared at file-top-level (`{mixin} = {{}}` or \
             `CreateFromMixins(...)`); each is the seed for a virtual XML template's mixin \
             attribute"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_nameplates_publishes_all_class_bar_globals(env: &WowLuaEnv) {

    for global in PUBLIC_CLASS_BAR_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{global})"))
            .unwrap_or_else(|err| panic!("type(_G.{global}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{global} must publish as a table — class-specific resource bars are addressed \
             by name from NamePlateDriverMixin:SetupClassNameplateBars (Blizzard_NamePlates.\
             lua line 241). Each class bar file declares `{global} = {{}}` (or \
             `Mixin({{}}, ParentMixin)`) at file-top-level. Missing globals would break \
             driver-side dispatch for that class"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_nameplates_does_not_leak_wowhack_only_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval(&format!("return type(_G.{WOWHACK_ONLY_GLOBAL})"))
        .expect("WOWHACK_ONLY_GLOBAL probe succeeds");
    assert_eq!(
        kind, "nil",
        "_G.{WOWHACK_ONLY_GLOBAL} must remain nil on retail. The mixin is declared only in \
         `Blizzard_ClassNameplateBar_AlternatePower.lua`, which carries the per-file \
         `[AllowLoadGameType wowhack]` annotation. The TOC parser drops both the .lua and \
         .xml at retail parse time (`is_allowed_game_type` at src/toc.rs:43-57 rejects the \
         `wowhack` token), so the file never executes and the global never publishes — proving \
         the per-file gate is wired through to runtime visibility"
    );
}
}

prefork_full_ui_case! {
fn blizzard_nameplates_does_not_leak_file_local_helpers_to_globals(env: &WowLuaEnv) {

    for symbol in FILE_PRIVATE_LOCALS_THAT_MUST_NOT_LEAK {
        let kind: String = env
            .eval(&format!("return type(_G.{symbol})"))
            .unwrap_or_else(|err| panic!("type(_G.{symbol}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{symbol} must remain nil — it is declared `local function {symbol}(...)` at \
             file scope inside Blizzard_NamePlates.lua / Blizzard_NamePlateAuras.lua. The \
             helpers are intentionally module-private; if they leak to _G they conflict with \
             addon namespaces and break the file-encapsulation contract"
        );
    }
}
}
