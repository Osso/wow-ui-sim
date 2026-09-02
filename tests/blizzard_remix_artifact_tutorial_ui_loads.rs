use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn remix_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RemixArtifactTutorialUI")
}

fn remix_toc() -> PathBuf {
    remix_dir().join("Blizzard_RemixArtifactTutorialUI.toc")
}

const REMIX_FILES: &[&str] = &[
    "Blizzard_RemixArtifactTutorialUI_Bootstrap.lua",
    "Blizzard_RemixArtifactTutorialUI.lua",
    "Blizzard_RemixArtifactTutorialUI.xml",
];

const MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "RegisterForRemixArtifactFrameEvents",
    "OnEvent",
    "UpdateArtifactSlot",
    "UpdateTutorialState",
    "ShouldShowTutorial",
    "OnPaperDollFrameVisibilityUpdated",
    "OnRemixArtifactFrameVisibilityUpdated",
    "OnRemixArtifactFrameConfigCommitted",
    "OnTalentButtonBaseUpdated",
    "UpdateRootNodeState",
];

fn load_remix_artifact_tutorial_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &remix_toc())
        .expect("Blizzard_RemixArtifactTutorialUI should load via explicit Rust loader call");
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved =
        find_toc_file(&remix_dir()).expect("Blizzard_RemixArtifactTutorialUI TOC should resolve");
    assert_eq!(
        resolved,
        remix_toc(),
        "Blizzard_RemixArtifactTutorialUI ships exactly one TOC at the bare \
         `Blizzard_RemixArtifactTutorialUI.toc` path — no `_Mainline` flavor split, no per-flavor \
         subdirectory like Blizzard_ReforgingUI's Classic\\ tree. find_toc_file at \
         src/loader/mod.rs:65 tries `_Mainline.toc` first (miss), then bare (hit) and returns it"
    );
}

