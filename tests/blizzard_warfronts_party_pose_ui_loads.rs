use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn warfronts_party_pose_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_WarfrontsPartyPoseUI")
}

fn warfronts_party_pose_toc() -> PathBuf {
    warfronts_party_pose_dir().join("Blizzard_WarfrontsPartyPoseUI.toc")
}

fn party_pose_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PartyPoseUI/Blizzard_PartyPoseUI.toc")
}

const MIXIN_OVERRIDE_METHODS: &[&str] = &[
    "PlayRewardsAnimations",
    "SetLeaveButtonText",
    "GetPartyPoseData",
    "OnLoad",
    "OnHide",
    "OnEvent",
    "Dismiss",
];

const PARENT_KEY_CHILDREN: &[&str] = &["OverlayElements", "ModelScene", "LeaveButton"];

fn load_warfronts_party_pose_ui_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &party_pose_toc())
        .expect("Blizzard_PartyPoseUI should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &warfronts_party_pose_toc())
        .expect("Blizzard_WarfrontsPartyPoseUI should load via explicit Rust loader call");
}

#[test]
fn find_toc_file_resolves_bare_variant() {
    let resolved = find_toc_file(&warfronts_party_pose_dir())
        .expect("Blizzard_WarfrontsPartyPoseUI TOC should resolve");
    assert_eq!(
        resolved,
        warfronts_party_pose_toc(),
        "Blizzard_WarfrontsPartyPoseUI ships exactly one bare TOC — find_toc_file probes the \
         `_Mainline.toc` variant first (miss) and falls through to the bare TOC name (hit)"
    );
}

#[test]
fn toc_declares_lod_with_one_required_dep() {
    let toc = TocFile::from_file(&warfronts_party_pose_toc())
        .expect("Blizzard_WarfrontsPartyPoseUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_WarfrontsPartyPoseUI declares `## LoadOnDemand: 1` — pulled in by \
         `WarfrontsPartyPose_LoadUI()` in Blizzard_UIParent/Mainline/UIParent.lua via \
         `UIParentLoadAddOn(\"Blizzard_WarfrontsPartyPoseUI\")` when a warfront completes; not \
         loaded eagerly because the typical play session never finishes a warfront"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_PartyPoseUI".to_string()],
        "Blizzard_WarfrontsPartyPoseUI declares `## RequiredDep: Blizzard_PartyPoseUI` \
         (singular `RequiredDep`, not the plural `RequiredDeps` or the more common \
         `Dependencies` — all three keys resolve through the same `dependencies()` accessor at \
         src/toc.rs:209-214). PartyPoseUI provides the PartyPoseMixin parent that \
         WarfrontsPartyPoseMixin extends via CreateFromMixins, the PartyPoseFrameTemplate the \
         named frame inherits, the PartyPoseModelFrameTemplate the ModelScene child inherits, \
         and the PartyPoseUtil.AddDismissClickHandler helper called from OnLoad"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_WarfrontsPartyPoseUI declares no `## OptionalDeps` — the single RequiredDep \
         provides every shared template/mixin the addon consumes (no UIWidgetContainer score \
         widget like the islands variant carries, since warfront rewards stream in via \
         QUEST_LOOT_RECEIVED + QUEST_CURRENCY_LOOT_RECEIVED rather than a server-driven widget \
         set)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_WarfrontsPartyPoseUI declares zero saved variables — every warfront end \
         screen re-fetches reward data from QUEST_LOOT_RECEIVED + QUEST_CURRENCY_LOOT_RECEIVED \
         events keyed against the SCENARIO_COMPLETED questID; nothing persists across reloads"
    );
}

