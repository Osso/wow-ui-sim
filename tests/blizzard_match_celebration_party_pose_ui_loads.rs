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

fn match_celebration_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MatchCelebrationPartyPoseUI")
}

fn match_celebration_toc() -> PathBuf {
    match_celebration_dir().join("Blizzard_MatchCelebrationPartyPoseUI.toc")
}

fn party_pose_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PartyPoseUI/Blizzard_PartyPoseUI.toc")
}

const MATCH_CELEBRATION_TOC_FILES: &[&str] = &[
    "Blizzard_MatchCelebrationPartyPoseUI_Bootstrap.lua",
    "Blizzard_MatchCelebrationPartyPoseUI.lua",
    "Blizzard_MatchCelebrationPartyPoseUI.xml",
];

const MIXIN_OVERRIDE_METHODS: &[&str] = &[
    "OnLoad",
    "Dismiss",
    "LoadPartyPose",
    "SetLeaveButtonText",
    "GetPartyPoseDataFromPartyPoseID",
];

const PARENT_KEY_CHILDREN: &[&str] = &["OverlayElements", "ModelScene", "Score", "ButtonContainer"];

fn load_match_celebration_party_pose_ui_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &party_pose_toc())
        .expect("Blizzard_PartyPoseUI should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &match_celebration_toc())
        .expect("Blizzard_MatchCelebrationPartyPoseUI should load via explicit Rust loader call");
}

#[test]
fn blizzard_match_celebration_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&match_celebration_dir())
        .expect("Blizzard_MatchCelebrationPartyPoseUI TOC should resolve");
    assert_eq!(
        resolved,
        match_celebration_toc(),
        "Blizzard_MatchCelebrationPartyPoseUI ships exactly one bare TOC. The match-celebration \
         scoreboard module is a cross-flavor end-of-LFG-instance screen with no flavor-suffixed \
         variants — the bare TOC resolves via `find_toc_file` after the `_Mainline.toc` lookup \
         misses"
    );
}

#[test]
fn blizzard_match_celebration_toc_declares_lod_with_current_dependencies() {
    let toc = TocFile::from_file(&match_celebration_toc())
        .expect("Blizzard_MatchCelebrationPartyPoseUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_MatchCelebrationPartyPoseUI declares `## LoadOnDemand: 1` — pulled in only by \
         `MatchCelebrationPartyPose_LoadUI()` (Blizzard_UIParent/Mainline/UIParent.lua:391-393), \
         which itself fires from the LFG_COMPLETION_REWARD path when an arena / battleground / \
         dungeon match completes; not loaded eagerly because most play sessions never hit a \
         scoreboard-eligible match"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_PartyPoseUI".to_string(),
            "Blizzard_LFGUtil".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Retail TOC declares RequiredDep Blizzard_PartyPoseUI and Blizzard_LFGUtil, followed by \
         Dependencies Blizzard_GameMenuEsc"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_MatchCelebrationPartyPoseUI declares no `## OptionalDeps` — the single \
         RequiredDep covers every external template / mixin the addon consumes; no optional \
         fallback path"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_MatchCelebrationPartyPoseUI declares zero saved variables — the celebration \
         screen pulls `partyPoseInfo.extraButtonText`, `partyPoseInfo.flags`, and \
         `partyPoseInfo.uiTextureKit` fresh from the server-driven partyPoseData each time \
         LoadPartyPose runs; no per-character persistence"
    );
}