#[test]
fn toc_declares_load_on_demand_standard_game_only_and_no_deps() {
    let toc = TocFile::from_file(&remix_toc())
        .expect("Blizzard_RemixArtifactTutorialUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_RemixArtifactTutorialUI declares `## LoadOnDemand: 1` — the tutorial controller \
         is opened only when a timerunning character equips a remix artifact, never auto-loaded. \
         The OnLoad guard `if not PlayerIsTimerunning() then return end` further short-circuits \
         on non-timerunning characters even after explicit load"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_RemixArtifactTutorialUI declares `## AllowLoadGameType: standard` — the \
         `standard` token matches the cross-flavor allowlist at src/toc.rs:294-302 alongside \
         `mainline`, so is_game_type_restricted returns FALSE. Distinct from \
         Blizzard_ReforgingUI's `## AllowLoadGameType: classic` which fails the allowlist and \
         restricts to legacy clients"
    );
    assert_eq!(
        toc.metadata.get("AllowLoadGameType").map(String::as_str),
        Some("standard"),
        "AllowLoadGameType=standard preserved verbatim in metadata HashMap — confirms the parser \
         captured the exact directive value (not a normalized form), and confirms the addon \
         intentionally targets retail-only via the standard alias"
    );
    assert_eq!(
        toc.metadata.get("DefaultState").map(String::as_str),
        Some("enabled"),
        "DefaultState=enabled preserved verbatim — declares the LOD addon is on the \
         enabled-by-default whitelist so C_AddOns.LoadAddOn proceeds without an explicit user \
         opt-in. Without this directive the user would have to toggle it in the AddOn list"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_RemixArtifactTutorialUI declares zero `## Dependencies:` — every collaborator \
         (CallbackRegistrantTemplate / CallbackRegistrantMixin / EventRegistry / FrameUtil / \
         HelpTip / MicroButtonPulse / TalentFrameBaseMixin) lives in shared FrameXML / \
         Blizzard_SharedXMLBase that loads-first, so no explicit dep declaration is needed"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_RemixArtifactTutorialUI declares zero `## OptionalDeps` — RemixArtifactFrame \
         (the LOD trait panel this controller hooks into) is referenced opportunistically in \
         RegisterForRemixArtifactFrameEvents but resolved at runtime, never declared as a load \
         ordering constraint"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_RemixArtifactTutorialUI declares zero saved variables — completion state \
         persists via SetCVarBitfield(\"closedRemixArtifactTutorialFrames\", specIndex) instead \
         of per-addon SavedVariables, so progress survives across UI reloads via the CVar store"
    );
}

#[test]
fn toc_lists_bootstrap_and_two_files_in_root_directory() {
    let toc = TocFile::from_file(&remix_toc())
        .expect("Blizzard_RemixArtifactTutorialUI TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, REMIX_FILES,
        "Retail 12.1.0.69497 lists the tutorial bootstrap before its Lua and XML files"
    );
}

#[test]
fn allows_screen_returns_true_only_for_game() {
    let toc = TocFile::from_file(&remix_toc())
        .expect("Blizzard_RemixArtifactTutorialUI TOC should parse");
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_RemixArtifactTutorialUI declares `## AllowLoad: Game` — must allow the in-game \
         screen. The tutorial hooks PaperDollFrame / CharacterMicroButton which only exist in \
         the Game scene"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_RemixArtifactTutorialUI must NOT allow non-Game screens — `## AllowLoad: \
             Game` matches the Game branch only at src/toc.rs:308. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn excluded_from_eager_discovery_via_load_on_demand() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RemixArtifactTutorialUI");
    assert!(
        !found,
        "Blizzard_RemixArtifactTutorialUI must be filtered out of eager auto-discovery on the \
         Game screen via the LoadOnDemand=1 gate. discover_blizzard_addons_for_screen routes \
         LOD addons to lod_pool at src/loader/mod.rs:530 and only pulls them into the eager set \
         via pull_required_lod_addons when a non-LOD addon declares them in `## Dependencies:`. \
         No retail addon depends on Blizzard_RemixArtifactTutorialUI, so it never gets pulled. \
         AllowLoadGameType=standard (cross-flavor / not restricted) and AllowLoad=Game both pass \
         their respective filters — the LOD pool is the only gate that excludes it"
    );
}

#[test]
fn root_directory_holds_bootstrap_and_two_files() {
    let mut entries: Vec<String> = std::fs::read_dir(remix_dir())
        .expect("Blizzard_RemixArtifactTutorialUI directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_RemixArtifactTutorialUI.toc")
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_RemixArtifactTutorialUI.lua".to_string(),
            "Blizzard_RemixArtifactTutorialUI.xml".to_string(),
            "Blizzard_RemixArtifactTutorialUI_Bootstrap.lua".to_string(),
        ],
        "Retail 12.1.0.69497 ships the tutorial bootstrap beside its Lua and XML files"
    );
}

