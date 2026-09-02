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

fn private_auras_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PrivateAurasUI")
}

fn private_auras_toc() -> PathBuf {
    private_auras_dir().join("Blizzard_PrivateAurasUI.toc")
}

const PRIVATE_AURAS_TOC_FILES: &[&str] = &[
    "Shared/PrivateAurasTooltip.lua",
    "Mainline/PrivateAurasTooltip.lua",
    "Mainline/PrivateAurasTooltip.xml",
    "Blizzard_PrivateAurasUI.lua",
    "Blizzard_PrivateAurasUI.xml",
    "PrivateAuraInit.lua",
];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "CompactDispelDebuffMixin",
    "PrivateAuraMixin",
    "PrivateAuraAnchorContainerMixin",
    "PrivateAuraAnchorSingleMixin",
    "CompactUnitFrameDispelOverlayMixin",
];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBAL_ENV: &[&str] = &[
    "PrivateAuraUnitWatcherTemplate",
    "PrivateAuraTemplate",
    "CompactDispelDebuffTemplate",
    "CompactUnitFrameDispelOverlayTemplate",
];

const SCOPED_NAMED_FRAMES_HIDDEN_FROM_GLOBAL_ENV: &[&str] =
    &["RaidBossEmoteFramePrivate", "PrivateAurasTooltip"];

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
fn blizzard_private_auras_ui_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&private_auras_dir()).expect("Blizzard_PrivateAurasUI TOC resolves");
    assert_eq!(
        resolved,
        private_auras_toc(),
        "Blizzard_PrivateAurasUI ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The addon hosts the in-world private-aura UI (raid-frame dispel overlays + \
         per-unit private aura buttons), all driven from C_UnitAurasPrivate callbacks, \
         a single bare TOC drives every flavor"
    );

    let mainline = private_auras_dir().join("Blizzard_PrivateAurasUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_private_auras_ui_toc_declares_secure_game_only_addon() {
    let toc = TocFile::from_file(&private_auras_toc()).expect("Blizzard_PrivateAurasUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on the Game screen so the \
         C_UnitAurasPrivate anchor callbacks are wired up before any unit-aura packet arrives"
    );
    assert!(!toc.is_load_first());
    assert!(
        toc.is_secure_env(),
        "TOC must declare `## UseSecureEnvironment: 1` — the addon manipulates secure \
         frames (raid-frame dispel overlays gated on protected unit-frame state) so its \
         compiled chunks must run in the secure environment. `is_secure_env` at \
         src/toc.rs:237-242 returns true when this flag is present"
    );

    assert!(
        !toc.is_game_type_restricted(),
        "TOC must NOT declare `## AllowLoadGameType:` — private auras are universal across \
         retail / classic / etc."
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) must enable Game screen — `allows_screen` at \
         src/toc.rs:306-313 uses case-insensitive comparison so the lowercase value matches"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: game` must EXCLUDE {screen:?} — private auras only matter \
             once unit frames exist, glue screens have no UnitID surface to anchor to"
        );
    }
}

#[test]
fn blizzard_private_auras_ui_toc_uses_singular_dep_form() {
    let toc = TocFile::from_file(&private_auras_toc()).expect("Blizzard_PrivateAurasUI TOC parses");

    let dependencies = toc.dependencies();
    assert_eq!(
        dependencies,
        [
            "Blizzard_SharedXMLGame",
            "Blizzard_FrameXMLUtil",
            "Blizzard_GameTooltip",
            "Blizzard_RaidWarning",
            "Blizzard_VisualAlerts",
        ],
        "Retail 12.1.0.69497 declares five hard dependencies through singular `## Dep:` lines"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — the addon's only soft surface is C_UnitAurasPrivate \
         (a C namespace, not an addon dep)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — anchor state is owned by the C_UnitAurasPrivate engine \
         side, the Lua side is a stateless view"
    );
}

