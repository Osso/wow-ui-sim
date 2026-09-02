use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn reforging_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ReforgingUI")
}

fn reforging_ui_toc() -> PathBuf {
    reforging_ui_dir().join("Blizzard_ReforgingUI_Classic.toc")
}

const REFORGING_FILES: &[&str] = &[
    "Classic/Blizzard_ReforgingUI_Bootstrap.lua",
    "Classic/Blizzard_ReforgingUI.lua",
    "Classic/Blizzard_ReforgingUI.xml",
    "Classic/Localization.lua",
];

const FREE_FUNCTION_GLOBALS: &[&str] = &[
    "ReforgingFrame_Show",
    "ReforgingFrame_Hide",
    "ReforgingFrame_OnLoad",
    "ReforgingFrame_OnShow",
    "ReforgingFrame_OnHide",
    "ReforgingFrame_OnEvent",
    "ReforgingFrame_OnFinishedAnim",
    "ReforgingFrame_Update",
    "HideStats",
    "ReforgingFrame_RestoreClick",
    "ReforgingFrame_ReforgeClick",
    "ReforgingFrame_AddItemClick",
    "ReforgingFrame_GetStatRow",
    "Stat_SetButtonChecked",
    "Stat_OnClick",
    "ReforgeFrame_OldStat_Initialize",
    "ReforgeFrame_NewStat_Initialize",
];

fn load_reforging_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &reforging_ui_toc())
        .expect("Blizzard_ReforgingUI should load via explicit Rust loader call");
}

#[test]
fn find_toc_file_resolves_classic_suffix_via_fallthrough() {
    let resolved =
        find_toc_file(&reforging_ui_dir()).expect("Blizzard_ReforgingUI TOC should resolve");
    assert_eq!(
        resolved,
        reforging_ui_toc(),
        "Blizzard_ReforgingUI ships ONLY a `_Classic` suffixed TOC — no Mainline or bare \
         variant. find_toc_file (src/loader/mod.rs:65) tries Mainline first, then bare, then \
         falls through to any non-{{_Cata,_Wrath,_TBC,_Vanilla,_Mists}}-suffixed TOC. `_Classic` \
         is NOT in that skip list, so the fallthrough returns Blizzard_ReforgingUI_Classic.toc"
    );
}

#[test]
fn toc_declares_load_on_demand_secure_classic_only_and_no_deps() {
    let toc =
        TocFile::from_file(&reforging_ui_toc()).expect("Blizzard_ReforgingUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_ReforgingUI declares `## LoadOnDemand: 1` — Cataclysm-era Reforging panel is \
         opened only when the player interacts with a Reforging NPC, never auto-loaded"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_ReforgingUI declares the LEGACY `## Secure: 1` directive — distinct from the \
         modern `## UseSecureEnvironment: 1` recognized by is_secure_env (src/toc.rs:236-242). \
         The legacy keyword is parsed into the metadata HashMap but is_secure_env stays false; \
         the secure-env opt-in is the newer keyword only"
    );
    assert_eq!(
        toc.metadata.get("Secure").map(String::as_str),
        Some("1"),
        "Blizzard_ReforgingUI's TOC carries the legacy `## Secure: 1` directive verbatim in the \
         metadata HashMap — the freeform parser captures every `## Key: Value` line so the \
         legacy keyword survives even though no struct accessor reads it"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_ReforgingUI declares zero `## Dependencies:` — Cataclysm Reforging is a \
         self-contained legacy panel; every collaborator (UIPanelWindows / EtherealFrameTemplate \
         / SmallMoneyFrameTemplate / MagicButtonTemplate / TruncatedButtonTemplate / \
         MoneyFrame_SetType / SOUNDKIT) is provided by FrameXML which loads-first"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_ReforgingUI declares zero `## OptionalDeps` — no conditional collaborators"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_ReforgingUI declares zero saved variables — reforge selection is server-side \
         per-item state, no client persistence"
    );
}

#[test]
fn toc_lists_bootstrap_and_three_files_in_classic_subdirectory() {
    let toc =
        TocFile::from_file(&reforging_ui_toc()).expect("Blizzard_ReforgingUI TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, REFORGING_FILES,
        "Retail 12.1.0.69497 lists the bootstrap before the classic Lua, XML, and localization files"
    );
}

#[test]
fn is_game_type_restricted_returns_true_for_classic_only_addon() {
    let toc =
        TocFile::from_file(&reforging_ui_toc()).expect("Blizzard_ReforgingUI TOC should parse");
    assert!(
        toc.is_game_type_restricted(),
        "Blizzard_ReforgingUI declares `## AllowLoadGameType: classic` — does not match \
         `mainline` or `standard`, so is_game_type_restricted (src/toc.rs:294) returns true. The \
         Reforging system was removed in Mists of Pandaria pre-patch (5.0.4) so the addon ships \
         only as a Classic-flavor panel"
    );
}