#[test]
fn blizzard_match_celebration_toc_omits_game_type_and_allow_load_keys() {
    let toc = TocFile::from_file(&match_celebration_toc())
        .expect("Blizzard_MatchCelebrationPartyPoseUI TOC should parse");

    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_MatchCelebrationPartyPoseUI omits `## AllowLoadGameType:` — the \
         is_game_type_restricted accessor (src/toc.rs:294) returns false when the key is \
         missing, so the addon is universal across every flavor that ships LFG match \
         celebrations"
    );

    let raw = std::fs::read_to_string(match_celebration_toc())
        .expect("Blizzard_MatchCelebrationPartyPoseUI TOC should read");
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must omit `## AllowLoad:` — LoD addons rely entirely on the explicit \
         UIParentLoadAddOn from MatchCelebrationPartyPose_LoadUI; the missing key routes through \
         the default branch at src/toc.rs:311 to Game-only when the auto-discovery sweep \
         evaluates allows_screen, but allows_screen is irrelevant for LoD because LOD addons \
         never appear in the eager addons set"
    );
    assert!(
        !raw.contains("## DefaultState:"),
        "TOC must omit `## DefaultState:` — LoD addons load only when the celebration flow \
         requests them; explicit DefaultState would conflict with the on-demand contract"
    );
    assert!(
        !raw.contains("## AllowLoadGameType:"),
        "TOC must omit `## AllowLoadGameType:` — match-celebration is shipped on every flavor \
         that ships the LFG completion reward path"
    );
}

#[test]
fn blizzard_match_celebration_toc_lists_bootstrap_lua_then_xml_in_order() {
    let toc = TocFile::from_file(&match_celebration_toc())
        .expect("Blizzard_MatchCelebrationPartyPoseUI TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        MATCH_CELEBRATION_TOC_FILES,
        "TOC body must list the load helper bootstrap, then Lua before XML. The bootstrap \
         publishes MatchCelebrationPartyPose_LoadUI; the Lua file publishes mixins before the \
         XML-instantiated frame resolves them"
    );
}

#[test]
fn blizzard_match_celebration_directory_holds_four_entries_with_bootstrap() {
    let entries = std::fs::read_dir(match_celebration_dir())
        .expect("Blizzard_MatchCelebrationPartyPoseUI directory should read")
        .count();
    assert_eq!(
        entries, 4,
        "Directory must hold exactly 4 entries: TOC, bootstrap, Lua, and XML"
    );
}