#[test]
fn blizzard_private_auras_ui_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(private_auras_toc())
        .expect("Blizzard_PrivateAurasUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PrivateAurasUI"),
        "TOC must declare `## Title: Blizzard_PrivateAurasUI` exactly"
    );
    assert!(
        raw.contains("## UseSecureEnvironment: 1"),
        "TOC must declare `## UseSecureEnvironment: 1` — secures the addon environment so \
         protected raid-frame manipulation does not taint the secure dispatch path"
    );
    assert!(
        raw.contains("## AllowLoad: game"),
        "TOC must declare `## AllowLoad: game` (lowercase exactly) — case-insensitive \
         matching means the lowercase form still routes through the Game-only screen gate"
    );

    for dep_line in [
        "## Dep: Blizzard_SharedXMLGame",
        "## Dep: Blizzard_FrameXMLUtil",
        "## Dep: Blizzard_GameTooltip",
        "## Dep: Blizzard_RaidWarning",
        "## Dep: Blizzard_VisualAlerts",
    ] {
        assert!(
            raw.contains(dep_line),
            "TOC must contain `{dep_line}` in raw bytes; the parser preserves singular \
             Dep entries as hard dependencies"
        );
    }

    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — it uses the non-canonical singular \
         `## Dep:` form instead (likely a Blizzard internal build-tool artifact)"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare `## RequiredDep:` or `## RequiredDeps:`"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on Game screen"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless C-API view"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — minimal header on this internal addon"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
}

#[test]
fn blizzard_private_auras_ui_toc_lists_files_in_order() {
    let toc = TocFile::from_file(&private_auras_toc()).expect("Blizzard_PrivateAurasUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PRIVATE_AURAS_TOC_FILES,
        "TOC body must list tooltip setup first, then Blizzard_PrivateAurasUI.lua \
         (publishes 5 mixin globals + addon-private PrivateAuras.unitWatchers / \
         PrivateAuras.PrivateAuraUnitWatcher bindings), Blizzard_PrivateAurasUI.xml \
         (defines aura templates inside a `<ScopedModifier forbidden hideFromGlobalEnv>` \
         wrapper), and finally PrivateAuraInit.lua (wires C_UnitAurasPrivate callbacks). \
         Order matters: PrivateAuraInit.lua must run last because it asserts \
         `PrivateAuras.unitWatchers ~= nil` which the .lua chunk creates"
    );
}

#[test]
fn blizzard_private_auras_ui_appears_only_in_game_screen_eager_discovery() {
    let ui = blizzard_ui_dir();

    let game = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game
        .iter()
        .any(|(name, _)| name == "Blizzard_PrivateAurasUI");
    assert!(
        game_found,
        "Blizzard_PrivateAurasUI must appear in eager discovery for ScreenKind::Game — \
         `## AllowLoad: game` opens the addon to Game screen only and there is no \
         LoadOnDemand filter to exclude it"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PrivateAurasUI");
        assert!(
            !found,
            "Blizzard_PrivateAurasUI must NOT appear in eager discovery for {screen:?} — \
             `## AllowLoad: game` excludes glue screens"
        );
    }
}

