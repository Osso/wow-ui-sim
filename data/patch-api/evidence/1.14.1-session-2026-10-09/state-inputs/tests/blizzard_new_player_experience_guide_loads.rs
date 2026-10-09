#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addons_for_screen, find_toc_file, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn guide_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_NewPlayerExperienceGuide")
}

fn guide_toc() -> PathBuf {
    guide_dir().join("Blizzard_NewPlayerExperienceGuide.toc")
}

const GUIDE_TOC_FILES: &[&str] = &["GuideCriteriaFrame.xml", "GuideFrame.xml"];

const PUBLIC_MIXINS: &[&str] = &[
    "GuideFrameMixin",
    "CriteriaDisplayMixin",
    "CriteriaBulletMixin",
    "CriterionMixin",
];

const NAMED_FRAMES: &[&str] = &["GuideFrame"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] =
    &["CriteriaBulletTemplate", "CriteriaDisplayTemplate"];

fn load_new_player_experience_guide_after_fork(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &guide_toc()).expect(
        "Blizzard_NewPlayerExperienceGuide load_addon succeeds after eager Game-screen sweep",
    );
}

#[test]
fn blizzard_npe_guide_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&guide_dir()).expect("Blizzard_NewPlayerExperienceGuide TOC resolves");
    assert_eq!(
        resolved,
        guide_toc(),
        "Blizzard_NewPlayerExperienceGuide ships exactly one bare TOC — no `_Mainline.toc` and \
         no `_Classic.toc`. The mentor/guide UI is a retail-only feature (it drives the \
         C_PlayerMentorship API and the NPEv2 `Be a Guide` apply flow), but the retail-onliness \
         is expressed via the absence of classic-flavor TOCs rather than via a flavor-split \
         metadata key — `find_toc_file` resolves the bare TOC after the `_Mainline.toc` lookup \
         misses"
    );

    let mainline = guide_dir().join("Blizzard_NewPlayerExperienceGuide_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the retail-only guide UI ships a single bare \
         TOC; flavor restriction is enforced implicitly via the C_PlayerMentorship namespace \
         (only present on retail) rather than at the TOC layer",
        mainline.display()
    );
}

#[test]
fn blizzard_npe_guide_toc_declares_load_on_demand_with_no_dependencies() {
    let toc =
        TocFile::from_file(&guide_toc()).expect("Blizzard_NewPlayerExperienceGuide TOC parses");
    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` — the guide UI is summoned by an NPC gossip \
         interaction (the mentor sign-up dialog), so eager-loading would waste resources on \
         every login. The `## LoadOnDemand: 1` route at src/loader/mod.rs:530-534 keeps the \
         addon out of the eager Game-screen discovery sweep until something explicitly calls \
         load_addon"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## RequiredDep:` / `## Dependencies:` — the guide UI has NO hard dependencies. \
         Unlike Blizzard_NewPlayerExperience (which depends on Blizzard_TutorialManager), the \
         guide UI is self-contained: it inherits PortraitFrameTemplate / ScrollFrameTemplate / \
         UIPanelButtonTemplate from foundational SharedXML / FrameXML (always loaded) and \
         calls C_PlayerMentorship / C_GossipInfo / C_SocialRestrictions (built-in C_* \
         namespaces, no Lua dependency)"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons. Every surface the guide UI touches \
         is either foundational FrameXML (UIPanelWindows, PlaySound, SOUNDKIT) or a built-in \
         C_* namespace; nothing is conditionally enhanced by another addon"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the guide UI is purely server-driven. Eligibility \
         (achievements / level / mentorship status / mute status / trial status) lives on the \
         server; the Lua side queries C_PlayerMentorship.GetMentorshipStatus and \
         C_PlayerMentorship.IsMentorRestricted on demand. Choice persistence (Mentor or not) is \
         the gossip-option selection, not a Lua SavedVariable"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — the guide UI is implicitly retail-only via the \
         C_PlayerMentorship namespace, but the file-list itself loads on every game type that \
         resolves the gossip trigger. `is_game_type_restricted()` at src/toc.rs:294 returns \
         false when the metadata key is absent"
    );
}

