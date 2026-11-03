use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap_or_else(|_| {
        wow_ui_sim::paths::default_blizzard_ui_addons_path()
            .expect("Blizzard UI cache should be available")
    })
}

fn quick_join_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_QuickJoin")
}

fn quick_join_toc() -> PathBuf {
    quick_join_dir().join("Blizzard_QuickJoin.toc")
}

#[test]
fn quick_join_placeholder_is_absent_before_blizzard_loads() {
    let env = WowLuaEnv::new().unwrap();
    let absent: bool = env.eval("return QuickJoinToastButton == nil").unwrap();
    assert!(absent, "QuickJoin owns the toast button");
}

#[test]
fn friends_update_uses_blizzard_quick_join_toast() {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_CompactRaidFrames", "Blizzard_QuickJoin"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_QuickJoin"));
    env.exec(
        r#"
        assert(QuickJoinToastButton.UpdateDisplayedFriendCount == QuickJoinToastMixin.UpdateDisplayedFriendCount)
        A_Admin.SetPartySize(1)
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert!(
        state.lua_errors.is_empty(),
        "Lua errors: {:?}",
        state.lua_errors
    );
}

const TOC_FILES: &[&str] = &[
    "QuickJoinToast.xml",
    "QuickJoin.xml",
    "QuickJoinSocialView.xml",
    "QuickJoinSocialView.lua",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_FriendsFrame"];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "QuickJoinMixin",
    "QuickJoinButtonMixin",
    "QuickJoinEntriesMixin",
    "QuickJoinEntryMixin",
    "QuickJoinRoleSelectionMixin",
    "JoinQueueButtonMixin",
    "QuickJoinToastMixin",
    "QuickJoinToastGroupMixin",
    "QuickJoinToastThrottleMixin",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &[
    "QuickJoinFrame",
    "QuickJoinRoleSelectionFrame",
    "QuickJoinToastButton",
];

const VIRTUAL_TEMPLATES_SAMPLE: &[&str] = &[
    "QuickJoinButtonMemberTemplate",
    "QuickJoinButtonQueueTemplate",
    "QuickJoinButtonTemplate",
    "QuickJoinToastTemplate",
];

const PUBLIC_GLOBAL_HELPERS: &[&str] = &[
    "QuickJoinToast_GetPriority",
    "QuickJoinToast_GetPriorityFromQueue",
    "QuickJoinToast_GetPriorityFromPlayers",
];

const TOAST_ON_LOAD_REGISTERED_EVENTS: &[&str] = &[
    "SOCIAL_QUEUE_UPDATE",
    "SOCIAL_QUEUE_CONFIG_UPDATED",
    "GROUP_JOINED",
    "GROUP_LEFT",
];

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

// Wrath manifest1-382/cache has no Blizzard_QuickJoin packaged input.
// Only literal TOC/discovery snapshots; all prefork behavior stays enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_join_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&quick_join_dir()).expect("Blizzard_QuickJoin TOC resolves");
    assert_eq!(
        resolved,
        quick_join_toc(),
        "Blizzard_QuickJoin ships a SINGLE bare `Blizzard_QuickJoin.toc` (NO \
         `_Mainline.toc` / `_Mists.toc` / `_Classic.toc` variant — the \
         mainline gate is carried inside the bare TOC via \
         `## AllowLoadGameType: mainline` rather than via a filename suffix). \
         `find_toc_file` walks the suffix-priority list `[_Mainline.toc, .toc]` \
         and falls through to the bare form because no Mainline-suffixed \
         variant exists"
    );

    for variant_suffix in ["_Mainline.toc", "_Mists.toc", "_Wrath.toc", "_Classic.toc"] {
        let variant = quick_join_dir().join(format!("Blizzard_QuickJoin{variant_suffix}"));
        assert!(
            !variant.exists(),
            "Blizzard_QuickJoin must NOT ship a {variant_suffix} variant — \
             single bare TOC only with the flavor gate inside the metadata"
        );
    }
}