#[test]
fn toc_omits_allow_load_directives() {
    let toc = TocFile::from_file(&warfronts_party_pose_toc())
        .expect("Blizzard_WarfrontsPartyPoseUI TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, allows_screen falls through to the default Game-only branch \
         at src/toc.rs:311 (None → Game)"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Without `## AllowLoad`, allows_screen rejects every glue screen ({screen:?}) — \
             the warfront end screen is in-world UI only"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "Without `## AllowLoadGameType`, is_game_type_restricted (src/toc.rs:294-302) returns \
         false via the `unwrap_or(false)` branch — the addon is treated as gametype-unrestricted"
    );

    let raw = std::fs::read_to_string(warfronts_party_pose_toc())
        .expect("Blizzard_WarfrontsPartyPoseUI TOC should read");
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must omit `## AllowLoad:` — LoadOnDemand addons rely on the explicit \
         `UIParentLoadAddOn` call from Blizzard_UIParent rather than the screen auto-discovery \
         sweep"
    );
    assert!(
        !raw.contains("## AllowLoadGameType:"),
        "TOC must omit `## AllowLoadGameType:` — the warfront end screen is unconditionally \
         available on every retail flavor that exposes Battle for Azeroth scenario data"
    );
    assert!(
        !raw.contains("## DefaultState:"),
        "TOC must omit `## DefaultState:` — LoD addons load only when the warfront flow \
         requests them; explicit DefaultState would conflict with the on-demand contract"
    );
}

#[test]
fn toc_lists_bootstrap_lua_then_xml_in_order() {
    let toc = TocFile::from_file(&warfronts_party_pose_toc())
        .expect("Blizzard_WarfrontsPartyPoseUI TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "Blizzard_WarfrontsPartyPoseUI_Bootstrap.lua".to_string(),
            "Blizzard_WarfrontsPartyPoseUI.lua".to_string(),
            "Blizzard_WarfrontsPartyPoseUI.xml".to_string(),
        ],
        "Retail 12.1.0.69497 TOC lists the bootstrap before lua and xml; the lua file declares WarfrontsPartyPoseMixin = \
         CreateFromMixins(PartyPoseMixin) at file scope (line 1) before any XML-instantiated \
         frame's `mixin=\"WarfrontsPartyPoseMixin\"` attribute or `<OnLoad method=\"OnLoad\"/>` \
         script binding tries to resolve the mixin table via `_G`"
    );
}

#[test]
fn toc_raw_bytes_pin_required_directives() {
    let raw = std::fs::read_to_string(warfronts_party_pose_toc())
        .expect("Blizzard_WarfrontsPartyPoseUI TOC should read");

    for directive in [
        "## Title: Blizzard_WarfrontsPartyPoseUI",
        "## Author: Blizzard Entertainment",
        "## RequiredDep: Blizzard_PartyPoseUI",
        "## Version: 1.0",
        "## LoadOnDemand: 1",
    ] {
        assert!(
            raw.contains(directive),
            "TOC must contain directive line `{directive}`"
        );
    }

    for body in [
        "Blizzard_WarfrontsPartyPoseUI_Bootstrap.lua [Bootstrap]",
        "Blizzard_WarfrontsPartyPoseUI.lua",
        "Blizzard_WarfrontsPartyPoseUI.xml",
    ] {
        assert!(
            raw.contains(body),
            "TOC must contain body file line `{body}`"
        );
    }
}

#[test]
fn directory_holds_four_entries_with_bootstrap() {
    let entries = std::fs::read_dir(warfronts_party_pose_dir())
        .expect("Blizzard_WarfrontsPartyPoseUI directory should read")
        .count();
    assert_eq!(
        entries, 4,
        "Retail 12.1.0.69497 directory holds the TOC, bootstrap Lua, main Lua, and XML"
    );
}