#[test]
fn blizzard_npe_guide_toc_declares_load_on_demand_in_raw_bytes() {
    let raw = std::fs::read_to_string(guide_toc())
        .expect("Blizzard_NewPlayerExperienceGuide TOC reads as utf-8");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly. The explicit `1` (rather than \
         omitting / `## LoadOnDemand: 0`) is what routes the addon to the lod_pool at \
         src/loader/mod.rs:530-534, keeping it out of the eager Game-screen discovery sweep"
    );
    assert!(
        !raw.contains("## RequiredDep:"),
        "TOC must NOT declare `## RequiredDep:` — the guide UI has zero hard dependencies. \
         Unlike Blizzard_NewPlayerExperience which requires Blizzard_TutorialManager, the \
         guide UI is self-contained against foundational FrameXML + built-in C_* namespaces"
    );
    assert!(
        !raw.contains("## Dependencies:"),
        "TOC must NOT declare `## Dependencies:` either — the alternate spelling for \
         RequiredDep, also absent. `dependencies()` at src/toc.rs:210-217 reads RequiredDep / \
         Dependencies / RequiredDeps as aliases; all three must be absent"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys (covers SavedVariables and \
         SavedVariablesPerCharacter). The guide UI is server-driven — eligibility queries \
         resolve to live C_PlayerMentorship state, not persisted Lua tables"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:`. With `## AllowLoad:` omitted, `allows_screen` \
         at src/toc.rs:311 defaults to Game-only (`screen == ScreenKind::Game`) — but the \
         LoadOnDemand routing means the addon does not appear in eager discovery on any \
         screen anyway; the AllowLoad default applies to all addons regardless of LoD status"
    );
}

#[test]
fn blizzard_npe_guide_toc_lists_two_xml_files_only() {
    let toc =
        TocFile::from_file(&guide_toc()).expect("Blizzard_NewPlayerExperienceGuide TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, GUIDE_TOC_FILES,
        "TOC body must list exactly 2 XML files in the canonical guide order: \
         GuideCriteriaFrame.xml first (declares the two virtual templates \
         CriteriaBulletTemplate + CriteriaDisplayTemplate plus the CriterionMixin / \
         CriteriaDisplayMixin / CriteriaBulletMixin globals via inline `<Script \
         file=\"GuideCriteriaFrame.lua\"/>`), then GuideFrame.xml (declares the named \
         GuideFrame inheriting PortraitFrameTemplate, with an ObjectivesFrame child inheriting \
         CriteriaDisplayTemplate from the first file). The Lua files (GuideFrame.lua, \
         GuideCriteriaFrame.lua) are NOT listed in the TOC body — they load transitively via \
         `<Script file=\"...\"/>` directives at the top of each XML file"
    );

    for entry in &listed {
        assert!(
            entry.ends_with(".xml"),
            "TOC body entry `{entry}` must be an XML file — this addon's TOC lists ONLY XML; \
             the Lua files are loaded transitively from `<Script file=\"...\"/>` directives \
             inside each XML, never from the TOC body. This is a different load pattern from \
             Blizzard_NewPlayerExperience (whose TOC body lists 1 XML + 6 Lua files in load \
             order)"
        );
    }
}

#[test]
fn blizzard_npe_guide_does_not_appear_in_eager_discovery_on_any_screen() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_NewPlayerExperienceGuide");
        assert!(
            !found,
            "Blizzard_NewPlayerExperienceGuide must NOT auto-discover on screen {screen:?} — \
             `## LoadOnDemand: 1` routes the addon to the lod_pool at \
             src/loader/mod.rs:530-534, not the eager set. The guide UI is loaded on demand by \
             an NPC gossip interaction (the mentor sign-up dialog) — there is no other entry \
             point that would justify eager loading"
        );
    }
}