#[test]
fn blizzard_quick_join_toc_pins_eager_mainline_only_with_default_state_enabled() {
    let toc = TocFile::from_file(&quick_join_toc()).expect("Blizzard_QuickJoin TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-loaded along with \
         Blizzard_FriendsFrame so the QuickJoin tab content + the toast \
         button + the role-selection dialog are all materialized at startup; \
         the SOCIAL_QUEUE_UPDATE / GROUP_JOINED events register on the toast \
         button OnLoad and fire as soon as the player enters the world"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` — \
         `is_game_type_restricted()` at src/toc.rs:294-302 returns FALSE \
         because `mainline` is in the cross-flavor allowlist alongside \
         `standard`. The loader filter at src/loader/mod.rs:527 keeps this \
         addon in the eager pool on retail; classic clients use a separate \
         older social-queue UI"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC declares `## AllowLoad: Game` (capitalized — pinned by the raw \
         bytes test). `allows_screen` at src/toc.rs:308 routes via \
         `eq_ignore_ascii_case(\"game\")` so both capitalized and lowercase \
         resolve identically"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only screen gate must EXCLUDE {screen:?} — the QuickJoin \
             tab attaches to FriendsFrame which is in-world only"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` — pure stateless display: \
         every group/queue pulls from live C_SocialQueue.GetAllGroups / \
         GetGroupQueues / GetGroupMembers queries each event tick"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}

#[test]
fn blizzard_quick_join_toc_declares_one_dependency() {
    let toc = TocFile::from_file(&quick_join_toc()).expect("TOC parses");
    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 1 hard dep: Blizzard_FriendsFrame. The \
         QuickJoinFrame XML declares `parent=\"FriendsFrame\"` and \
         `setAllPoints=\"true\"` — it overlays the FriendsFrame container \
         from Blizzard_FriendsFrame as a tab content panel. Without the \
         FriendsFrame parent, the XML parent-resolution would fail. The \
         QuickJoin tab itself is added to FriendsFrame's tab strip by the \
         friends frame's tab-registration code"
    );
}

#[test]
fn blizzard_quick_join_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(quick_join_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_QuickJoin"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` — the AddOn list UI \
         shows the addon enabled-by-default for users who toggle it manually"
    );
    assert!(raw.contains("## Dependencies: Blizzard_FriendsFrame"));
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` (CAPITALIZED) — distinct from \
         the lowercase `## AllowLoad: game` form used by Blizzard_PVPUI / \
         Blizzard_QuestTimer / Blizzard_QueueStatusFrame; the parser at \
         src/toc.rs:308 normalizes via eq_ignore_ascii_case so both resolve \
         identically"
    );
    assert!(raw.contains("## AllowLoadGameType: mainline"));

    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT carry any LoadOnDemand directive (eager-loaded)"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT carry any SavedVariables directive"
    );
    assert!(
        !raw.contains("## OnlyBetaAndPTR"),
        "TOC must NOT carry OnlyBetaAndPTR — ships on live retail"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT carry a Version directive — one of the Blizzard_* \
         addons missing the canonical version line (same omission as \
         Blizzard_QuestTimer / Blizzard_QueueStatusFrame)"
    );
}

#[test]
fn blizzard_quick_join_toc_lists_current_toast_frame_and_social_view_files() {
    let toc = TocFile::from_file(&quick_join_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body lists four entries (paths normalized to forward slashes by src/toc.rs:147) \
         in canonical order: QuickJoinToast.xml (which loads QuickJoinToast.lua through its \
         Script entry), QuickJoin.xml (which loads QuickJoin.lua), then the current social-view \
         pair QuickJoinSocialView.xml and QuickJoinSocialView.lua. The social-view files are \
         direct TOC entries, not XML Script inclusions, so the load contract is the literal \
         four-entry ordered list"
    );
}

#[test]
fn blizzard_quick_join_appears_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_QuickJoin");
    assert!(
        game_found,
        "Blizzard_QuickJoin MUST appear in eager Game-screen discovery: no \
         `## LoadOnDemand:` (so `is_load_on_demand()` false), \
         `## AllowLoadGameType: mainline` (so `is_game_type_restricted()` \
         false), `## AllowLoad: Game` (so the Game screen gate passes). All \
         3 loader filters at src/loader/mod.rs:527 admit it"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let glue_found = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_QuickJoin");
        assert!(
            !glue_found,
            "Blizzard_QuickJoin must NOT appear in eager discovery for \
             {screen:?} — the `## AllowLoad: Game` gate excludes glue-screen \
             load paths"
        );
    }
}

#[test]
fn blizzard_quick_join_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_QuickJoin");
    assert!(
        found,
        "Blizzard_QuickJoin MUST appear in `discover_all_blizzard_addons`"
    );
}

}

