use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_blizzard_addon_closure_for_screen, discover_blizzard_addons_for_screen, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be synced")
}

fn raf_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RecruitAFriend/Blizzard_RecruitAFriend.toc")
}

const OPTIONAL_DEPENDENCIES: &[&str] = &[
    "Blizzard_BNet",
    "Blizzard_FriendsFrame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_SocialUIShared",
    "Blizzard_TransmogShared",
    "Blizzard_UnitPopup",
];

const MIXIN_TABLES: &[&str] = &[
    "RecruitAFriendSystemMixin",
    "RecruitAFriendFrameMixin",
    "RecruitActivityButtonMixin",
    "RecruitActivityButtonModelMixin",
    "RecruitListButtonMixin",
    "RecruitAFriendNextRewardInfoButtonMixin",
    "RecruitAFriendVersionInfoButtonMixin",
    "RecruitAFriendClaimRewardButtonBaseMixin",
    "RecruitAFriendClaimLegacyRewardsButtonMixin",
    "RecruitAFriendClaimOrViewRewardButtonMixin",
    "RecruitAFriendRewardsFrameMixin",
    "RecruitAFriendRewardMixin",
    "RecruitAFriendRewardButtonMixin",
    "RecruitAFriendRewardButtonWithCheckMixin",
    "RecruitAFriendRewardButtonWithFanfareMixin",
    "RecruitAFriendRewardTabMixin",
    "RecruitAFriendRecruitmentButtonMixin",
    "RecruitAFriendRecruitmentFrameMixin",
    "RecruitAFriendGenerateOrCopyLinkButtonMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "RAFInfoButtonTemplate",
    "RAFClaimRewardButtonBaseTemplate",
    "RecruitAFriendRewardTabTemplate",
    "RecruitAFriendRewardButtonTemplate",
    "RecruitAFriendRewardTemplate",
    "RecruitTextTemplate",
    "RecruitSmallTextTemplate",
    "RecruitActivityButtonTemplate",
    "RecruitListButtonTemplate",
];

const NAMED_TOP_LEVEL_FRAMES: &[&str] = &[
    "RecruitAFriendRewardsFrame",
    "RecruitAFriendRecruitmentFrame",
    "RecruitAFriendFrame",
];

const REGISTERED_EVENTS: &[&str] = &[
    "RAF_SYSTEM_ENABLED_STATUS",
    "RAF_RECRUITING_ENABLED_STATUS",
    "RAF_SYSTEM_INFO_UPDATED",
    "RAF_INFO_UPDATED",
    "BN_FRIEND_INFO_CHANGED",
    "VARIABLES_LOADED",
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
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_recruit_a_friend_toc_pins_eager_game_only_with_optional_social_dependencies() {
    let toc = TocFile::from_file(&raf_toc()).expect("Blizzard_RecruitAFriend TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_RecruitAFriend has no `## LoadOnDemand` line — the RAF system is \
         eagerly loaded so its event handlers (RAF_SYSTEM_ENABLED_STATUS, \
         RAF_RECRUITING_ENABLED_STATUS, RAF_SYSTEM_INFO_UPDATED) are wired up before \
         the user opens the social panel"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_RecruitAFriend has no `## LoadFirst` line — it is a leaf social-panel \
         addon, not part of the early-bootstrap tier"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_RecruitAFriend does not declare `## UseSecureEnvironment` — the RAF \
         panel runs in the standard taint environment"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_RecruitAFriend declares `## AllowLoadGameType: mainline`, but \
         `is_game_type_restricted()` returns false because src/toc.rs:299 treats \
         `mainline` and `standard` as the unrestricted (retail) game type — RAF is \
         retail-only because the C_RecruitAFriend backend doesn't exist on classic \
         flavors"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_RecruitAFriend declares no `## SavedVariables*` — RAF state \
         (recruits, claimed rewards, version info) lives server-side and is fetched \
         via C_RecruitAFriend.GetRAFInfo / GetRAFSystemInfo, never persisted in \
         per-character SavedVars"
    );

    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_RecruitAFriend has no hard `## Dependencies`; it participates in eager \
         game-screen discovery and its social integrations are optional."
    );
    assert_eq!(
        toc.optional_deps(),
        OPTIONAL_DEPENDENCIES
            .iter()
            .map(|dependency| dependency.to_string())
            .collect::<Vec<_>>(),
        "Blizzard_RecruitAFriend declares the current six `## OptionalDeps`, including the \
         newer Blizzard_SocialUIShared and Blizzard_UnitPopup integrations."
    );

    let toc_text = std::fs::read_to_string(raf_toc()).expect("RAF TOC reads");
    assert!(
        toc_text.contains("## Title: Blizzard_RecruitAFriend"),
        "Blizzard_RecruitAFriend declares the underscored `## Title: Blizzard_RecruitAFriend` \
         form (matches the directory name) — distinct from the spaced `## Title: Blizzard \
         Recent Allies` / `## Title: Blizzard Raid UI` forms used by sibling social-panel \
         addons"
    );
    assert!(
        toc_text.contains("## AllowLoad: game"),
        "Blizzard_RecruitAFriend declares `## AllowLoad: game` (lowercase) — RAF is \
         in-world only (no glue-screen access), distinct from sibling \
         Blizzard_FriendsFrame's `## AllowLoad: both`"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_RecruitAFriend declares `## DefaultState: enabled` — the panel is on \
         by default"
    );
    assert!(
        toc_text.contains("## Author: Blizzard Entertainment"),
        "Blizzard_RecruitAFriend declares the canonical `## Author: Blizzard Entertainment` \
         attribution — distinct from Blizzard_RecentAllies / Blizzard_RaidUI which omit \
         the field entirely"
    );
}

