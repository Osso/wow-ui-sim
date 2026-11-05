#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn global_fx_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GlobalFXModelScenes")
}

fn global_fx_toc() -> PathBuf {
    global_fx_dir().join("Blizzard_GlobalFXModelScenes.toc")
}

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
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_global_fx_model_scenes_resolves_bare_toc() {
    let resolved = find_toc_file(&global_fx_dir())
        .expect("Blizzard_GlobalFXModelScenes directory must contain a discoverable TOC");
    let resolved_name = resolved
        .file_name()
        .expect("resolved TOC must have a filename")
        .to_str()
        .expect("resolved TOC filename must be utf-8");

    assert_eq!(
        resolved_name, "Blizzard_GlobalFXModelScenes.toc",
        "Blizzard_GlobalFXModelScenes ships only the bare \
         `Blizzard_GlobalFXModelScenes.toc` (no `_Mainline.toc` variant); \
         src/loader/mod.rs:65's `find_toc_file` falls through the `_Mainline.toc` lookup \
         and resolves the bare suffix"
    );
}

#[test]
fn blizzard_global_fx_model_scenes_toc_is_minimal_eager_game_only_no_deps() {
    let toc = TocFile::from_file(&global_fx_toc()).expect("Blizzard_GlobalFXModelScenes TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_GlobalFXModelScenes does NOT declare `## LoadOnDemand` — it loads \
         eagerly with the Game-screen auto-discovery pass so consumers like \
         Blizzard_CovenantSanctum / Blizzard_EncounterJournal / Blizzard_ItemButton / \
         Blizzard_PlayerChoice / Blizzard_ProfessionsTemplates / Blizzard_OrderHallUI / \
         Blizzard_TorghastLevelPicker can call `GlobalFXDialogModelScene:AddEffect(...)` \
         immediately on Show without first issuing a LoadAddOn"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GlobalFXModelScenes does not declare `## UseSecureEnvironment` — visual \
         effect playback is purely cosmetic, no protected-action surface"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GlobalFXModelScenes declares no `## SavedVariables` — the addon owns \
         only three persistent ModelScene host frames, not user-visible state"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GlobalFXModelScenes declares no `## AllowLoadGameType` — the FX scenes \
         are reused across mainline + classic flavors, so the absence of the directive \
         keeps `is_game_type_restricted()` returning false at src/toc.rs:299"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_GlobalFXModelScenes declares NO `## Dependencies` / `## RequiredDep` — \
         the addon's only Lua surface is the inherited \
         `ScriptAnimatedModelSceneTemplate` (provided by Blizzard_SharedXML's globally- \
         registered template surface), so it doesn't need the TOC to enumerate parents. \
         Got: {:?}",
        toc.dependencies()
    );
}

#[test]
fn blizzard_global_fx_model_scenes_toc_declares_allow_load_game_default_enabled() {
    let toc_text = std::fs::read_to_string(global_fx_toc())
        .expect("Blizzard_GlobalFXModelScenes TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_GlobalFXModelScenes must declare `## AllowLoad: Game` — the FX scene \
         hosts are meaningful only on the in-world Game screen (the Login glue uses its \
         own GlueModelScenes), so `allows_screen` returns true only for ScreenKind::Game"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_GlobalFXModelScenes must declare `## DefaultState: enabled` — the addon \
         is bundled with the client and active by default; without LoadOnDemand, \
         DefaultState=enabled is the directive that keeps it in the auto-discovery set"
    );
}

#[test]
fn blizzard_global_fx_model_scenes_toc_lists_one_xml_zero_lua() {
    let toc_text = std::fs::read_to_string(global_fx_toc())
        .expect("Blizzard_GlobalFXModelScenes TOC should read");
    let lua_count = toc_text.matches(".lua").count();
    let xml_count = toc_text.matches(".xml").count();
    assert_eq!(
        lua_count, 0,
        "Blizzard_GlobalFXModelScenes TOC enumerates ZERO .lua files — the addon is \
         pure XML, every behavior comes from the inherited \
         `ScriptAnimatedModelSceneTemplate`. Got: {lua_count}"
    );
    assert_eq!(
        xml_count, 1,
        "Blizzard_GlobalFXModelScenes TOC enumerates exactly 1 .xml file \
         (GlobalFXModelScenes.xml — owns the 3 toplevel ModelScene instances). \
         Got: {xml_count}"
    );
    assert!(
        toc_text.contains("GlobalFXModelScenes.xml"),
        "TOC must list GlobalFXModelScenes.xml — the only file in the addon"
    );
}

#[test]
fn blizzard_global_fx_model_scenes_appears_in_game_auto_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GlobalFXModelScenes");
    assert!(
        in_game,
        "Blizzard_GlobalFXModelScenes (non-LOD with `## AllowLoad: Game` + \
         `## DefaultState: enabled`) must appear in Game-screen auto-discovery so the 3 \
         FX ModelScenes are present on UIParent before any consumer attempts \
         `GlobalFXDialogModelScene:AddEffect(...)` on Show"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GlobalFXModelScenes");
    assert!(
        !in_login,
        "Blizzard_GlobalFXModelScenes declares `## AllowLoad: Game` — it must be absent \
         from Login auto-discovery; the Login glue's own GlueModelScenes addon owns the \
         glue-screen FX layer"
    );
}