#[test]
fn excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_WarfrontsPartyPoseUI");
        assert!(
            !found,
            "Blizzard_WarfrontsPartyPoseUI must NOT appear in any ScreenKind auto-discovery \
             sweep — `## LoadOnDemand: 1` excludes it from every eager pass; only an explicit \
             load_addon call (driven by WarfrontsPartyPose_LoadUI in Blizzard_UIParent) pulls \
             it in. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn dep_directory_exists_on_disk() {
    let dep_toc = party_pose_toc();
    assert!(
        dep_toc.exists(),
        "Blizzard_PartyPoseUI dep TOC must exist on disk at `{}` — the addon is itself \
         LoadOnDemand:1 so it is not auto-discovered, but its directory must be present so the \
         test harness can pre-load it before pulling in WarfrontsPartyPoseUI",
        dep_toc.display()
    );
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_WarfrontsPartyPoseUI")
                || message.contains("WarfrontsPartyPoseMixin")
                || message.contains("WarfrontsPartyPoseFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_WarfrontsPartyPoseUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_lod_with_deps(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let warfronts_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_WarfrontsPartyPoseUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        warfronts_loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_WarfrontsPartyPoseUI') must return true after the \
         explicit load_addon call"
    );

    let party_pose_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PartyPoseUI')")
        .expect("Blizzard_PartyPoseUI IsAddOnLoaded probe should succeed");
    assert!(
        party_pose_loaded,
        "Blizzard_PartyPoseUI dep must register as loaded after the explicit dep load — \
         provides the PartyPoseMixin parent and PartyPoseFrameTemplate the warfronts frame \
         consumes"
    );
}
}

prefork_full_ui_case! {
fn loader_function_publishes_in_uiparent(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WarfrontsPartyPose_LoadUI)")
        .expect("WarfrontsPartyPose_LoadUI probe should succeed");
    assert_eq!(
        kind, "function",
        "WarfrontsPartyPose_LoadUI must publish at `_G` as a function — declared at \
         Blizzard_UIParent/Mainline/UIParent.lua:387 as the only call site that issues \
         `UIParentLoadAddOn(\"Blizzard_WarfrontsPartyPoseUI\")`. Blizzard_UIParent loads eagerly \
         during the Game-screen sweep, so the loader hook is published BEFORE the addon itself \
         is brought in via the explicit load chain"
    );
}
}

prefork_full_ui_case! {
fn mixin_publishes_with_partypose_inherited_methods(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WarfrontsPartyPoseMixin)")
        .expect("WarfrontsPartyPoseMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "WarfrontsPartyPoseMixin must publish at `_G` as a table — declared at \
         Blizzard_WarfrontsPartyPoseUI.lua:1 via `WarfrontsPartyPoseMixin = \
         CreateFromMixins(PartyPoseMixin)` so the mixin table inherits every PartyPoseMixin \
         method via copy-on-init"
    );

    for method in MIXIN_OVERRIDE_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(WarfrontsPartyPoseMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("WarfrontsPartyPoseMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "WarfrontsPartyPoseMixin.{method} must publish as a function — defined directly on \
             the mixin (Blizzard_WarfrontsPartyPoseUI.lua) so it overrides the PartyPoseMixin \
             parent's same-named method when the warfronts scoreboard flow dispatches"
        );
    }

    let inherited_kind: String = env
        .eval("return type(WarfrontsPartyPoseMixin.AddReward)")
        .expect("WarfrontsPartyPoseMixin.AddReward probe should succeed");
    assert_eq!(
        inherited_kind, "function",
        "WarfrontsPartyPoseMixin.AddReward must publish as a function — proves \
         CreateFromMixins(PartyPoseMixin) copy-on-init pulled in PartyPoseMixin.AddReward, the \
         method OnEvent's QUEST_LOOT_RECEIVED branch invokes via `self:AddReward(name, \
         texture, quality, id, \"item\", rewardItemLink, quantity, quantity, false)` per \
         Blizzard_WarfrontsPartyPoseUI.lua:155"
    );
}
}

prefork_full_ui_case! {
fn named_frame_publishes_with_inherits_and_mixin_chain(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WarfrontsPartyPoseFrame)")
        .expect("WarfrontsPartyPoseFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "WarfrontsPartyPoseFrame must publish at `_G` as a table — declared at \
         Blizzard_WarfrontsPartyPoseUI.xml:3 with `name=\"WarfrontsPartyPoseFrame\"` \
         `parent=\"UIParent\"` `inherits=\"PartyPoseFrameTemplate\"` \
         `mixin=\"WarfrontsPartyPoseMixin\"` `toplevel=\"true\"`"
    );

    let name: String = env
        .eval("return WarfrontsPartyPoseFrame:GetName()")
        .expect("WarfrontsPartyPoseFrame:GetName() probe should succeed");
    assert_eq!(name, "WarfrontsPartyPoseFrame");
}
}