#[test]
fn blizzard_private_auras_ui_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PrivateAurasUI");
    assert!(
        found,
        "Blizzard_PrivateAurasUI must appear in `discover_all_blizzard_addons` — the \
         unfiltered inventory at src/loader/mod.rs lists every parseable Blizzard_* TOC \
         regardless of AllowLoad gating"
    );
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_is_addon_loaded_after_game_screen_boot(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PrivateAurasUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PrivateAurasUI') must return true after the \
         eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_publishes_five_mixins_into_secure_env(env: &WowLuaEnv) {

    for name in PUBLIC_MIXIN_GLOBALS {
        let in_secure_env: String = env
            .eval(&format!("return type(rawget(__secureenv, '{name}'))"))
            .unwrap_or_else(|err| panic!("rawget(__secureenv, '{name}') probe failed: {err}"));
        assert_eq!(
            in_secure_env, "table",
             "__secureenv.{name} must publish as a table — Blizzard_PrivateAurasUI.lua \
             runs under `## UseSecureEnvironment: 1`, so mark_secure_state at \
             src/lua_api/globals/security/secure_env.rs:82 swaps the chunk's fenv to \
             __secureenv (a shallow _G copy without a live `_G` fallback). Plain \
             top-level assignments like `CompactDispelDebuffMixin = {{}}` therefore \
             write into __secureenv, not _G. The 5 mixins exposed are: \
             CompactDispelDebuffMixin (raid-frame dispel overlay debuff icon), \
             PrivateAuraMixin (per-aura button modeled on AuraButtonMixin with \
             OnLoad/Reset/ShowTooltip/OnEnter/OnLeave/OnUpdate), \
             PrivateAuraAnchorContainerMixin (per-unit anchor container that pools \
             PrivateAuraTemplate frames), PrivateAuraAnchorSingleMixin (single-anchor \
             variant), CompactUnitFrameDispelOverlayMixin (raid-frame fill+highlight \
             overlay using RaidFrame-Dispel-Fill / RaidFrame-DispelHighlight atlases)"
        );

        let in_plain_g: String = env
            .eval(&format!("return type(rawget(_G, '{name}'))"))
            .unwrap_or_else(|err| panic!("rawget(_G, '{name}') probe failed: {err}"));
        assert_eq!(
            in_plain_g, "nil",
            "rawget(_G, '{name}') must be nil — secure-env writes do NOT propagate back \
             to _G. Insecure code reaches these mixins via __secureenv directly, or \
             through the XML mixin attribute (resolved by the loader against both envs)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_virtual_templates_are_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBAL_ENV {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual XML templates are stored in the template \
             registry (consumed via inherits=...) and never become _G entries. The 4 \
             virtual templates here (PrivateAuraUnitWatcherTemplate, PrivateAuraTemplate, \
             CompactDispelDebuffTemplate, CompactUnitFrameDispelOverlayTemplate) define \
             the per-unit watcher Frame, the per-aura button (with Cooldown child \
             inheriting CooldownFrameTemplate, hideCountdownNumbers/reverse/useParentLevel), \
             the 14x14 dispel-debuff Texture (parentArray=dispelDebuffFrames), and the \
             dispel overlay Frame (frameLevel=200, with 3 inherited \
             CompactDispelDebuffTemplate textures). All wrapped in a `<ScopedModifier \
             forbidden=true hideFromGlobalEnv=true>` block"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_scoped_named_frames_stay_out_of_global_env(env: &WowLuaEnv) {

    for frame in SCOPED_NAMED_FRAMES_HIDDEN_FROM_GLOBAL_ENV {
        let kind: String = env
            .eval(&format!("return type(rawget(_G, '{frame}'))"))
            .unwrap_or_else(|err| panic!("rawget(_G, '{frame}') probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "rawget(_G, '{frame}') must be nil — the XML wraps \
             RaidBossEmoteFramePrivate and PrivateAurasTooltip in \
             `<ScopedModifier forbidden=\"true\" hideFromGlobalEnv=\"true\">`, so named \
             frames inside the scope are created without publishing global bindings"
        );
    }
}
}

#[test]
fn blizzard_private_auras_ui_xml_uses_scoped_modifier_wrapper() {
    let raw = std::fs::read_to_string(private_auras_dir().join("Blizzard_PrivateAurasUI.xml"))
        .expect("Blizzard_PrivateAurasUI.xml reads utf-8");

    assert!(
        raw.contains("<ScopedModifier forbidden=\"true\" hideFromGlobalEnv=\"true\">"),
        "XML must open the frame block with `<ScopedModifier forbidden=\"true\" \
         hideFromGlobalEnv=\"true\">` — this wrapper marks every contained frame as \
         forbidden (secure-restricted, IsForbidden() returns true) AND keeps them out of \
         _G even when they declare a `name=...` attribute. Handled by the simulator at \
         src/loader/xml_file.rs:82,103-106 via process_scoped_modifier"
    );
    assert!(
        raw.contains("</ScopedModifier>"),
        "XML must close the wrapper — every frame definition lives inside the single \
         ScopedModifier block"
    );

    assert!(
        raw.contains("inherits=\"AuraButtonArtTemplate\""),
        "PrivateAuraTemplate must inherit AuraButtonArtTemplate — the per-aura button \
         template reuses the standard aura-button art surface (icon, border, count \
         FontString, etc.) and only swaps in Private-aura-specific scripts via the \
         PrivateAuraMixin"
    );
    assert!(
        raw.contains("inherits=\"CooldownFrameTemplate\""),
        "PrivateAuraTemplate's Cooldown child must inherit CooldownFrameTemplate"
    );
    assert!(
        raw.contains("inherits=\"RaidBossEmoteFrameTemplate\""),
        "RaidBossEmoteFramePrivate must inherit RaidBossEmoteFrameTemplate"
    );
    let tooltip_raw =
        std::fs::read_to_string(private_auras_dir().join("Mainline/PrivateAurasTooltip.xml"))
            .expect("Mainline/PrivateAurasTooltip.xml reads utf-8");
    assert!(
        tooltip_raw.contains("inherits=\"SharedTooltipArtTemplate\""),
        "PrivateAurasTooltip must inherit SharedTooltipArtTemplate — the standard tooltip \
         art surface (border, NineSlice background) reused by every Blizzard tooltip"
    );
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PrivateAurasUI")
                || message.contains("PrivateAura")
                || message.contains("CompactDispelDebuff")
                || message.contains("CompactUnitFrameDispelOverlay")
                || message.contains("PrivateAurasTooltip")
                || message.contains("RaidBossEmoteFramePrivate")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PrivateAurasUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}