fn is_three_d_model_gap(message: &str) -> bool {
    message.contains("ScriptAnimatedModelSceneTemplate")
        || message.contains("ModelScene")
        || message.contains("SetTargetDistance")
        || message.contains("SetFacingLeft")
        || message.contains("SetFacingRight")
        || message.contains("SetCamDistanceScale")
}

prefork_full_ui_case! {
fn blizzard_global_fx_model_scenes_loads_without_non_three_d_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let fx_errors: Vec<String> = load_errors
        .iter()
        .filter(|message| message.contains("GlobalFX"))
        .filter(|message| !is_three_d_model_gap(message))
        .cloned()
        .collect();
    assert!(
        fx_errors.is_empty(),
        "Blizzard_GlobalFXModelScenes emitted non-3D-model Lua errors during load \
         (3D-rendering surface is an intentional permanent gap per AGENTS.md — \
         Model/ModelScene/PlayerModel/DressUpModel methods are stub-only):\n  {}",
        fx_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_global_fx_model_scenes_is_addon_loaded_after_full_game_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval(
            "return C_AddOns and C_AddOns.IsAddOnLoaded('Blizzard_GlobalFXModelScenes') \
             or false",
        )
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_GlobalFXModelScenes') must return true after \
         the standard Game-screen auto-discovery pass (the addon is non-LOD + \
         AllowLoad=Game + DefaultState=enabled, so it's pulled in eagerly)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_global_fx_model_scenes_publishes_three_named_modelscenes(env: &WowLuaEnv) {

    let scenes: (bool, bool, bool) = env
        .eval(
            "return GlobalFXDialogModelScene ~= nil, \
                    GlobalFXMediumModelScene ~= nil, \
                    GlobalFXBackgroundModelScene ~= nil",
        )
        .expect("ModelScene global probe should succeed");
    assert_eq!(
        scenes,
        (true, true, true),
        "GlobalFXModelScenes.xml declares exactly 3 toplevel ModelScene instances as \
         globals: GlobalFXDialogModelScene (line 3 — the canonical DIALOG-strata FX host \
         consumed by ItemButtonTemplate enchant burst, ProfessionsRecipeCrafterDetails \
         flare, MonthlyActivities reward burst, OrderHallTalents purchase, \
         CovenantSanctumUpgrades anima gain, PlayerChoicePowerChoice select), \
         GlobalFXMediumModelScene (line 4 — MEDIUM-strata host consumed by \
         PlayerChoiceToggleButton), GlobalFXBackgroundModelScene (line 5 — LOW-strata + \
         BACKGROUND draw-layer host consumed by PlayerChoiceTorghastOption smoke + \
         TorghastLevelPicker background smoke)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_global_fx_model_scenes_publish_strata_and_uiparent_parent(env: &WowLuaEnv) {

    let strata: (String, String, String) = env
        .eval(
            "return GlobalFXDialogModelScene:GetFrameStrata(), \
                    GlobalFXMediumModelScene:GetFrameStrata(), \
                    GlobalFXBackgroundModelScene:GetFrameStrata()",
        )
        .expect("Strata probe should succeed");
    assert_eq!(
        strata.0, "DIALOG",
        "GlobalFXDialogModelScene must ride at DIALOG strata — XML line 3 declares \
         `frameStrata=DIALOG` so dialog-tier FX render above the standard MEDIUM-strata \
         frames but below TOOLTIP/FULLSCREEN_DIALOG"
    );
    assert_eq!(
        strata.1, "MEDIUM",
        "GlobalFXMediumModelScene must ride at MEDIUM strata — XML line 4 declares \
         `frameStrata=MEDIUM` so toggle-button FX co-exist with standard mid-tier UI"
    );
    assert_eq!(
        strata.2, "LOW",
        "GlobalFXBackgroundModelScene must ride at LOW strata — XML line 5 declares \
         `frameStrata=LOW` + `drawLayer=BACKGROUND` so background smoke FX render below \
         the standard MEDIUM-strata UI"
    );

    let parents: (String, String, String) = env
        .eval(
            "return GlobalFXDialogModelScene:GetParent():GetName(), \
                    GlobalFXMediumModelScene:GetParent():GetName(), \
                    GlobalFXBackgroundModelScene:GetParent():GetName()",
        )
        .expect("Parent probe should succeed");
    assert_eq!(
        parents,
        (
            "UIParent".to_string(),
            "UIParent".to_string(),
            "UIParent".to_string(),
        ),
        "All 3 ModelScenes must be parented to UIParent — XML declares \
         `parent=UIParent setAllPoints=true` so each scene fills the entire UIParent \
         viewport for global cross-frame effect playback"
    );
}
}

prefork_full_ui_case! {
fn blizzard_global_fx_model_scenes_disable_mouse_input(env: &WowLuaEnv) {

    let mouse: (bool, bool, bool) = env
        .eval(
            "return GlobalFXDialogModelScene:IsMouseEnabled(), \
                    GlobalFXMediumModelScene:IsMouseEnabled(), \
                    GlobalFXBackgroundModelScene:IsMouseEnabled()",
        )
        .expect("Mouse-enabled probe should succeed");
    assert_eq!(
        mouse,
        (false, false, false),
        "All 3 ModelScenes must declare `enableMouse=false` — XML lines 3-5 explicitly \
         disable mouse input so the fullscreen FX host frames don't intercept clicks \
         destined for the underlying UI elements they overlay"
    );
}
}