#[test]
fn blizzard_recruit_a_friend_toc_carries_rare_suppress_local_table_ref_directive() {
    let toc = TocFile::from_file(&raf_toc()).expect("RAF TOC parse");

    let suppress = toc
        .metadata
        .get("SuppressLocalTableRef")
        .map(String::as_str);
    assert_eq!(
        suppress,
        Some("1"),
        "Blizzard_RecruitAFriend declares `## SuppressLocalTableRef: 1` — a RARE \
         directive that opts out of the per-addon local-table-ref injection (the \
         `local _addonName, _addonTable = ...` parameters that Blizzard's loader \
         normally passes as varargs to each top-level chunk). Without this directive \
         the Lua chunks would receive `(addonName, addonTable)` as `(...)` varargs; \
         with it they receive nothing, which is what RAF wants because every mixin is \
         declared at the top level via `*Mixin = {{}}` rather than namespaced inside \
         a private addon table. Got metadata SuppressLocalTableRef = {:?}",
        suppress
    );
}

#[test]
fn blizzard_recruit_a_friend_toc_lists_frame_and_social_view_sources() {
    let toc = TocFile::from_file(&raf_toc()).expect("RAF TOC parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(
        listed,
        vec![
            "RecruitAFriendFrame.xml".to_string(),
            "RecruitAFriendSocialView.lua".to_string(),
            "RecruitAFriendSocialView.xml".to_string(),
        ],
        "Blizzard_RecruitAFriend lists the core frame XML plus the current Social View Lua/XML \
         pair. RecruitAFriendFrame.lua remains pulled by its XML `<Script>` directive. Got: {:?}",
        listed
    );

    let xml_path = blizzard_ui_dir().join("Blizzard_RecruitAFriend/RecruitAFriendFrame.xml");
    let xml = std::fs::read_to_string(&xml_path).expect("RAF XML reads");
    assert!(
        xml.contains("<Script file=\"RecruitAFriendFrame.lua\"/>"),
        "RecruitAFriendFrame.xml line 3 must carry `<Script file=\"RecruitAFriendFrame.lua\"/>` \
         — this is what pulls in the .lua companion when XML parsing reaches the directive"
    );
}

#[test]
fn blizzard_recruit_a_friend_only_allows_game_screen_via_allow_load_game() {
    let toc = TocFile::from_file(&raf_toc()).expect("RAF TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` MUST allow the Game screen (src/toc.rs:307)"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: game` MUST reject the Login screen — RAF is in-world only"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: game` MUST reject the CharacterSelect screen"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterCreate),
        "`## AllowLoad: game` MUST reject the CharacterCreate screen"
    );
}

#[test]
fn blizzard_recruit_a_friend_appears_in_eager_game_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RecruitAFriend");
    assert!(
        in_game,
        "Blizzard_RecruitAFriend has no `## LoadOnDemand` line and `## AllowLoad: game`, \
         so the eager-discovery filter at src/loader/mod.rs:527 MUST keep it in the \
         Game-screen inventory — every loader-filter gate (load_on_demand=false / \
         ptr_only=false / game_type_restricted=false / allows_screen(Game)=true) admits it"
    );
}