prefork_full_ui_case! {
fn named_frame_carries_three_parent_key_children(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    for parent_key in PARENT_KEY_CHILDREN {
        let kind: String = env
            .eval(&format!(
                "return type(WarfrontsPartyPoseFrame['{parent_key}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("WarfrontsPartyPoseFrame.{parent_key} probe failed: {err}")
            });
        assert_eq!(
            kind, "table",
            "WarfrontsPartyPoseFrame.{parent_key} must publish as a parentKey child — declared \
             at Blizzard_WarfrontsPartyPoseUI.xml:7-27. The 3 children are: OverlayElements \
             (frameLevel=510 setAllPoints with the parentKey Topper Texture in ARTWORK layer \
             anchored BOTTOM relativePoint=TOP for the faction-specific scoreboard topper); \
             ModelScene (inherits PartyPoseModelFrameTemplate useParentLevel=true — the 3D \
             character pose backdrop with grunt actor IDs keyed by mapID 1876/1943/2111/2105 \
             for Horde/Alliance Arathi + Darkshore); LeaveButton (inherits \
             UIPanelButtonNoTooltipResizeToFitTemplate with minimumWidth=164 KeyValue, anchored \
             BOTTOMRIGHT to the ModelScene at -20,-45 — fires Dismiss → LeaveInstanceParty + \
             PartyPoseMixin.Dismiss). Note: NO Score child unlike IslandsPartyPoseUI — \
             warfronts stream rewards via QUEST_LOOT_RECEIVED + QUEST_CURRENCY_LOOT_RECEIVED \
             rather than a server-driven UIWidgetContainer"
        );
    }
}
}

prefork_full_ui_case! {
fn overlay_elements_carries_topper_texture(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let kind: String = env
        .eval("return type(WarfrontsPartyPoseFrame.OverlayElements.Topper)")
        .expect("OverlayElements.Topper probe should succeed");
    assert_eq!(
        kind, "table",
        "WarfrontsPartyPoseFrame.OverlayElements.Topper must publish as a parentKey Texture — \
         the GetPartyPoseData override sets partyPoseData.themeData.Topper to either \
         `scoreboard-horde-header` or `scoreboard-alliance-header` (per UnitFactionGroup) so \
         the Texture's atlas is faction-driven at runtime, not baked into the XML"
    );
}
}

prefork_full_ui_case! {
fn modelscene_publishes_as_modelscene_subtype(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let object_type: String = env
        .eval("return WarfrontsPartyPoseFrame.ModelScene:GetObjectType()")
        .expect("ModelScene:GetObjectType() probe should succeed");
    assert_eq!(
        object_type, "ModelScene",
        "WarfrontsPartyPoseFrame.ModelScene:GetObjectType() must return `ModelScene` — \
         declared as `<ModelScene parentKey=\"ModelScene\" \
         inherits=\"PartyPoseModelFrameTemplate\" useParentLevel=\"true\"/>` so the loader \
         instantiates the 3D-pose-backdrop widget type, not a plain Frame"
    );
}
}

prefork_full_ui_case! {
fn leave_button_keeps_minimum_width_keyvalue(env: &WowLuaEnv) {
    load_warfronts_party_pose_ui_with_dependency(env);

    let min_width: f64 = env
        .eval("return WarfrontsPartyPoseFrame.LeaveButton.minimumWidth")
        .expect("LeaveButton.minimumWidth probe should succeed");
    assert_eq!(
        min_width, 164.0,
        "WarfrontsPartyPoseFrame.LeaveButton.minimumWidth must equal 164 — declared via \
         `<KeyValue key=\"minimumWidth\" value=\"164\" type=\"number\"/>` at \
         Blizzard_WarfrontsPartyPoseUI.xml:22. UIPanelButtonNoTooltipResizeToFitTemplate uses \
         this floor when ResizeToFit shrinks the button to fit the WARFRONTS_LEAVE locale \
         string so the button never collapses below 164px"
    );
}
}
