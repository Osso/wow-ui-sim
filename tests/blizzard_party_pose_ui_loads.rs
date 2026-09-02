#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn party_pose_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PartyPoseUI")
}

fn party_pose_toc() -> PathBuf {
    party_pose_dir().join("Blizzard_PartyPoseUI.toc")
}

const PARTY_POSE_TOC_FILES: &[&str] = &[
    "Blizzard_PartyPoseUI_Bootstrap.lua",
    "Blizzard_PartyPoseUI.lua",
    "Blizzard_PartyPoseUI.xml",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_Colors"];

const PUBLIC_MIXINS: &[&str] = &["PartyPoseRewardsMixin", "PartyPoseMixin"];

const PUBLIC_NAMESPACE_TABLES: &[&str] = &["PartyPoseUtil"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "PartyPoseRewardsButtonTemplate",
    "PartyPoseFrameTemplate",
    "PartyPoseModelFrameTemplate",
    "PartyPoseModelShadowTextureTemplate",
];

fn load_full_game_ui_with_party_pose() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &party_pose_toc())
        .expect("explicit load_addon for Blizzard_PartyPoseUI succeeds");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_party_pose_ui_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&party_pose_dir()).expect("Blizzard_PartyPoseUI TOC resolves");
    assert_eq!(
        resolved,
        party_pose_toc(),
        "Blizzard_PartyPoseUI ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The party-pose UI is the end-of-encounter / end-of-scenario victory pose screen \
         (think Mythic+ completion screen with Bwonsamdi twirling, the dungeon-finish \
         screen with the boss model, etc.); a retail-only feature but the TOC stayed bare \
         because the addon predates the flavor-split convention"
    );

    let mainline = party_pose_dir().join("Blizzard_PartyPoseUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_party_pose_ui_toc_declares_load_on_demand_with_blizzard_colors_dep() {
    let toc = TocFile::from_file(&party_pose_toc()).expect("Blizzard_PartyPoseUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true — the \
         party-pose UI is a heavy 3D ModelScene-based panel that only matters at the end \
         of an encounter; lazy-loading defers the model-scene cost until \
         `LoadAddOn('Blizzard_PartyPoseUI')` is fired by the encounter-end gossip handler"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` at src/toc.rs:311 returns true for ScreenKind::Game \
         when AllowLoad is omitted"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — party-pose only happens \
             in-world after combat/scenario completion; glue screens cannot trigger it"
        );
    }

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "TOC must declare exactly 1 RequiredDep (`Blizzard_Colors`) — the rewards button \
         calls `ColorManager.GetColorDataForItemQuality(quality)` to set the icon-border \
         tint, so Blizzard_Colors's ColorManager / BAG_ITEM_QUALITY_COLORS / \
         AUCTION_HOUSE_ITEM_QUALITY_ICON_BORDER_ATLASES tables must be initialized before \
         PartyPoseUI loads. `dependencies()` at src/toc.rs:210-217 reads `Dependencies` as \
         the canonical retail spelling here"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the party-pose UI is a stateless mirror of \
         encounter-completion data fetched from C_PartyPose APIs every time \
         LoadScreenByPartyPoseID is called"
    );
}

#[test]
fn blizzard_party_pose_ui_toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(party_pose_toc()).expect("Blizzard_PartyPoseUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PartyPoseUI"),
        "TOC must declare `## Title: Blizzard_PartyPoseUI` exactly. UNUSUAL: the title uses \
         the underscore-namespace spelling (rather than the space-and-prose form like \
         `Blizzard Party Pose UI`); the minority pattern, suggesting the addon was \
         scaffolded from a code template"
    );
    assert!(
        raw.contains("## Notes: Every time I'm here it's a party! :3"),
        "TOC must declare the WHIMSICAL `## Notes:` line verbatim — UNUSUAL: most \
         Blizzard-shipped addons OMIT the `## Notes:` key entirely; this one ships a \
         whimsical author-comment-as-tooltip-text (`:3` emoticon and all). The Notes key \
         is engine-visible in the AddOns panel as the addon's tooltip — this one was \
         written by a developer who knew the line would surface"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## Version: 1.0"),
        "TOC must declare `## Version: 1.0` exactly — UNUSUAL: stub version that nobody \
         updates despite the file having seen multiple expansions of changes; matches the \
         Blizzard_OrderHallUI pattern"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the canonical retail spelling \
         for explicit lazy loading"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_Colors"),
        "TOC must declare `## Dependencies: Blizzard_Colors` exactly (singular dep, no \
         comma)"
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
}

#[test]
fn blizzard_party_pose_ui_toc_lists_bootstrap_lua_then_xml() {
    let toc = TocFile::from_file(&party_pose_toc()).expect("Blizzard_PartyPoseUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PARTY_POSE_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, then Blizzard_PartyPoseUI.lua and \
         Blizzard_PartyPoseUI.xml. The main Lua still publishes the mixins before XML parses"
    );
}