#[test]
fn excluded_from_every_screen_auto_discovery_under_retail() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_ReforgingUI");
        assert!(
            !found,
            "Blizzard_ReforgingUI must be filtered out of auto-discovery on retail across every \
             ScreenKind. Three independent gates each suffice: `## AllowLoadGameType: classic` \
             (game-type-restricted), `## LoadOnDemand: 1` (lod_pool only, never eager), and the \
             default Game-only AllowLoad which excludes glue screens. The screen sweep at \
             src/loader/mod.rs:527 short-circuits via is_game_type_restricted first. (Screen \
             tested: {screen:?})"
        );
    }
}

#[test]
fn classic_subdirectory_holds_bootstrap_lua_xml_and_localization() {
    let classic_dir = reforging_ui_dir().join("Classic");
    let mut entries: Vec<String> = std::fs::read_dir(&classic_dir)
        .expect("Blizzard_ReforgingUI/Classic directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_ReforgingUI.lua".to_string(),
            "Blizzard_ReforgingUI.xml".to_string(),
            "Blizzard_ReforgingUI_Bootstrap.lua".to_string(),
            "Localization.lua".to_string(),
        ],
        "Retail 12.1.0.69497 ships the classic bootstrap beside the Lua, XML, and localization files"
    );
}

prefork_full_ui_case! {
fn loads_with_only_expected_classic_event_warnings(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ReforgingUI")
                || message.contains("ReforgingFrame")
                || message.contains("ReforgingStatTemplate")
                || message.contains("Reforge")
        })
        .cloned()
        .collect();
    let unexpected_errors: Vec<&String> = load_errors
        .iter()
        .filter(|message| {
            !message.contains("FORGE_MASTER_SET_ITEM")
                && !message.contains("FORGE_MASTER_ITEM_CHANGED")
        })
        .collect();
    assert!(
        unexpected_errors.is_empty(),
        "Blizzard_ReforgingUI emitted UNEXPECTED Lua errors during explicit load. The two \
         classic-only events FORGE_MASTER_SET_ITEM / FORGE_MASTER_ITEM_CHANGED registered by \
         ReforgingFrame_OnLoad are deliberately not modelled by the simulator (Reforging was \
         removed from retail in MoP 5.0.4) so warnings about them are expected and filtered \
         out. Anything else is a real load failure:\n  {}",
        unexpected_errors
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ReforgingUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ReforgingUI') must return true after the explicit \
         load_addon call — confirms the loader registers the LoadOnDemand classic-only addon \
         with the loaded-set even though discover_blizzard_addons_for_screen filtered it out"
    );
}
}

prefork_full_ui_case! {
fn publishes_named_top_level_frame_under_uiparent(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let frame_kind: String = env
        .eval("return type(ReforgingFrame)")
        .expect("ReforgingFrame probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "ReforgingFrame must publish at `_G` as a table — declared at \
         Classic/Blizzard_ReforgingUI.xml:163 with `name=\"ReforgingFrame\"` \
         `inherits=\"EtherealFrameTemplate\"` `parent=\"UIParent\"` `toplevel=\"true\"` \
         `movable=\"true\"` `enableMouse=\"true\"` `hidden=\"true\"`"
    );

    let parented_to_uiparent: bool = env
        .eval("return ReforgingFrame:GetParent() == UIParent")
        .expect("ReforgingFrame parent probe should succeed");
    assert!(
        parented_to_uiparent,
        "ReforgingFrame must be parented to UIParent — `parent=\"UIParent\"` XML attribute"
    );
}
}

prefork_full_ui_case! {
fn publishes_pre_mixin_global_handler_functions(env: &WowLuaEnv) {
    load_reforging_ui(env);

    for name in FREE_FUNCTION_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{name}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish at `_G` as a function — Blizzard_ReforgingUI follows the \
             Cataclysm-era PRE-MIXIN pattern where every script handler and helper is a free \
             `function GlobalName(...)` definition (NOT a `Mixin = {{}}` table-method binding). \
             The XML wires script handlers via inline attribute calls like \
             `<OnLoad>ReforgingFrame_OnLoad(self)</OnLoad>` that resolve through the global \
             scope, not through SetScript with mixin self-reference"
        );
    }
}
}