prefork_full_ui_case! {
fn loads_without_lua_errors_after_explicit_load(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_RemixArtifactTutorialUI")
                || message.contains("RemixArtifactTutorialController")
                || message.contains("RemixArtifactTutorialControllerMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_RemixArtifactTutorialUI emitted Lua errors during explicit load. The OnLoad \
         handler guards via `if not PlayerIsTimerunning() then return end` so on a non-timerunning \
         simulator player every reference inside (FrameUtil.RegisterFrameForEvents, \
         EventRegistry:RegisterCallback, INVSLOT_MAINHAND/INVSLOT_OFFHAND, the unregistered \
         REMIX_ARTIFACT_UPDATE / REMIX_ARTIFACT_ITEM_SPECS_LOADED events, the unregistered \
         C_RemixArtifactUI namespace) is skipped. Errors mean the early-exit was bypassed or \
         module-scope code at lines 1-13 (mixin assignment + two file-local tables) failed:\n  \
         {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RemixArtifactTutorialUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_RemixArtifactTutorialUI') must return true after the \
         explicit load_addon call — confirms the loader registers the LoadOnDemand standard-only \
         addon with the loaded-set even though discover_blizzard_addons_for_screen filtered it \
         out of the eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn publishes_named_top_level_frame_under_uiparent(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    let frame_kind: String = env
        .eval("return type(RemixArtifactTutorialControllerFrame)")
        .expect("RemixArtifactTutorialControllerFrame probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "RemixArtifactTutorialControllerFrame must publish at `_G` as a table — declared at \
         Blizzard_RemixArtifactTutorialUI.xml:4 with `name=\"RemixArtifactTutorialControllerFrame\"` \
         `mixin=\"RemixArtifactTutorialControllerMixin\"` `inherits=\"CallbackRegistrantTemplate\"` \
         `parent=\"UIParent\"`. The frame is the singleton tutorial controller — not a visible \
         widget, just a callback-registry-bearing event sink"
    );

    let parented_to_uiparent: bool = env
        .eval("return RemixArtifactTutorialControllerFrame:GetParent() == UIParent")
        .expect("RemixArtifactTutorialControllerFrame parent probe should succeed");
    assert!(
        parented_to_uiparent,
        "RemixArtifactTutorialControllerFrame must be parented to UIParent — `parent=\"UIParent\"` \
         XML attribute. Parenting under UIParent ensures the frame inherits the standard UI \
         coordinate space and lifecycle even though it never renders"
    );
}
}

prefork_full_ui_case! {
fn publishes_modern_mixin_table_with_eleven_methods(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    let mixin_kind: String = env
        .eval("return type(RemixArtifactTutorialControllerMixin)")
        .expect("RemixArtifactTutorialControllerMixin probe should succeed");
    assert_eq!(
        mixin_kind, "table",
        "RemixArtifactTutorialControllerMixin must publish at `_G` as a table — \
         Blizzard_RemixArtifactTutorialUI.lua:2 builds it via \
         `CreateFromMixins(CallbackRegistrantMixin)` so the table inherits AddDynamicEventMethod / \
         AddStaticEventMethod / RemoveStaticEventMethod / OnShow/OnHide auto-registration before \
         the file's own 11 method definitions extend it. Modern Mixin pattern — distinct from \
         Blizzard_ReforgingUI's pre-mixin all-free-functions shape"
    );

    for method in MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(RemixArtifactTutorialControllerMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("type(RemixArtifactTutorialControllerMixin.{method}) probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "RemixArtifactTutorialControllerMixin.{method} must be a function — every script \
             handler and helper hangs off the mixin table via `function Mixin:Name(...)` colon \
             syntax. The XML's `<OnLoad method=\"OnLoad\"/>` and `<OnEvent method=\"OnEvent\"/>` \
             attributes look up these by name from the mixin and bind them as scripts via the \
             `mixin=\"...\"` attribute path, never via global-scope resolution"
        );
    }
}
}

prefork_full_ui_case! {
fn inherited_callback_registrant_methods_carry_through(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    for inherited in [
        "AddDynamicEventMethod",
        "AddStaticEventMethod",
        "RemoveStaticEventMethod",
    ] {
        let kind: String = env
            .eval(&format!(
                "return type(RemixArtifactTutorialControllerMixin['{inherited}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("type(RemixArtifactTutorialControllerMixin.{inherited}) probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "RemixArtifactTutorialControllerMixin.{inherited} must be a function inherited from \
             CallbackRegistrantMixin via CreateFromMixins. The OnRemixArtifactFrameVisibilityUpdated \
             handler at Blizzard_RemixArtifactTutorialUI.lua:113-130 invokes \
             self:AddStaticEventMethod / self:RemoveStaticEventMethod on the trait frame — these \
             must be resolvable through the mixin chain or the show/hide hook would error"
        );
    }
}
}

prefork_full_ui_case! {
fn xml_declares_no_virtual_templates_at_global_scope(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    let no_template_leak: bool = env
        .eval(
            "return _G['RemixArtifactTutorialControllerTemplate'] == nil \
                and _G['RemixArtifactTutorialTemplate'] == nil",
        )
        .expect("Template-name probes should succeed");
    assert!(
        no_template_leak,
        "Blizzard_RemixArtifactTutorialUI/Blizzard_RemixArtifactTutorialUI.xml declares ONE \
         <Frame> with `name=\"RemixArtifactTutorialControllerFrame\"` and NO `virtual=\"true\"` — \
         it is a concrete singleton frame, not a virtual template. Probing the obvious template \
         names (RemixArtifactTutorialControllerTemplate / RemixArtifactTutorialTemplate) must \
         return nil — confirms the addon defines no reusable shape, only the one tutorial-state \
         singleton"
    );
}
}

prefork_full_ui_case! {
fn on_load_short_circuits_when_player_is_not_timerunning(env: &WowLuaEnv) {
    load_remix_artifact_tutorial_ui(env);

    let timerunning: bool = env
        .eval("return PlayerIsTimerunning()")
        .expect("PlayerIsTimerunning probe should succeed");
    assert!(
        !timerunning,
        "PlayerIsTimerunning() must default to false on a fresh simulator player — drives the \
         OnLoad early-exit at Blizzard_RemixArtifactTutorialUI.lua:17. Without this default the \
         test would need to model REMIX_ARTIFACT_UPDATE / REMIX_ARTIFACT_ITEM_SPECS_LOADED events \
         and the C_RemixArtifactUI namespace, none of which the simulator currently provides"
    );

    let no_curr_slot: bool = env
        .eval("return RemixArtifactTutorialControllerFrame.currEquippedArtifactSlotID == nil")
        .expect("currEquippedArtifactSlotID probe should succeed");
    assert!(
        no_curr_slot,
        "RemixArtifactTutorialControllerFrame.currEquippedArtifactSlotID must remain nil after \
         load — the OnLoad early-exit prevents UpdateArtifactSlot(INVSLOT_MAINHAND) and \
         UpdateArtifactSlot(INVSLOT_OFFHAND) from running, so the field never gets initialized. \
         Verifies the timerunning guard actually short-circuits before any state mutation"
    );
}
}

#[test]
fn xml_uses_method_attribute_dispatch_not_global_resolution() {
    let xml_text =
        std::fs::read_to_string(remix_dir().join("Blizzard_RemixArtifactTutorialUI.xml"))
            .expect("Blizzard_RemixArtifactTutorialUI.xml should read");
    assert!(
        xml_text.contains("<OnLoad method=\"OnLoad\"/>"),
        "Blizzard_RemixArtifactTutorialUI.xml must wire OnLoad via `<OnLoad method=\"OnLoad\"/>` \
         — the modern mixin-method dispatch attribute looks the handler up on the frame's mixin \
         table, NOT through `_G`. Distinct from Blizzard_ReforgingUI's pre-mixin XML which uses \
         `<OnLoad>ReforgingFrame_OnLoad(self)</OnLoad>` inline-call resolution"
    );
    assert!(
        xml_text.contains("<OnEvent method=\"OnEvent\"/>"),
        "Blizzard_RemixArtifactTutorialUI.xml must wire OnEvent via `<OnEvent method=\"OnEvent\"/>` \
         — same mixin-method dispatch path. The OnEvent body branches on event name to \
         UpdateArtifactSlot / UpdateTutorialState, all colon-call mixin methods"
    );
    assert!(
        xml_text.contains("inherits=\"CallbackRegistrantTemplate\""),
        "Blizzard_RemixArtifactTutorialUI.xml must inherit CallbackRegistrantTemplate from \
         Blizzard_SharedXMLBase — the inherited template's mixin=\"CallbackRegistrantMixin\" \
         attribute combined with this addon's mixin=\"RemixArtifactTutorialControllerMixin\" \
         attribute yields the full inheritance chain. CallbackRegistrantMixin contributes the \
         OnShow/OnHide auto-register lifecycle the controller relies on"
    );
}
