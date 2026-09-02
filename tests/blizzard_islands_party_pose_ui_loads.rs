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

fn islands_party_pose_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_IslandsPartyPoseUI")
}

fn islands_party_pose_toc() -> PathBuf {
    islands_party_pose_dir().join("Blizzard_IslandsPartyPoseUI.toc")
}

fn party_pose_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PartyPoseUI/Blizzard_PartyPoseUI.toc")
}

fn ui_widgets_toc() -> PathBuf {
    find_toc_file(&blizzard_ui_dir().join("Blizzard_UIWidgets"))
        .expect("Blizzard_UIWidgets TOC should resolve")
}

fn lfg_util_toc() -> PathBuf {
    find_toc_file(&blizzard_ui_dir().join("Blizzard_LFGUtil"))
        .expect("Blizzard_LFGUtil TOC should resolve")
}

const MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "SetRewards",
    "SetLeaveButtonText",
    "GetPartyPoseData",
    "Dismiss",
];

const PARENT_KEY_CHILDREN: &[&str] = &["OverlayElements", "ModelScene", "Score", "LeaveButton"];

fn load_islands_party_pose_ui_with_dependencies(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &party_pose_toc())
        .expect("Blizzard_PartyPoseUI should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &ui_widgets_toc())
        .expect("Blizzard_UIWidgets should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &lfg_util_toc())
        .expect("Blizzard_LFGUtil should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &islands_party_pose_toc())
        .expect("Blizzard_IslandsPartyPoseUI should load via explicit Rust loader call");
}

#[test]
fn blizzard_islands_party_pose_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&islands_party_pose_dir())
        .expect("Blizzard_IslandsPartyPoseUI TOC should resolve");
    assert_eq!(
        resolved,
        islands_party_pose_toc(),
        "Blizzard_IslandsPartyPoseUI ships exactly one bare TOC — the LoadOnDemand island-end \
         scoreboard module resolves via `find_toc_file` fallthrough"
    );
}

#[test]
fn blizzard_islands_party_pose_toc_declares_lod_with_three_required_deps() {
    let toc = TocFile::from_file(&islands_party_pose_toc())
        .expect("Blizzard_IslandsPartyPoseUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_IslandsPartyPoseUI declares `## LoadOnDemand: 1` — pulled in by the islands \
         end-of-run scoreboard flow when the LFG_COMPLETION_REWARD event fires; not loaded \
         eagerly because most play sessions never run an island expedition"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_PartyPoseUI".to_string(),
            "Blizzard_UIWidgets".to_string(),
            "Blizzard_LFGUtil".to_string(),
        ],
        "Current retail TOC declares PartyPoseUI, UIWidgets, and LFGUtil as RequiredDeps"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_IslandsPartyPoseUI declares no `## OptionalDeps` — its three RequiredDeps \
         cover the declared load floor"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_IslandsPartyPoseUI declares zero saved variables — every island scoreboard \
         re-fetches reward data from the server via GetLFGCompletionReward / \
         GetLFGCompletionRewardItem on each LFG_COMPLETION_REWARD event; no persistence"
    );
}

#[test]
fn blizzard_islands_party_pose_toc_declares_standard_game_type_only() {
    let toc = TocFile::from_file(&islands_party_pose_toc())
        .expect("Blizzard_IslandsPartyPoseUI TOC should parse");

    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_IslandsPartyPoseUI declares `## AllowLoadGameType: standard` — \
         `is_game_type_restricted()` (src/toc.rs:294) treats both `standard` and `mainline` as \
         the unrestricted retail flavor, so the addon is available on retail Midnight"
    );

    let raw = std::fs::read_to_string(islands_party_pose_toc())
        .expect("Blizzard_IslandsPartyPoseUI TOC should read");
    assert!(
        raw.contains("## AllowLoadGameType: standard"),
        "TOC must declare `## AllowLoadGameType: standard` — pins the addon to the retail \
         island-expedition codepath; classic flavors don't ship the LFG completion-reward \
         bridge this scoreboard consumes"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must omit `## AllowLoad:` — LoD addons rely entirely on the explicit LoadAddOn \
         from the islands flow, never via the screen auto-discovery sweep"
    );
    assert!(
        !raw.contains("## DefaultState:"),
        "TOC must omit `## DefaultState:` — LoD addons load only when the islands flow \
         requests them; explicit DefaultState would conflict with the on-demand contract"
    );
}