#[test]
fn blizzard_recruit_a_friend_loads_with_profile_is_enabled_surface() {
    let ui = blizzard_ui_dir();
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];

    let mut recruit_a_friend_warnings = None;
    for (name, toc_path) in discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_RecruitAFriend"],
    ) {
        let result = load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("{name} should load: {error}"));
        if name == "Blizzard_RecruitAFriend" {
            recruit_a_friend_warnings = Some(result.warnings);
        }
    }

    let probe: String = env
        .eval(
            "if C_RecruitAFriend.IsEnabled == nil then return 'absent' end \
             return tostring(C_RecruitAFriend.IsEnabled())",
        )
        .expect("Recruit-A-Friend enabled probe should run");
    // 12.1.0 removed C_RecruitAFriend.IsEnabled (12.1.0 API changes, Global API
    // removals) and the cached Blizzard_RecruitAFriend no longer calls it.
    let expected = if cfg!(feature = "retail-12-1-0") {
        "absent"
    } else {
        "false"
    };
    assert_eq!(probe, expected, "Recruit-A-Friend is modeled disabled by default");

    let warnings = recruit_a_friend_warnings.expect("Recruit-A-Friend addon should load");
    assert!(
        !warnings
            .iter()
            .any(|warning| warning.contains("C_RecruitAFriend.IsEnabled")),
        "real Blizzard_RecruitAFriend load must not reference IsEnabled: {warnings:?}"
    );
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_loads_without_errors_during_full_game_startup(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("RecruitAFriend")
                || message.contains("RecruitActivity")
                || message.contains("RecruitList")
                || message.contains("RAF")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_RecruitAFriend emitted Lua errors during full Game-screen startup:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RecruitAFriend') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_RecruitAFriend') must \
         return true — eager auto-discovery picks up the addon (no LoadOnDemand) and \
         `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_publishes_nineteen_mixin_tables(env: &WowLuaEnv) {

    for mixin in MIXIN_TABLES {
        let exists: bool = env
            .eval(&format!("return type({mixin}) == 'table'"))
            .unwrap_or_else(|err| panic!("Mixin probe `{mixin}` failed: {err}"));
        assert!(
            exists,
            "After Blizzard_RecruitAFriend loads, the global `{mixin}` must be a table. \
             RecruitAFriendFrame.lua publishes 19 mixin tables: a base \
             RecruitAFriendSystemMixin (provides GetRecruitAFriendFrame / \
             GetRecruitAFriendRewardsFrame helpers shared by every other system mixin), \
             RecruitAFriendFrameMixin = CreateFromMixins(CallbackRegistryMixin) (the \
             FriendsFrame tab — drives the RecruitList ScrollBox + reward tabs + system \
             enable/disable handling), then 5 mixins composed from RecruitAFriendSystemMixin \
             (NextRewardInfoButton / VersionInfoButton / ClaimRewardButtonBase / \
             RewardsFrame / RewardTab), 4 standalone behavior mixins (RecruitActivityButton \
             / RecruitActivityButtonModel / RecruitListButton / RecruitAFriendReward), 2 \
             reward-button variants composed from RecruitAFriendRewardButtonMixin (WithCheck \
             — for already-claimed checkmark overlay; WithFanfare — for the fanfare \
             reveal animation on first-time claim), and 4 dialog mixins \
             (ClaimLegacyRewardsButton / ClaimOrViewRewardButton / RecruitmentButton / \
             RecruitmentFrame / GenerateOrCopyLinkButton). The all-mixin pattern is \
             distinct from sibling Blizzard_RaidUI's pre-mixin all-free-functions \
             dispatch — RAF.lua has ZERO top-level free functions, only `:method` \
             definitions plus 3 file-local helpers"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_publishes_three_named_top_level_frames(env: &WowLuaEnv) {

    for frame in NAMED_TOP_LEVEL_FRAMES {
        let exists: bool = env
            .eval(&format!("return type({frame}) == 'table'"))
            .unwrap_or_else(|err| panic!("Top-level frame probe `{frame}` failed: {err}"));
        assert!(
            exists,
            "After Blizzard_RecruitAFriend loads, the global `{frame}` must be a table. \
             RecruitAFriendFrame.xml declares three named non-virtual top-level frames: \
             RecruitAFriendRewardsFrame (xml:54 — parent=UIParent, hidden, DIALOG \
             strata, ResizeLayoutFrame, RecruitAFriendRewardsFrameMixin — the popup that \
             shows the cosmetic reward grid when the user clicks View Rewards), \
             RecruitAFriendRecruitmentFrame (xml:245 — parent=UIParent, hidden, DIALOG \
             strata, RecruitAFriendRecruitmentFrameMixin — the BattleNet recruit-link \
             generate/copy popup driven via StaticPopupSpecial_Show/Hide), and \
             RecruitAFriendFrame (xml:466 — parent=FriendsFrame, hidden, setAllPoints, \
             RecruitAFriendFrameMixin + CallbackRegistrantTemplate — the actual \
             FriendsFrame tab content)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for tmpl in VIRTUAL_TEMPLATES {
        let leaked: bool = env
            .eval(&format!("return _G[{tmpl:?}] ~= nil"))
            .unwrap_or_else(|err| panic!("Template probe `{tmpl}` failed: {err}"));
        assert!(
            !leaked,
            "Virtual template `{tmpl}` must NOT appear as a global. The 9 virtual \
             templates declared in RecruitAFriendFrame.xml (RAFInfoButtonTemplate, \
             RAFClaimRewardButtonBaseTemplate, RecruitAFriendRewardTabTemplate, \
             RecruitAFriendRewardButtonTemplate, RecruitAFriendRewardTemplate, \
             RecruitTextTemplate, RecruitSmallTextTemplate, RecruitActivityButtonTemplate, \
             RecruitListButtonTemplate) are XML-only fixtures consumed via inherits=\"…\" \
             / ScrollUtil factories — they should never CreateFrame-instantiate by name \
             and should never leak into _G"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_frame_is_in_friends_frame_subframes_table(env: &WowLuaEnv) {

    let in_subframes: bool = env
        .eval(
            "for _, name in ipairs(FRIENDSFRAME_SUBFRAMES or {}) do \
                 if name == 'RecruitAFriendFrame' then return true end \
             end \
             return false",
        )
        .expect("FRIENDSFRAME_SUBFRAMES probe should succeed");
    assert!(
        in_subframes,
        "RecruitAFriendFrame must be enrolled in FRIENDSFRAME_SUBFRAMES — \
         FriendsFrame.lua line 63 declares the array of frame-name strings that \
         FriendsFrame_ShowSubFrame iterates to mutually exclude. \
         RecruitAFriendFrame is named in that list (entry 5 in the standard 6-entry \
         array) so the social-panel tab strip can show/hide it via name lookup. RAF now \
         lists Blizzard_FriendsFrame as an optional social integration; eager game-screen \
         discovery loads FriendsFrame before RAF's XML parses."
    );
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_frame_mixin_publishes_current_callback_event_enum(env: &WowLuaEnv) {

    let events: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(RecruitAFriendFrameMixin.Event) == 'table', \
                    RecruitAFriendFrameMixin.Event.NewRewardTabSelected ~= nil, \
                    RecruitAFriendFrameMixin.Event.RewardsListOpened ~= nil, \
                    RecruitAFriendFrameMixin.Event.RewardsListClosed ~= nil, \
                    RecruitAFriendFrameMixin.Event.SelectedRAFVersionChanged == nil",
        )
        .expect("Callback event enum probe should succeed");
    assert_eq!(
        events,
        (true, true, true, true, true),
        "Current RecruitAFriendFrameMixin GenerateCallbackEvents publishes only \
         NewRewardTabSelected, RewardsListOpened, and RewardsListClosed. \
         SelectedRAFVersionChanged is absent from the retail 12.1.0.69497 source."
    );
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_frame_mixin_exposes_lifecycle_and_state_methods(env: &WowLuaEnv) {

    let lifecycle: (bool, bool, bool) = env
        .eval(
            "return type(RecruitAFriendFrameMixin.OnLoad) == 'function', \
                    type(RecruitAFriendFrameMixin.OnHide) == 'function', \
                    type(RecruitAFriendFrameMixin.OnEvent) == 'function'",
        )
        .expect("Lifecycle probe should succeed");
    assert_eq!(
        lifecycle,
        (true, true, true),
        "RecruitAFriendFrameMixin publishes the lifecycle trio: OnLoad (registers 6 \
         events + initializes the RecruitList ScrollBox via \
         CreateScrollBoxListLinearView with SetElementExtentCalculator branching on \
         elementData.isDivider for DIVIDER_HEIGHT=16 vs RECRUIT_HEIGHT=34, then calls \
         CallbackRegistryMixin.OnLoad and AddDynamicEventMethod for the 4 callback \
         events), OnHide (closes the rewards frame and the recruitment frame popup), \
         OnEvent (dispatches the 6 registered events to UpdateRAFInfo / \
         UpdateRecruitList / SetRAFSystemEnabled / etc.)"
    );

    let state_helpers: (bool, bool, bool, bool) = env
        .eval(
            "return type(RecruitAFriendFrameMixin.SetRAFSystemEnabled) == 'function', \
                    type(RecruitAFriendFrameMixin.SetRAFRecruitingEnabled) == 'function', \
                    type(RecruitAFriendFrameMixin.UpdateRAFSystemInfo) == 'function', \
                    type(RecruitAFriendFrameMixin.UpdateRAFInfo) == 'function'",
        )
        .expect("State-helper probe should succeed");
    assert_eq!(
        state_helpers,
        (true, true, true, true),
        "RecruitAFriendFrameMixin's state pipeline: SetRAFSystemEnabled / \
         SetRAFRecruitingEnabled toggle the two server-driven enable flags; \
         UpdateRAFSystemInfo refreshes the reward-tier metadata; UpdateRAFInfo \
         rebuilds the recruit list and current-month progress display from the \
         C_RecruitAFriend.GetRAFInfo struct"
    );
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_row_height_constants_stay_file_local(env: &WowLuaEnv) {

    let leaked: (bool, bool) = env
        .eval("return RECRUIT_HEIGHT ~= nil, DIVIDER_HEIGHT ~= nil")
        .expect("File-local constant probe should succeed");
    assert_eq!(
        leaked,
        (false, false),
        "RecruitAFriendFrame.lua lines 1-2 declare RECRUIT_HEIGHT=34 and DIVIDER_HEIGHT=16 \
         as `local` constants — they MUST NOT leak into _G. These drive the \
         RecruitList ScrollBox view's SetElementExtentCalculator branching \
         (elementData.isDivider chooses DIVIDER_HEIGHT for separator rows, \
         RECRUIT_HEIGHT for actual recruit rows). Keeping them file-local is the \
         conventional Blizzard pattern for layout magic numbers — and the \
         `## SuppressLocalTableRef: 1` directive in the TOC reinforces this by \
         opting out of the per-addon `local _, _addonTable = ...` injection so the \
         module file isn't tempted to namespace these constants under an addon table"
    );
}
}