#[test]
fn blizzard_npe_guide_appears_in_discover_all_blizzard_addons() {
    let all = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = all
        .iter()
        .any(|(name, _)| name == "Blizzard_NewPlayerExperienceGuide");
    assert!(
        found,
        "Blizzard_NewPlayerExperienceGuide must appear in `discover_all_blizzard_addons` — \
         that helper enumerates every `Blizzard_*` directory regardless of LoD or screen \
         restriction. The addon-management UI relies on this exhaustive sweep to render every \
         addon row, including LoD addons that are not eagerly discovered"
    );
}

prefork_full_ui_case! {
fn blizzard_npe_guide_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_NewPlayerExperienceGuide")
                || message.contains("GuideFrame")
                || message.contains("GuideCriteriaFrame")
                || message.contains("CriteriaBullet")
                || message.contains("CriteriaDisplay")
                || message.contains("CriterionMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_NewPlayerExperienceGuide emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_npe_guide_is_addon_loaded_after_explicit_load_addon_call(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_NewPlayerExperienceGuide')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_NewPlayerExperienceGuide') must return true after \
         the explicit load_addon call — proves the LoadOnDemand routing reaches the loaded-set \
         only via explicit request, not via eager discovery. On retail, the gossip handler \
         calls a load_addon equivalent when the player interacts with a mentor-program NPC"
    );
}
}