#[test]
fn blizzard_party_pose_ui_does_not_appear_in_eager_discovery_for_any_screen() {
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
            .any(|(name, _)| name == "Blizzard_PartyPoseUI");
        assert!(
            !found,
            "Blizzard_PartyPoseUI must NOT appear in eager discovery for {screen:?} — \
             LoadOnDemand: 1 keeps the addon in the lod_pool at src/loader/mod.rs:530-534, \
             not in the eager-discovery set"
        );
    }
}

#[test]
fn blizzard_party_pose_ui_appears_in_full_addon_inventory() {
    let ui = blizzard_ui_dir();
    let inventory = discover_all_blizzard_addons(&ui);
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PartyPoseUI");
    assert!(
        found,
        "Blizzard_PartyPoseUI must appear in `discover_all_blizzard_addons` — the full \
         inventory walks every parseable TOC under Interface/BlizzardUI regardless of \
         LoadOnDemand or AllowLoad; LoD addons must be visible in the inventory so users \
         can manually enable/disable them in the addon-manager UI"
    );
}

#[test]
fn blizzard_party_pose_ui_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_with_party_pose();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PartyPoseUI")
                || message.contains("PartyPose")
                || message.contains("PartyPoseRewards")
                || message.contains("PartyPoseUtil")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PartyPoseUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn blizzard_party_pose_ui_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_with_party_pose();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PartyPoseUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PartyPoseUI') must return true after explicit \
         load_addon — the addon is LoadOnDemand so only the explicit load path makes \
         IsAddOnLoaded report true"
    );
}

#[test]
fn blizzard_party_pose_ui_publishes_two_mixin_tables() {
    let env = load_full_game_ui_with_party_pose();

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_PartyPoseUI.lua declares 2 \
             mixins at module top: PartyPoseRewardsMixin (per-reward button — owns OnLoad / \
             OnEnter / OnLeave / OnHide / SetupReward / IsAzeriteCurrency / \
             SetRewardsQuality / PlayRewardAnimation / PauseRewardAnimation / \
             ResumeRewardAnimation / OnAnimationFinished / PlayNextRewardAnimation / \
             CheckForIndefinitePause; the per-button rewards animation chain that fades in \
             each reward icon in sequence after the encounter ends), and PartyPoseMixin \
             (the panel-level lifecycle owner — owns HideAzeriteGlowModelScenes / \
             PlayNextRewardAnimation / PauseRewardAnimation / ResumeRewardAnimation / \
             CanResumeAnimation / AddReward / GetFirstReward / PlayModelSceneAnimations / \
             UpdateShadow / SetupShadow / SetModelScene / AddCreatureActor / \
             AddModelSceneActors / PlaySounds / GetPartyPoseData / \
             GetPartyPoseDataFromPartyPoseID / LoadScreen / LoadScreenByPartyPoseID / \
             ReloadPartyPose / OnLoad / OnEvent / OnKeyDown / Dismiss; orchestrates the \
             ModelScene actor setup, plays the victory sound, walks the rewards list, and \
             listens for ESC to dismiss). Each mixin is referenced by an XML \
             `mixin=\"...\"` attribute"
        );
    }
}

#[test]
fn blizzard_party_pose_ui_publishes_party_pose_util_namespace() {
    let env = load_full_game_ui_with_party_pose();

    for namespace in PUBLIC_NAMESPACE_TABLES {
        let kind: String = env
            .eval(&format!("return type(_G.{namespace})"))
            .unwrap_or_else(|err| panic!("type(_G.{namespace}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{namespace} must publish as a table — Blizzard_PartyPoseUI.lua declares the \
             PartyPoseUtil namespace at line 445 with the single function \
             `PartyPoseUtil.AddDismissClickHandler(button, panelFrame)` — the public helper \
             that consumer addons (the encounter-end gossip handler, scenario reward \
             screens, etc.) call to wire a custom Dismiss button to the party-pose panel \
             without touching PartyPoseMixin internals"
        );
    }

    let dismiss_helper_present: bool = env
        .eval("return type(PartyPoseUtil.AddDismissClickHandler) == 'function'")
        .expect("AddDismissClickHandler probe succeeds");
    assert!(
        dismiss_helper_present,
        "PartyPoseUtil.AddDismissClickHandler must be a function — it is the only public \
         method on the namespace"
    );
}

#[test]
fn blizzard_party_pose_ui_does_not_leak_virtual_templates_to_globals() {
    let env = load_full_game_ui_with_party_pose();

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — Blizzard_PartyPoseUI.xml ships 4 virtual templates \
             and ZERO named non-virtual frames: PartyPoseRewardsButtonTemplate (the \
             per-reward button bound to PartyPoseRewardsMixin), PartyPoseFrameTemplate \
             (the panel base — FULLSCREEN strata, enableMouse + enableKeyboard, hidden by \
             default; consumer addons inherit this template to build encounter-specific \
             party-pose screens), PartyPoseModelFrameTemplate (the ModelScene template \
             inheriting NonInteractableModelSceneMixinTemplate — owns the 3D actor scene), \
             PartyPoseModelShadowTextureTemplate (the under-actor shadow texture using the \
             scoreboard-charactermodels-shadow atlas with useAtlasSize=true). Virtual \
             templates live in the template registry, NOT in `_G` — leaking would let \
             consumer addons mutate the template definition and break every existing \
             instance"
        );
    }
}