#[test]
fn blizzard_islands_party_pose_toc_lists_bootstrap_lua_then_xml_in_order() {
    let toc = TocFile::from_file(&islands_party_pose_toc())
        .expect("Blizzard_IslandsPartyPoseUI TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "Blizzard_IslandsPartyPoseUI_Bootstrap.lua".to_string(),
            "Blizzard_IslandsPartyPoseUI.lua".to_string(),
            "Blizzard_IslandsPartyPoseUI.xml".to_string(),
        ],
        "Current retail TOC lists the bootstrap before the lua and XML files"
    );
}

#[test]
fn blizzard_islands_party_pose_directory_holds_four_entries() {
    let entries = std::fs::read_dir(islands_party_pose_dir())
        .expect("Blizzard_IslandsPartyPoseUI directory should read")
        .count();
    assert_eq!(
        entries, 4,
        "Directory must hold exactly 4 entries: the TOC, bootstrap, lua, and XML files"
    );
}

#[test]
fn blizzard_islands_party_pose_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_IslandsPartyPoseUI");
        assert!(
            !found,
            "Blizzard_IslandsPartyPoseUI must NOT appear in any ScreenKind auto-discovery sweep \
             — `## LoadOnDemand: 1` excludes it from every eager pass; only `load_addon` \
             called explicitly by the islands LFG_COMPLETION_REWARD flow pulls it in. \
             (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_IslandsPartyPoseUI")
                || message.contains("IslandsPartyPoseMixin")
                || message.contains("IslandsPartyPoseFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_IslandsPartyPoseUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_is_addon_loaded_after_explicit_lod_with_deps(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_IslandsPartyPoseUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_IslandsPartyPoseUI') must return true after the \
         explicit load_addon call"
    );

    let party_pose_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PartyPoseUI')")
        .expect("Blizzard_PartyPoseUI IsAddOnLoaded probe should succeed");
    assert!(
        party_pose_loaded,
        "Blizzard_PartyPoseUI dep must register as loaded after the explicit dep load — \
         provides the PartyPoseMixin parent and PartyPoseFrameTemplate the islands frame \
         consumes"
    );
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_mixin_publishes_with_partypose_inherited_methods(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let kind: String = env
        .eval("return type(IslandsPartyPoseMixin)")
        .expect("IslandsPartyPoseMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "IslandsPartyPoseMixin must publish at `_G` as a table — declared at \
         Blizzard_IslandsPartyPoseUI.lua:1 via `IslandsPartyPoseMixin = \
         CreateFromMixins(PartyPoseMixin)` so the mixin table inherits every PartyPoseMixin \
         method via copy-on-init"
    );

    for method in MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(IslandsPartyPoseMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("IslandsPartyPoseMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "IslandsPartyPoseMixin.{method} must publish as a function — defined directly on \
             the mixin (Blizzard_IslandsPartyPoseUI.lua) so it overrides the PartyPoseMixin \
             parent's same-named method when the islands scoreboard flow dispatches"
        );
    }

    let inherited_kind: String = env
        .eval("return type(IslandsPartyPoseMixin.AddReward)")
        .expect("IslandsPartyPoseMixin.AddReward probe should succeed");
    assert_eq!(
        inherited_kind, "function",
        "IslandsPartyPoseMixin.AddReward must publish as a function — proves \
         CreateFromMixins(PartyPoseMixin) copy-on-init pulled in PartyPoseMixin.AddReward (the \
         method SetRewards's continuableContainer:ContinueOnLoad callback invokes via \
         `self:AddReward(...)` per Blizzard_IslandsPartyPoseUI.lua:32)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_named_frame_publishes_with_inherits_and_mixin_chain(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let kind: String = env
        .eval("return type(IslandsPartyPoseFrame)")
        .expect("IslandsPartyPoseFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "IslandsPartyPoseFrame must publish at `_G` as a table — declared at \
         Blizzard_IslandsPartyPoseUI.xml:3 with `name=\"IslandsPartyPoseFrame\"` \
         `parent=\"UIParent\"` `inherits=\"PartyPoseFrameTemplate\"` \
         `mixin=\"IslandsPartyPoseMixin\"` `toplevel=\"true\"`"
    );

    let name: String = env
        .eval("return IslandsPartyPoseFrame:GetName()")
        .expect("IslandsPartyPoseFrame:GetName() probe should succeed");
    assert_eq!(name, "IslandsPartyPoseFrame");
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_named_frame_carries_four_parent_key_children(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    for parent_key in PARENT_KEY_CHILDREN {
        let kind: String = env
            .eval(&format!(
                "return type(IslandsPartyPoseFrame['{parent_key}'])"
            ))
            .unwrap_or_else(|err| panic!("IslandsPartyPoseFrame.{parent_key} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "IslandsPartyPoseFrame.{parent_key} must publish as a parentKey child — declared \
             at Blizzard_IslandsPartyPoseUI.xml:7-44. The 4 children are: OverlayElements \
             (frameLevel=510 setAllPoints with the parentKey Topper Texture in ARTWORK layer \
             anchored BOTTOM relativePoint=TOP for the faction-specific scoreboard topper); \
             ModelScene (inherits PartyPoseModelFrameTemplate enableMouse=false \
             useParentLevel=true centered — the 3D character pose backdrop); Score (inherits \
             UIWidgetContainerTemplate sized 100x40 with showAndHideOnWidgetSetRegistration=\
             false KeyValue, anchored BOTTOMLEFT to the ModelScene at 15,-55 — holds the \
             server-driven score widget); LeaveButton (inherits \
             UIPanelButtonNoTooltipResizeToFitTemplate with minimumWidth=164 KeyValue, \
             anchored BOTTOMRIGHT to the ModelScene at -20,-45 — fires Dismiss → \
             ConfirmOrLeaveLFGParty)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_overlay_elements_carries_topper_texture(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let kind: String = env
        .eval("return type(IslandsPartyPoseFrame.OverlayElements.Topper)")
        .expect("OverlayElements.Topper probe should succeed");
    assert_eq!(
        kind, "table",
        "IslandsPartyPoseFrame.OverlayElements.Topper must publish as a parentKey Texture — \
         the GetPartyPoseData override sets partyPoseData.themeData.Topper to either \
         `scoreboard-horde-header` or `scoreboard-alliance-header` (per UnitFactionGroup) so \
         the Texture's atlas is faction-driven at runtime, not baked into the XML"
    );
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_modelscene_publishes_as_modelscene_subtype(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let object_type: String = env
        .eval("return IslandsPartyPoseFrame.ModelScene:GetObjectType()")
        .expect("ModelScene:GetObjectType() probe should succeed");
    assert_eq!(
        object_type, "ModelScene",
        "IslandsPartyPoseFrame.ModelScene:GetObjectType() must return `ModelScene` — declared \
         as `<ModelScene parentKey=\"ModelScene\" inherits=\"PartyPoseModelFrameTemplate\">` so \
         the loader instantiates the 3D-pose-backdrop widget type, not a plain Frame"
    );
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_score_widget_container_keeps_register_visibility_off(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let kv: bool = env
        .eval("return IslandsPartyPoseFrame.Score.showAndHideOnWidgetSetRegistration")
        .expect("Score.showAndHideOnWidgetSetRegistration probe should succeed");
    assert!(
        !kv,
        "IslandsPartyPoseFrame.Score.showAndHideOnWidgetSetRegistration must equal false — \
         declared via `<KeyValue key=\"showAndHideOnWidgetSetRegistration\" value=\"false\" \
         type=\"boolean\"/>` (Blizzard_IslandsPartyPoseUI.xml:30) so the Score container's \
         visibility is driven by the islands scoreboard flow's explicit Show / Hide calls, NOT \
         by the UIWidget API auto-toggling whenever a widget set registers / unregisters"
    );
}
}

prefork_full_ui_case! {
fn blizzard_islands_party_pose_leave_button_keeps_minimum_width_keyvalue(env: &WowLuaEnv) {
    load_islands_party_pose_ui_with_dependencies(env);

    let min_width: f64 = env
        .eval("return IslandsPartyPoseFrame.LeaveButton.minimumWidth")
        .expect("LeaveButton.minimumWidth probe should succeed");
    assert_eq!(
        min_width, 164.0,
        "IslandsPartyPoseFrame.LeaveButton.minimumWidth must equal 164 — declared via \
         `<KeyValue key=\"minimumWidth\" value=\"164\" type=\"number\"/>` \
         (Blizzard_IslandsPartyPoseUI.xml:39). UIPanelButtonNoTooltipResizeToFitTemplate uses \
         this floor when ResizeToFit shrinks the button to fit the ISLAND_LEAVE locale string \
         so the button never collapses below 164px regardless of locale"
    );
}
}