prefork_full_ui_case! {
fn blizzard_npe_guide_publishes_four_mixin_tables(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    for global in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{global})"))
            .unwrap_or_else(|err| panic!("type(_G.{global}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{global} must publish as a table after Blizzard_NewPlayerExperienceGuide \
             loads. `GuideFrameMixin` (GuideFrame.lua line 10) is the mixin attached to the \
             named GuideFrame via the XML `mixin=\"GuideFrameMixin\"` attribute — it owns 13 \
             methods (OnLoad / OnEvent / OnShow / OnHide / SetStateInternal / \
             BeginGuideInteraction / SetDescription / GetDescription / SetCanGuide / CanGuide / \
             SetStateCannotGuide / SetState / GetState / ConfirmChoice). `CriteriaDisplayMixin` \
             (GuideCriteriaFrame.lua line 25) is attached to CriteriaDisplayTemplate (and \
             therefore to GuideFrame.ScrollFrame.Child.ObjectivesFrame which inherits it) — it \
             owns OnLoad / SetTitle / AddCriterion / ClearCriteria / Update. \
             `CriteriaBulletMixin` (line 75) is attached to CriteriaBulletTemplate which is \
             pooled inside CriteriaDisplayMixin via CreateFramePool — it owns SetUp / \
             CheckSetFontOverride / OnHyperlinkClick. `CriterionMixin` (line 1) is the \
             prototype constructed by CreateAndInitFromMixin from inside \
             CriteriaDisplayMixin:AddCriterion — it owns Init / IsComplete / GetText. All four \
             must be tables for the XML mixin attribute and the CreateAndInitFromMixin call to \
             succeed at frame instantiation"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_npe_guide_publishes_named_frame_as_global(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    for frame in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a userdata-backed Frame — GuideFrame.xml line 5 \
             declares `<Frame name=\"{frame}\" toplevel=\"true\" parent=\"UIParent\" \
             inherits=\"PortraitFrameTemplate\" mixin=\"GuideFrameMixin\" hidden=\"true\">`. \
             The frame is 359x608, hidden by default (the gossip handler shows it on demand), \
             and inherits PortraitFrameTemplate so the title-bar portrait icon is set via \
             SetPortraitToAsset(\"Interface/Icons/UI_GreenFlag\") in OnLoad. The \
             UIPanelWindows[\"GuideFrame\"] entry registered on GuideFrame.lua line 1 lets \
             ShowUIPanel position it in the left slot when shown"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_npe_guide_does_not_leak_virtual_templates_as_globals(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — `<Frame name=\"{template}\" ... virtual=\"true\">` at \
             top level of GuideCriteriaFrame.xml registers the frame as a TEMPLATE in the XML \
             template registry, NOT as a `_G` global. Templates are looked up by name from \
             `inherits=\"...\"` attributes on other XML frames (the ObjectivesFrame in \
             GuideFrame.xml inherits CriteriaDisplayTemplate; CreateFramePool resolves \
             CriteriaBulletTemplate by name) and from CreateFrame's template parameter — never \
             via `_G`. Distinct from the unusual virtual+nested-Frames quirk in \
             Blizzard_NewPlayerExperience where KeyboardMouseConfirmButton DID leak: there the \
             virtual button was nested inside a parent's `<Frames>` block; here the virtual \
             templates are top-level XML elements"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_npe_guide_populates_enum_guide_frame_state_constants(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    let kind: String = env
        .eval("return type(_G.Enum.GuideFrameState)")
        .expect("Enum.GuideFrameState probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.Enum.GuideFrameState must publish as a table — GuideFrame.lua lines 3-8 declare \
         `Enum.GuideFrameState = {{ StartGuiding = 1, StopGuiding = 2, CannotGuide = 3 }};`. \
         This is one of the addon-extends-Enum patterns: the `Enum` global is foundational \
         (populated by the engine before any Lua runs), but addons can add subtables to it. \
         `Enum.GuideFrameState` keys the `stateSetup` table inside GuideFrameMixin (mapping \
         each state to its title / description / button text) and the `GuideFrameStateHandlers` \
         dispatch table (mapping each state to SetStateInternal or SetStateCannotGuide)"
    );

    for (name, expected) in [
        ("StartGuiding", 1i64),
        ("StopGuiding", 2),
        ("CannotGuide", 3),
    ] {
        let actual: i64 = env
            .eval(&format!("return _G.Enum.GuideFrameState.{name}"))
            .unwrap_or_else(|err| panic!("Enum.GuideFrameState.{name} probe failed: {err}"));
        assert_eq!(
            actual, expected,
            "Enum.GuideFrameState.{name} must equal {expected} — the StartGuiding=1 / \
             StopGuiding=2 / CannotGuide=3 ordering is locked by GuideFrameStateHandlers \
             dispatch (line 240-244): StartGuiding and StopGuiding both route to \
             SetStateInternal (the standard render path), while CannotGuide routes to \
             SetStateCannotGuide (the error-message override path). Reordering would silently \
             swap which state shows the eligibility-blocked message"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_npe_guide_registers_uipanelwindows_entry(env: &WowLuaEnv) {
    load_new_player_experience_guide_after_fork(env);

    let kind: String = env
        .eval("return type(_G.UIPanelWindows.GuideFrame)")
        .expect("UIPanelWindows.GuideFrame probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.UIPanelWindows.GuideFrame must publish as a table — GuideFrame.lua line 1 \
         declares `UIPanelWindows[\"GuideFrame\"] = {{ area = \"left\", pushable = 1, \
         whileDead = 1, width = 359, height = 608 }};`. The UIPanelWindows registry is \
         consulted by ShowUIPanel / HideUIPanel to decide where on screen the frame snaps \
         (left / center / right slots, with stacking rules) and whether the frame remains \
         shown while dead. Without this entry, ShowUIPanel(GuideFrame) would refuse to \
         position the frame"
    );

    let area: String = env
        .eval("return _G.UIPanelWindows.GuideFrame.area")
        .expect("UIPanelWindows.GuideFrame.area probe succeeds");
    assert_eq!(
        area, "left",
        "UIPanelWindows.GuideFrame.area must equal \"left\" — the guide UI snaps to the \
         left-slot panel position (next to QuestLogFrame / CharacterFrame), reflecting that \
         it is a player-facing modal dialog rather than a tooltip / overlay"
    );
}
}