prefork_full_ui_case! {
fn blizzard_recruit_a_friend_frame_registers_six_events_in_onload(env: &WowLuaEnv) {

    for event in REGISTERED_EVENTS {
        let registered: bool = env
            .eval(&format!(
                "return RecruitAFriendFrame:IsEventRegistered({event:?})"
            ))
            .unwrap_or_else(|err| panic!("Event probe `{event}` failed: {err}"));
        assert!(
            registered,
            "RecruitAFriendFrame:OnLoad (lua:24-58) registers `{event}` directly via \
             self:RegisterEvent. The 6 events split into: 2 system-state events \
             (RAF_SYSTEM_ENABLED_STATUS — toggles the master enable flag; \
             RAF_RECRUITING_ENABLED_STATUS — toggles whether new recruitment links \
             can be generated, distinct from the master toggle), 2 data-refresh events \
             (RAF_SYSTEM_INFO_UPDATED — reward-tier metadata refresh; RAF_INFO_UPDATED \
             — recruit list / progress refresh), 1 cross-system event \
             (BN_FRIEND_INFO_CHANGED — driven by Blizzard_BNet when a recruit's \
             BattleNet status changes, triggers UpdateRecruitList), and 1 startup \
             gate (VARIABLES_LOADED — sets self.varsLoaded=true to unblock \
             UpdateRAFTutorialTips which won't show tips until SavedVariables are \
             ready)"
        );
    }
}
}