prefork_full_ui_case! {
fn blizzard_quick_join_loads_in_eager_game_sweep_without_lua_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_QuickJoin")
                || message.contains("QuickJoinFrame")
                || message.contains("QuickJoinToast")
                || message.contains("QuickJoinButton")
                || message.contains("QuickJoinEntries")
                || message.contains("QuickJoinEntry")
                || message.contains("QuickJoinRoleSelection")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_QuickJoin emitted addon-specific Lua errors during eager \
         load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_quick_join_publishes_eight_mixin_globals(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G[{mixin:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {mixin} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_QuickJoin ships \
             exactly 8 public Mixin globals across both Lua files: 5 in \
             QuickJoin.lua (QuickJoinMixin defined as `CreateFromMixins()` \
             owning the QuickJoinFrame OnLoad/OnShow/OnHide/OnEvent state \
             machine with the dynamic-event registration via \
             FrameUtil.RegisterFrameForEvents in OnShow; QuickJoinButtonMixin \
             owning each per-row entry button with hyperlink/click/tooltip \
             handlers; QuickJoinEntriesMixin owning the entry-list manager \
             that wraps C_SocialQueue.GetAllGroups + filtering; \
             QuickJoinEntryMixin owning each individual entry's priority + \
             role + queue state; QuickJoinRoleSelectionMixin owning the \
             tank/healer/dps role picker dialog inheriting \
             RoleSelectionTemplate), and 3 in QuickJoinToast.lua \
             (QuickJoinToastMixin owning the ChatAlertFrame-anchored toast \
             button with the 4-event OnLoad registration; \
             QuickJoinToastGroupMixin owning each per-group toast state; \
             QuickJoinToastThrottleMixin owning the priority-based throttle \
             that gates how often toasts can appear)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_quick_join_publishes_three_named_top_level_frames(env: &WowLuaEnv) {

    for frame_name in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G[{frame_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {frame_name} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame_name} must publish as a frame userdata — \
             QuickJoinFrame is the QuickJoin tab content frame parented to \
             FriendsFrame (NOT UIParent — the addon attaches to the friends \
             frame as a tab page) with `setAllPoints=\"true\"` so it fills \
             the parent's content area, mixin=QuickJoinMixin; \
             QuickJoinRoleSelectionFrame is the role-picker dialog parented \
             to UIParent inheriting RoleSelectionTemplate, mixin=\
             QuickJoinRoleSelectionMixin; QuickJoinToastButton is the \
             ContainedAlertFrame-typed toast button at frameStrata=LOW + \
             frameLevel=4 parented to UIParent inheriting \
             QuickKeybindButtonTemplate, mixin=QuickJoinToastMixin, anchored \
             as a ChatAlertFrame subsystem in OnLoad. All 3 frames start \
             hidden=true (or motionScriptsWhileDisabled=true for the toast)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_quick_join_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_SAMPLE {
        let kind: String = env
            .eval(&format!("return type(_G[{template:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the \
             template registry, NOT the global environment. The 4 virtual \
             templates Blizzard_QuickJoin ships are: \
             QuickJoinButtonMemberTemplate (FontString virtual template \
             inheriting UserScaledFontGameNormalSmall used for member name \
             rows inside each entry), QuickJoinButtonQueueTemplate \
             (FontString virtual template inheriting \
             UserScaledFontGameNormalSmall used for queue-name rows with a \
             placeholder text 'Random Warlords of Draenor Heroic Dungeon or \
             Something' for sizing), QuickJoinButtonTemplate (the per-entry \
             Button virtual template with hyperlinksEnabled=true and \
             mixin=QuickJoinButtonMixin used by the QuickJoinFrame ScrollBox \
             initializer to instantiate one button per group), and \
             QuickJoinToastTemplate (the toast button virtual frame template \
             from QuickJoinToast.xml)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_quick_join_publishes_global_helper_functions(env: &WowLuaEnv) {

    for helper in PUBLIC_GLOBAL_HELPERS {
        let kind: String = env
            .eval(&format!("return type(_G[{helper:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {helper} failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{helper} must publish as a function — Blizzard_QuickJoin \
             ships exactly 3 public helper functions outside the mixin tables: the priority \
             helpers in QuickJoinToast.lua (QuickJoinToast_GetPriority(group, queues, players) \
             computes toast priority from combined queue + player scores; \
             QuickJoinToast_GetPriorityFromQueue(queue) returns per-queue priority; \
             QuickJoinToast_GetPriorityFromPlayers(players) returns per-player-list priority). \
             The join button is now implemented by JoinQueueButtonMixin:OnClick, which calls \
             its parent QuickJoinMixin:JoinQueue()"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_quick_join_toast_registers_four_events_in_onload(env: &WowLuaEnv) {

    for event in TOAST_ON_LOAD_REGISTERED_EVENTS {
        let registered: bool = env
            .eval(&format!(
                "return QuickJoinToastButton:IsEventRegistered({event:?})"
            ))
            .unwrap_or_else(|err| panic!("event probe for {event} failed: {err}"));
        assert!(
            registered,
            "QuickJoinToastButton must register `{event}` in OnLoad — \
             QuickJoinToastMixin:OnLoad calls RegisterEvent for 4 distinct \
             social-queue events: SOCIAL_QUEUE_UPDATE (a group's queue \
             membership changed — drives the toast appearance + throttle \
             check), SOCIAL_QUEUE_CONFIG_UPDATED (the C_SocialQueue.GetConfig \
             throttle parameters changed — re-initializes the \
             QuickJoinToastThrottleMixin), GROUP_JOINED (the player joined \
             a new group — adds a new QuickJoinToastGroupMixin entry to \
             self.groups), and GROUP_LEFT (the player left a group — \
             removes the matching entry). Note that PVP_BRAWL_INFO_UPDATED \
             is registered in OnShow (NOT OnLoad) and unregistered in \
             OnHide, so it is NOT in this OnLoad-registered set"
        );
    }
}
}