#[test]
fn blizzard_match_celebration_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_MatchCelebrationPartyPoseUI");
        assert!(
            !found,
            "Blizzard_MatchCelebrationPartyPoseUI must NOT appear in any ScreenKind \
             auto-discovery sweep — `## LoadOnDemand: 1` excludes it from every eager pass; only \
             `load_addon` called explicitly by the MatchCelebrationPartyPose_LoadUI flow pulls \
             it in. (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_match_celebration_loads_with_only_known_blizzard_source_bug(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let known_self_leave_button_bug =
        "Blizzard_PartyPoseUI.lua:452: attempt to index local 'button' (a nil value)";

    let unexpected_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            (message.contains("Blizzard_MatchCelebrationPartyPoseUI")
                || message.contains("MatchCelebrationPartyPoseMixin")
                || message.contains("MatchCelebrationPartyPoseFrame")
                || message.contains("MatchCelebrationExtraButtonMixin"))
                && !message.contains(known_self_leave_button_bug)
        })
        .cloned()
        .collect();
    assert!(
        unexpected_errors.is_empty(),
        "Blizzard_MatchCelebrationPartyPoseUI emitted unexpected addon-specific Lua errors \
         during load (the `self.LeaveButton` bug at PartyPoseUI.lua:452 is documented \
         separately):\n  {}",
        unexpected_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_onload_emits_known_self_leave_button_bug(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let bug_message = "Blizzard_PartyPoseUI.lua:452: attempt to index local 'button' (a nil value)";

    let bug_present = env.state().borrow().lua_errors.iter().any(|message| {
        message.contains("MatchCelebrationPartyPoseFrame") && message.contains(bug_message)
    });
    assert!(
        bug_present,
        "MatchCelebrationPartyPoseFrame OnLoad must emit the known Blizzard-source bug — \
         MatchCelebrationPartyPoseMixin:OnLoad (Blizzard_MatchCelebrationPartyPoseUI.lua:5) \
         calls `PartyPoseUtil.AddDismissClickHandler(self.LeaveButton, self)`, but \
         self.LeaveButton is nil because the XML places the LeaveButton at \
         `MatchCelebrationPartyPoseFrame.ButtonContainer.LeaveButton` (xml:45-50, nested inside \
         the ButtonContainer HorizontalLayoutFrame), not at `MatchCelebrationPartyPoseFrame.\
         LeaveButton`. The AddDismissClickHandler helper at PartyPoseUI.lua:447-453 then tries \
         `button:SetScript(\"OnClick\", ...)` on the nil and crashes with `attempt to index \
         local 'button' (a nil value)`. This inconsistency persists in upstream Blizzard \
         source: the LoadPartyPose override (lua:25) correctly uses \
         `self.ButtonContainer.LeaveButton`, but the OnLoad uses `self.LeaveButton`. The \
         simulator preserves the bug verbatim — fixing it would diverge from Blizzard's actual \
         behavior"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_is_addon_loaded_after_explicit_lod_with_dep(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MatchCelebrationPartyPoseUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MatchCelebrationPartyPoseUI') must return true after \
         the explicit load_addon call"
    );

    let party_pose_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PartyPoseUI')")
        .expect("Blizzard_PartyPoseUI IsAddOnLoaded probe should succeed");
    assert!(
        party_pose_loaded,
        "Blizzard_PartyPoseUI dep must register as loaded after the explicit dep load — provides \
         the PartyPoseMixin parent and PartyPoseFrameTemplate the celebration frame consumes"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_mixin_publishes_with_partypose_inherited_methods(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(MatchCelebrationPartyPoseMixin)")
        .expect("MatchCelebrationPartyPoseMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "MatchCelebrationPartyPoseMixin must publish at `_G` as a table — declared at \
         Blizzard_MatchCelebrationPartyPoseUI.lua:1 via `MatchCelebrationPartyPoseMixin = \
         CreateFromMixins(PartyPoseMixin)` so the mixin table inherits every PartyPoseMixin \
         method via shallow copy-on-init"
    );

    for method in MIXIN_OVERRIDE_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MatchCelebrationPartyPoseMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("MatchCelebrationPartyPoseMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MatchCelebrationPartyPoseMixin.{method} must publish as a function — defined \
             directly on the mixin (Blizzard_MatchCelebrationPartyPoseUI.lua) so it overrides \
             the PartyPoseMixin parent's same-named method when the celebration scoreboard \
             flow dispatches"
        );
    }

    let inherited_kind: String = env
        .eval("return type(MatchCelebrationPartyPoseMixin.AddReward)")
        .expect("MatchCelebrationPartyPoseMixin.AddReward probe should succeed");
    assert_eq!(
        inherited_kind, "function",
        "MatchCelebrationPartyPoseMixin.AddReward must publish as a function — proves \
         CreateFromMixins(PartyPoseMixin) copy-on-init pulled in PartyPoseMixin.AddReward (the \
         method ChooseFinalRewards's continuableContainer:ContinueOnLoad callback invokes via \
         `self:AddReward(...)` per the PartyPose reward-loading contract). The celebration \
         mixin doesn't override AddReward — it consumes the parent implementation directly"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_extra_button_mixin_publishes_with_onclick(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(MatchCelebrationExtraButtonMixin)")
        .expect("MatchCelebrationExtraButtonMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "MatchCelebrationExtraButtonMixin must publish at `_G` as a table — declared at \
         Blizzard_MatchCelebrationPartyPoseUI.lua:64 via `MatchCelebrationExtraButtonMixin = \
         {{}}` (NOT a CreateFromMixins descendant — it's a fresh standalone mixin scoped to the \
         ExtraButton's OnClick contract)"
    );

    let onclick_kind: String = env
        .eval("return type(MatchCelebrationExtraButtonMixin.OnClick)")
        .expect("MatchCelebrationExtraButtonMixin.OnClick probe should succeed");
    assert_eq!(
        onclick_kind, "function",
        "MatchCelebrationExtraButtonMixin.OnClick must publish as a function — declared at \
         Blizzard_MatchCelebrationPartyPoseUI.lua:66 to drive the `<OnClick method=\"OnClick\"/>` \
         binding on the ExtraButton (xml line 57). The body fires C_PartyPose.ExtraAction with \
         the active partyPoseID then HideUIPanel(MatchCelebrationPartyPoseFrame), so a single \
         click both kicks off the server-side extra action AND closes the celebration panel"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_named_frame_publishes_with_inherits_and_mixin_chain(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(MatchCelebrationPartyPoseFrame)")
        .expect("MatchCelebrationPartyPoseFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "MatchCelebrationPartyPoseFrame must publish at `_G` as a table — declared at \
         Blizzard_MatchCelebrationPartyPoseUI.xml:3 with \
         `name=\"MatchCelebrationPartyPoseFrame\"` `parent=\"UIParent\"` \
         `inherits=\"PartyPoseFrameTemplate\"` `mixin=\"MatchCelebrationPartyPoseMixin\"` \
         `toplevel=\"true\"`"
    );

    let name: String = env
        .eval("return MatchCelebrationPartyPoseFrame:GetName()")
        .expect("MatchCelebrationPartyPoseFrame:GetName() probe should succeed");
    assert_eq!(name, "MatchCelebrationPartyPoseFrame");
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_named_frame_carries_four_parent_key_children(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    for parent_key in PARENT_KEY_CHILDREN {
        let kind: String = env
            .eval(&format!(
                "return type(MatchCelebrationPartyPoseFrame['{parent_key}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("MatchCelebrationPartyPoseFrame.{parent_key} probe failed: {err}")
            });
        assert_eq!(
            kind, "table",
            "MatchCelebrationPartyPoseFrame.{parent_key} must publish as a parentKey child — \
             declared at Blizzard_MatchCelebrationPartyPoseUI.xml:8-61. The 4 children are: \
             OverlayElements (frameLevel=510 setAllPoints with the parentKey Topper Texture in \
             ARTWORK layer anchored BOTTOM relativePoint=TOP for the texture-kit-driven \
             scoreboard topper); ModelScene (inherits PartyPoseModelFrameTemplate \
             enableMouse=false useParentLevel=true centered — the 3D character pose backdrop); \
             Score (inherits UIWidgetContainerTemplate sized 200x40 with \
             showAndHideOnWidgetSetRegistration=false KeyValue, anchored BOTTOM to the ModelScene \
             at 0,-45 — holds the server-driven score widget); ButtonContainer (inherits \
             HorizontalLayoutFrame with spacing=100 KeyValue, anchored BOTTOM to the ModelScene \
             at y=-40 — wraps the LeaveButton + ExtraButton in a horizontal layout strip)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_overlay_elements_carries_topper_texture(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(MatchCelebrationPartyPoseFrame.OverlayElements.Topper)")
        .expect("OverlayElements.Topper probe should succeed");
    assert_eq!(
        kind, "table",
        "MatchCelebrationPartyPoseFrame.OverlayElements.Topper must publish as a parentKey \
         Texture — the GetPartyPoseDataFromPartyPoseID override sets \
         partyPoseData.themeData.Topper to GetFinalNameFromTextureKit(\"%s-topper\", textureKit) \
         when C_Texture.GetAtlasInfo confirms the per-match texture-kit topper exists \
         (Blizzard_MatchCelebrationPartyPoseUI.lua:38,42), so the Texture's atlas is \
         match-mode-driven at runtime, not baked into the XML"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_modelscene_publishes_as_modelscene_subtype(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let object_type: String = env
        .eval("return MatchCelebrationPartyPoseFrame.ModelScene:GetObjectType()")
        .expect("ModelScene:GetObjectType() probe should succeed");
    assert_eq!(
        object_type, "ModelScene",
        "MatchCelebrationPartyPoseFrame.ModelScene:GetObjectType() must return `ModelScene` — \
         declared as `<ModelScene parentKey=\"ModelScene\" \
         inherits=\"PartyPoseModelFrameTemplate\">` so the loader instantiates the \
         3D-pose-backdrop widget type, not a plain Frame"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_score_widget_container_keeps_register_visibility_off(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let kv: bool = env
        .eval("return MatchCelebrationPartyPoseFrame.Score.showAndHideOnWidgetSetRegistration")
        .expect("Score.showAndHideOnWidgetSetRegistration probe should succeed");
    assert!(
        !kv,
        "MatchCelebrationPartyPoseFrame.Score.showAndHideOnWidgetSetRegistration must equal \
         false — declared via `<KeyValue key=\"showAndHideOnWidgetSetRegistration\" \
         value=\"false\" type=\"boolean\"/>` (Blizzard_MatchCelebrationPartyPoseUI.xml:30) so \
         the Score container's visibility is driven by the celebration scoreboard flow's \
         explicit Show / Hide calls, NOT by the UIWidget API auto-toggling whenever a widget \
         set registers / unregisters"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_button_container_carries_leave_and_extra_buttons(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let leave_kind: String = env
        .eval("return type(MatchCelebrationPartyPoseFrame.ButtonContainer.LeaveButton)")
        .expect("ButtonContainer.LeaveButton probe should succeed");
    assert_eq!(
        leave_kind, "table",
        "MatchCelebrationPartyPoseFrame.ButtonContainer.LeaveButton must publish as a parentKey \
         Button — declared at Blizzard_MatchCelebrationPartyPoseUI.xml:45-50 with \
         `inherits=\"UIPanelButtonNoTooltipResizeToFitTemplate\"` and KeyValues layoutIndex=0 \
         (leftmost slot in the HorizontalLayoutFrame) + minimumWidth=164. The SetLeaveButtonText \
         override sets it to INSTANCE_LEAVE, and PartyPoseUtil.AddDismissClickHandler \
         registers a click handler that fires Dismiss → ConfirmOrLeaveLFGParty"
    );

    let extra_kind: String = env
        .eval("return type(MatchCelebrationPartyPoseFrame.ButtonContainer.ExtraButton)")
        .expect("ButtonContainer.ExtraButton probe should succeed");
    assert_eq!(
        extra_kind, "table",
        "MatchCelebrationPartyPoseFrame.ButtonContainer.ExtraButton must publish as a parentKey \
         Button — declared at Blizzard_MatchCelebrationPartyPoseUI.xml:51-59 with \
         `inherits=\"UIPanelButtonNoTooltipResizeToFitTemplate\"` \
         `mixin=\"MatchCelebrationExtraButtonMixin\"` and KeyValues layoutIndex=1 (rightmost \
         slot in the HorizontalLayoutFrame) + minimumWidth=164. The OnClick script binding \
         dispatches to MatchCelebrationExtraButtonMixin.OnClick which fires \
         C_PartyPose.ExtraAction(partyPoseID) then HideUIPanel(MatchCelebrationPartyPoseFrame)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_button_container_layout_keeps_spacing_keyvalue(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let spacing: f64 = env
        .eval("return MatchCelebrationPartyPoseFrame.ButtonContainer.spacing")
        .expect("ButtonContainer.spacing probe should succeed");
    assert_eq!(
        spacing, 100.0,
        "MatchCelebrationPartyPoseFrame.ButtonContainer.spacing must equal 100 — declared via \
         `<KeyValue key=\"spacing\" value=\"100\" type=\"number\"/>` \
         (Blizzard_MatchCelebrationPartyPoseUI.xml:39). HorizontalLayoutFrame reads this floor \
         when laying out children; 100px is the canonical match-celebration gutter between \
         LeaveButton and ExtraButton so the two ResizeToFit-shrunk buttons always render with a \
         visible gap regardless of locale-specific button-text width"
    );
}
}

prefork_full_ui_case! {
fn blizzard_match_celebration_extra_button_minimum_width_keeps_keyvalue(env: &WowLuaEnv) {
    load_match_celebration_party_pose_ui_with_dependency(env);

    let min_width: f64 = env
        .eval("return MatchCelebrationPartyPoseFrame.ButtonContainer.ExtraButton.minimumWidth")
        .expect("ExtraButton.minimumWidth probe should succeed");
    assert_eq!(
        min_width, 164.0,
        "MatchCelebrationPartyPoseFrame.ButtonContainer.ExtraButton.minimumWidth must equal \
         164 — declared via `<KeyValue key=\"minimumWidth\" value=\"164\" type=\"number\"/>` \
         (Blizzard_MatchCelebrationPartyPoseUI.xml:54). UIPanelButtonNoTooltipResizeToFitTemplate \
         uses this floor when ResizeToFit shrinks the button to fit the dynamic \
         partyPoseInfo.extraButtonText (or CLOSE fallback) so the button never collapses below \
         164px regardless of locale"
    );
}
}