prefork_full_ui_case! {
fn publishes_reforge_max_stats_shown_constant(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let value: f64 = env
        .eval("return REFORGE_MAX_STATS_SHOWN")
        .expect("REFORGE_MAX_STATS_SHOWN probe should succeed");
    assert_eq!(
        value, 8.0,
        "REFORGE_MAX_STATS_SHOWN must equal 8 — Classic/Blizzard_ReforgingUI.lua:2 publishes \
         the constant at file scope. ReforgingFrame_GetStatRow uses it as the upper bound when \
         lazily creating ReforgingFrameLeftStat<N>/ReforgingFrameRightStat<N> CHECKBUTTON \
         clones via CreateFrame(\"CHECKBUTTON\", ..., \"ReforgingStatTemplate\")"
    );
}
}

prefork_full_ui_case! {
fn registers_panel_window_metadata_for_reforging_frame(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let area: String = env
        .eval("return UIPanelWindows.ReforgingFrame.area")
        .expect("UIPanelWindows.ReforgingFrame.area probe should succeed");
    assert_eq!(
        area, "left",
        "UIPanelWindows.ReforgingFrame.area must equal \"left\" — \
         Classic/Blizzard_ReforgingUI.lua:4 mutates the shared UIPanelWindows registry with \
         `{{ area = \"left\", pushable = 0 }}` so ShowUIPanel docks the frame to the left panel \
         slot, contesting the same slot as the bag/quest/inspect frames"
    );

    let pushable: f64 = env
        .eval("return UIPanelWindows.ReforgingFrame.pushable")
        .expect("UIPanelWindows.ReforgingFrame.pushable probe should succeed");
    assert_eq!(
        pushable, 0.0,
        "UIPanelWindows.ReforgingFrame.pushable must equal 0 — pushable=0 means UIParent will \
         NOT shove this panel out of the way to make room for higher-priority center panels; \
         opening a higher-priority window simply hides ReforgingFrame outright"
    );
}
}

prefork_full_ui_case! {
fn virtual_check_button_template_stays_off_global_scope(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let kind: String = env
        .eval("return type(_G['ReforgingStatTemplate'])")
        .expect("ReforgingStatTemplate probe should succeed");
    assert_eq!(
        kind, "nil",
        "ReforgingStatTemplate must NOT publish at `_G` — declared as `virtual=\"true\"` at \
         Classic/Blizzard_ReforgingUI.xml:60 so the loader keeps it in the template registry \
         only. Consumed by `<CheckButton inherits=\"ReforgingStatTemplate\">` for the static \
         $parentLeftStat1/$parentRightStat1 children plus by ReforgingFrame_GetStatRow's \
         CreateFrame(..., \"ReforgingStatTemplate\") clone calls; never resolved through `_G`"
    );
}
}

prefork_full_ui_case! {
fn defines_no_mixin_tables_pre_mixin_pattern(env: &WowLuaEnv) {
    load_reforging_ui(env);

    let no_mixin_leak: bool = env
        .eval(
            "return _G['ReforgingFrameMixin'] == nil \
                and _G['ReforgingMixin'] == nil \
                and _G['ReforgingStatMixin'] == nil",
        )
        .expect("Mixin-name probes should succeed");
    assert!(
        no_mixin_leak,
        "Blizzard_ReforgingUI predates the Mixin pattern (introduced ~Legion). The 17 handler / \
         helper functions are free globals, not bound to any Mixin table. Probing the obvious \
         Mixin names (ReforgingFrameMixin / ReforgingMixin / ReforgingStatMixin) must return \
         nil — confirms the addon retains its Cataclysm shape rather than being silently \
         migrated under the loader"
    );
}
}

#[test]
fn xml_loads_via_script_directive_and_inline_lua_listed_in_toc() {
    let xml_text =
        std::fs::read_to_string(reforging_ui_dir().join("Classic/Blizzard_ReforgingUI.xml"))
            .expect("Classic/Blizzard_ReforgingUI.xml should read");
    assert!(
        xml_text.contains("<Script file=\"Blizzard_ReforgingUI.lua\"/>"),
        "Classic/Blizzard_ReforgingUI.xml must declare `<Script file=\"Blizzard_ReforgingUI.lua\"/>` \
         at the document head — Blizzard_ReforgingUI is one of the rare addons that lists the \
         .lua file BOTH in the TOC body AND via XML's <Script file=> directive, ensuring the \
         globals are loaded before the XML's inline `<OnLoad>ReforgingFrame_OnLoad(self)</OnLoad>` \
         attributes resolve them through `_G`"
    );

    let toc_text =
        std::fs::read_to_string(reforging_ui_toc()).expect("Blizzard_ReforgingUI TOC should read");
    assert!(
        toc_text.contains("Classic\\Blizzard_ReforgingUI.lua"),
        "TOC body must list `Classic\\Blizzard_ReforgingUI.lua` BEFORE the .xml so the loader \
         compiles the lua first; the XML's <Script file=> directive then becomes a redundant \
         re-load of the already-cached chunk (idempotent — re-defining the same globals is a \
         no-op)"
    );
}
