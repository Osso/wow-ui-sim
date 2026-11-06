use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn weekly_rewards_util_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_WeeklyRewardsUtil")
}

fn weekly_rewards_util_toc() -> PathBuf {
    weekly_rewards_util_dir().join("Blizzard_WeeklyRewardsUtil.toc")
}

const UTIL_FUNCTIONS: &[&str] = &[
    "GetNextMythicLevel",
    "GetLowestLevelInTopDungeonRuns",
    "GetActivitiesProgress",
    "HasUnlockedRewards",
    "GetMaxNumRewards",
    "GetNumUnlockedRewards",
];

const MIXIN_METHODS: &[&str] = &[
    "OnMouseUp",
    "HasUnlockedRewards",
    "GetMaxNumRewards",
    "GetNumUnlockedRewards",
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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_variant() {
    let resolved = find_toc_file(&weekly_rewards_util_dir())
        .expect("Blizzard_WeeklyRewardsUtil TOC should resolve");
    assert_eq!(
        resolved,
        weekly_rewards_util_toc(),
        "Blizzard_WeeklyRewardsUtil ships exactly one bare TOC — find_toc_file probes the \
         `_Mainline.toc` variant first (miss) and falls through to the bare TOC name (hit). \
         No flavor companion exists; the util is shared across every retail flavor that ships \
         the Great Vault feature"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_declares_explicit_lod_zero() {
    let toc = TocFile::from_file(&weekly_rewards_util_toc())
        .expect("Blizzard_WeeklyRewardsUtil TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_WeeklyRewardsUtil declares `## LoadOnDemand: 0` — explicitly opts OUT of LoD \
         even though directive omission would have the same effect (toc.rs:259-264 returns \
         false for `\"0\"` and for missing key alike). The explicit `0` is a deliberate \
         convention used by Blizzard utility / shared-foundation addons (every \
         Blizzard_Deprecated* addon, Blizzard_ChatFrameUtil, Blizzard_ClientSavedVariables, \
         Blizzard_CovenantToasts, etc. all use it) to signal: \"this addon is intentionally \
         eager because other addons depend on its globals at file scope, and listing the \
         directive prevents accidental flips to LoD during Blizzard's internal refactors\""
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_WeeklyRewardsUtil declares no dependencies — the lua file consumes only the \
         C_WeeklyRewards C API + C_MythicPlus.GetRunHistory + Enum.WeeklyRewardChestThresholdType \
         + table.sort + math.min + the global WeeklyRewards_ShowUI helper (called from \
         WeeklyRewardMixin:OnMouseUp on left-click). All of those are provided by the \
         eagerly-loaded core (FrameXML stdlib, the C API surface, and \
         Blizzard_UIParent/Mainline/UIParent.lua's WeeklyRewards_ShowUI declaration); none \
         require an explicit dep"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_omits_allow_load_directives() {
    let toc = TocFile::from_file(&weekly_rewards_util_toc())
        .expect("Blizzard_WeeklyRewardsUtil TOC should parse");

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
            "Without `## AllowLoad`, allows_screen rejects every glue screen ({screen:?}) — the \
             Great Vault is in-world UI only; util loads with the rest of the Game-screen UI"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "Without `## AllowLoadGameType`, is_game_type_restricted returns false (unrestricted \
         gametype)"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_lists_only_lua_body_file() {
    let toc = TocFile::from_file(&weekly_rewards_util_toc())
        .expect("Blizzard_WeeklyRewardsUtil TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec!["Blizzard_WeeklyRewardsUtil.lua".to_string()],
        "TOC body lists ONLY the lua file — no XML, no localization file, no flavor subdirectory. \
         The addon is pure Lua: a single namespace table (WeeklyRewardsUtil) + a single mixin \
         (WeeklyRewardMixin) + 6 namespace methods + 4 mixin methods + 2 sentinel constants. \
         Inverse of Blizzard_WeeklyRewards (which lists ONLY xml and loads lua via \
         `<Script file>`); this addon lists ONLY lua and has no XML at all"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_raw_bytes_pin_minimal_directives() {
    let raw = std::fs::read_to_string(weekly_rewards_util_toc())
        .expect("Blizzard_WeeklyRewardsUtil TOC should read");

    for directive in [
        "## Title: Blizzard Weekly Rewards Util",
        "## LoadOnDemand: 0",
    ] {
        assert!(
            raw.contains(directive),
            "TOC must contain directive line `{directive}`"
        );
    }

    assert!(
        raw.contains("Blizzard_WeeklyRewardsUtil.lua"),
        "TOC must contain body file line `Blizzard_WeeklyRewardsUtil.lua`"
    );

    for absent_directive in [
        "## Author:",
        "## Version:",
        "## Notes:",
        "## Dependencies:",
        "## RequiredDep:",
        "## RequiredDeps:",
        "## OptionalDeps:",
        "## SavedVariables:",
        "## AllowLoad:",
        "## AllowLoadGameType:",
        "## DefaultState:",
        "## LoadFirst:",
        "## LoadWith:",
    ] {
        assert!(
            !raw.contains(absent_directive),
            "TOC must NOT contain `{absent_directive}` — Blizzard_WeeklyRewardsUtil is the \
             second-most-minimal TOC analyzed in this campaign (after Blizzard_WeeklyRewards): \
             just Title + LoadOnDemand:0 + 1 lua body line"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn directory_holds_two_entries() {
    let entries = std::fs::read_dir(weekly_rewards_util_dir())
        .expect("Blizzard_WeeklyRewardsUtil directory should read")
        .count();
    assert_eq!(
        entries, 2,
        "Directory must hold exactly 2 entries (1 TOC + 1 lua file). No XML, no \
         flavor subdirectory, no Localization.lua — every UI string the util references comes \
         from the global locale table (none used directly: this addon emits zero strings)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_WeeklyRewardsUtil");
    assert!(
        found,
        "Blizzard_WeeklyRewardsUtil must appear in Game eager discovery — `## LoadOnDemand: 0` \
         keeps it eligible for the eager pass, and the absent `## AllowLoad` directive defaults \
         to Game-only"
    );
}
}

#[test]
fn absent_from_glue_screen_auto_discovery() {
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_WeeklyRewardsUtil");
        assert!(
            !found,
            "Blizzard_WeeklyRewardsUtil must NOT appear in {screen:?} auto-discovery — the \
             addon defaults to Game-only without an `## AllowLoad` directive"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_WeeklyRewardsUtil")
                || message.contains("WeeklyRewardsUtil.")
                || message.contains("WeeklyRewardMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_WeeklyRewardsUtil emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_full_game_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_WeeklyRewardsUtil')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_WeeklyRewardsUtil') must return true after the eager \
         Game-screen pass — confirms the LoadOnDemand:0 + Game-default screen filter combination \
         puts it in the eager bring-up path"
    );
}
}

prefork_full_ui_case! {
fn weekly_rewards_util_table_publishes_with_constants(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(WeeklyRewardsUtil)")
        .expect("WeeklyRewardsUtil probe should succeed");
    assert_eq!(
        kind, "table",
        "WeeklyRewardsUtil must publish at `_G` as a table — declared at lua:2 with two sentinel \
         level constants (HeroicLevel = -1, MythicLevel = 0). Both other constants land at \
         non-Mythic+ levels (regular Heroic = -1 and base Mythic = 0) intentionally chosen to \
         avoid collision with the level field on real C_MythicPlus.GetRunHistory entries (which \
         start at level 2 for the lowest keystone)"
    );

    let heroic_level: i64 = env
        .eval("return WeeklyRewardsUtil.HeroicLevel")
        .expect("HeroicLevel probe should succeed");
    assert_eq!(
        heroic_level, -1,
        "WeeklyRewardsUtil.HeroicLevel must equal -1 — sentinel for Heroic dungeon runs that \
         don't carry a numeric level in the activity data (in real C_WeeklyRewards.GetActivities \
         output, both Heroic and base-Mythic dungeon runs report level=0; the util uses -1 to \
         differentiate Heroic from Mythic in GetLowestLevelInTopDungeonRuns's return tuple)"
    );

    let mythic_level: i64 = env
        .eval("return WeeklyRewardsUtil.MythicLevel")
        .expect("MythicLevel probe should succeed");
    assert_eq!(
        mythic_level, 0,
        "WeeklyRewardsUtil.MythicLevel must equal 0 — sentinel for regular (non-keystone) Mythic \
         dungeons; GetNextMythicLevel(0) bumps to 2 because Mythic+ keystones start at level 2 \
         (no level-1 keystone exists in the live game)"
    );
}
}

prefork_full_ui_case! {
fn weekly_rewards_util_methods_publish_as_functions(env: &WowLuaEnv) {

    for fn_name in UTIL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(WeeklyRewardsUtil.{fn_name})"))
            .unwrap_or_else(|err| panic!("WeeklyRewardsUtil.{fn_name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "WeeklyRewardsUtil.{fn_name} must publish as a function — declared with the \
             `function WeeklyRewardsUtil.{fn_name}(...)` colon-less syntax (these are namespace \
             functions, not method calls — `self` is not threaded through)"
        );
    }
}
}

prefork_full_ui_case! {
fn get_next_mythic_level_jumps_zero_to_two(env: &WowLuaEnv) {

    let from_mythic: i64 = env
        .eval("return WeeklyRewardsUtil.GetNextMythicLevel(WeeklyRewardsUtil.MythicLevel)")
        .expect("GetNextMythicLevel(0) probe should succeed");
    assert_eq!(
        from_mythic, 2,
        "GetNextMythicLevel(MythicLevel=0) must return 2 — the special-case branch at lua:8-10 \
         skips level 1 because the lowest Mythic+ keystone in the live game is level 2; \
         level 1 keystones do not exist"
    );

    let from_two: i64 = env
        .eval("return WeeklyRewardsUtil.GetNextMythicLevel(2)")
        .expect("GetNextMythicLevel(2) probe should succeed");
    assert_eq!(
        from_two, 3,
        "GetNextMythicLevel(2) must return 3 — generic level+1 branch at lua:11"
    );

    let from_fifteen: i64 = env
        .eval("return WeeklyRewardsUtil.GetNextMythicLevel(15)")
        .expect("GetNextMythicLevel(15) probe should succeed");
    assert_eq!(
        from_fifteen, 16,
        "GetNextMythicLevel(15) must return 16 — generic level+1 path applies for every level \
         above MythicLevel=0"
    );
}
}

prefork_full_ui_case! {
fn weekly_reward_mixin_publishes_with_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(WeeklyRewardMixin)")
        .expect("WeeklyRewardMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "WeeklyRewardMixin must publish at `_G` as a table — declared at lua:100 as the chassis \
         that ChallengeModeWeeklyChestMixin (in Blizzard_ChallengesUI/Mainline/Blizzard_ChallengesUI.lua:282) \
         extends via `CreateFromMixins(WeeklyRewardMixin)`. The mixin is the bridge between the \
         Mythic+ challenges UI weekly-chest button and the Great Vault: clicking the chest in \
         the challenges panel opens the vault via OnMouseUp's WeeklyRewards_ShowUI() call"
    );

    for method_name in MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(WeeklyRewardMixin.{method_name})"))
            .unwrap_or_else(|err| panic!("WeeklyRewardMixin.{method_name} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "WeeklyRewardMixin.{method_name} must publish as a function — declared at file scope \
             after the WeeklyRewardMixin = {{}} table on lua:100"
        );
    }
}
}

prefork_full_ui_case! {
fn weekly_reward_mixin_methods_delegate_to_util_functions(env: &WowLuaEnv) {

    let delegates: bool = env
        .eval(
            "local frame = {} \
             setmetatable(frame, {__index = WeeklyRewardMixin}) \
             local util_max = WeeklyRewardsUtil.GetMaxNumRewards(nil) \
             local mixin_max = frame:GetMaxNumRewards(nil) \
             local util_unlocks = WeeklyRewardsUtil.GetNumUnlockedRewards(nil) \
             local mixin_unlocks = frame:GetNumUnlockedRewards(nil) \
             local util_has = WeeklyRewardsUtil.HasUnlockedRewards(nil) \
             local mixin_has = frame:HasUnlockedRewards(nil) \
             return util_max == mixin_max \
                 and util_unlocks == mixin_unlocks \
                 and util_has == mixin_has",
        )
        .expect("WeeklyRewardMixin delegation probe should succeed");
    assert!(
        delegates,
        "WeeklyRewardMixin's HasUnlockedRewards / GetMaxNumRewards / GetNumUnlockedRewards \
         methods must each forward exactly to the corresponding WeeklyRewardsUtil namespace \
         function — the mixin is a thin shim that lets a frame react with `frame:Method()` \
         syntax instead of `WeeklyRewardsUtil.Method()`. The shim layer exists so that the \
         frame in question (ChallengeModeWeeklyChestMixin's chest button in Blizzard_ChallengesUI) \
         can delegate to the util without each call site having to reach into the namespace \
         table directly"
    );
}
}

prefork_full_ui_case! {
fn weekly_reward_mixin_on_mouse_up_calls_show_ui_for_left_click_inside(env: &WowLuaEnv) {

    let triggered: bool = env
        .eval(
            "local original = WeeklyRewards_ShowUI \
             local called = false \
             WeeklyRewards_ShowUI = function() called = true end \
             local frame = {} \
             setmetatable(frame, {__index = WeeklyRewardMixin}) \
             frame:OnMouseUp('LeftButton', true) \
             local left_inside = called \
             called = false \
             frame:OnMouseUp('RightButton', true) \
             local right_inside = called \
             called = false \
             frame:OnMouseUp('LeftButton', false) \
             local left_outside = called \
             WeeklyRewards_ShowUI = original \
             return left_inside and (not right_inside) and (not left_outside)",
        )
        .expect("WeeklyRewardMixin:OnMouseUp probe should succeed");
    assert!(
        triggered,
        "WeeklyRewardMixin:OnMouseUp must call WeeklyRewards_ShowUI() ONLY when (button == \
         'LeftButton' AND upInside == true). RightButton or release-outside-the-button must NOT \
         trigger the vault open — the MMO standard click-to-open contract from the chest button \
         in the Mythic+ challenges panel"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_WeeklyRewardsUtil, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_weeklyrewardsutil_toc_is_absent_from_classic_caches() {
    let dir = weekly_rewards_util_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_WeeklyRewardsUtil"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
