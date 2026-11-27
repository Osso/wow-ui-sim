//! Behavior pin: passing `toggleGuildView=true` to
//! `AchievementFrame_ToggleAchievementFrame` routes through the
//! `GuildCategoryIndex=2` branch, which selects
//! `GUILD_ACHIEVEMENT_FUNCTIONS` and makes `InGuildView()` return true,
//! and downstream `AchievementFrame_RefreshView` and
//! `AchievementFrameSummaryCategoriesStatusBar_Update` forward that
//! boolean to `GetTotalAchievementPoints` and
//! `GetNumCompletedAchievements`. The reverse drive
//! (`toggleGuildView=false`) routes through `AchievementCategoryIndex=1`
//! and forwards `false`.
//!
//! Two distinct edges are pinned across two tests so each body stays
//! comfortably under the readability budget.
//!
//! 1. **Tab dispatch edge** — see
//!    `toggle_dispatches_base_tab_onclick_with_index_2_for_guild_and_index_1_for_player`.
//!    `AchievementFrame_ToggleAchievementFrame` (lua:195-223) reassigns
//!    `AchievementFrameTab_OnClick = AchievementFrameBaseTab_OnClick` at
//!    lua:201 and then dispatches `AchievementFrameTab_OnClick(2)` for
//!    guild (lua:209) or `AchievementFrameTab_OnClick(1)` for player
//!    (lua:211). The reassignment at lua:201 is INSIDE the toggle, not
//!    at module scope, so the test must spy
//!    `_G.AchievementFrameBaseTab_OnClick` (the source of the binding)
//!    rather than `_G.AchievementFrameTab_OnClick` (the destination,
//!    which gets overwritten at every toggle call).
//!
//! 2. **Boolean forwarding edge** — see
//!    `refresh_view_and_summary_status_bar_propagate_in_guild_view_to_total_points_and_num_completed_globals`.
//!    `AchievementFrame_RefreshView` (lua:347-379) reads
//!    `GetTotalAchievementPoints(InGuildView())` at lua:378.
//!    `AchievementFrameSummaryCategoriesStatusBar_Update` (lua:2471-2476)
//!    reads `GetNumCompletedAchievements(InGuildView())` at lua:2472.
//!    `InGuildView()` is the local at lua:103-105 that returns
//!    `achievementFunctions == GUILD_ACHIEVEMENT_FUNCTIONS`. The locals
//!    `AchievementCategoryIndex=1`, `GuildCategoryIndex=2`,
//!    `StatisticsCategoryIndex=3` (lua:78-80), `InGuildView` itself,
//!    AND the `achievementFunctions` upvalue (declared at lua:70 as
//!    `local achievementFunctions;`) are all not exported. To toggle
//!    `InGuildView()`'s return value the test uses `debug.setupvalue`
//!    on `_G.AchievementFrameBaseTab_OnClick` (a global closure that
//!    closes over the SAME `achievementFunctions` upvalue cell — Lua
//!    5.1 shares one cell across all closures that capture the same
//!    outer local). Mutating the cell via `debug.setupvalue` is
//!    visible to `InGuildView()` and to the points-header read at
//!    lua:378 / status-bar read at lua:2472.
//!
//! **PLAN-named tripwire.** PLAN refers to "GetNumCompletedAchievements"
//! and "GetTotalAchievementPoints" gaps via a stale `depends-on` tag.
//! Both globals are already wired by `register_summary_globals` at
//! `src/lua_api/globals/missing_surface/achievement_info.rs:367-389`.
//! A `nil` reading on either would mean the registration regressed —
//! the simulator's default `_OnShow` handler at lua:272 would crash on
//! the first show.
//!
//! **No `set_tabs_called` / `show_panel_called` budget.** The toggle
//! also calls `AchievementFrame_SetTabs()` and `ShowUIPanel(...)` in
//! the player-tab dispatch at lua:206-207. The test no-ops both so
//! that the panel doesn't actually transition to "shown" — keeping
//! `AchievementFrame:IsShown()` false on the second drive ensures the
//! second toggle goes through the same else branch (lua:205-213) and
//! NOT the early-return-on-already-shown branch at lua:203-204.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const PLAN_DEPENDS_ON_NUM_COMPLETED: &str = "GetNumCompletedAchievements";
const PLAN_DEPENDS_ON_TOTAL_POINTS: &str = "GetTotalAchievementPoints";
#[cfg(not(feature = "client-wowforever"))]
const EXPECTED_TAB_INDICES: &str = "tab_calls=2 guild_index=2 player_index=1";
const EXPECTED_TOTAL_POINTS: &str =
    "total_calls=2 first_arg=true second_arg=false guild_view_after_first=true";
const EXPECTED_NUM_COMPLETED: &str =
    "num_calls=2 first_arg=true second_arg=false guild_view_after_first=true";

#[cfg(not(feature = "client-wowforever"))]
type TabDispatchProbe = (String, String, String);
type ForwardingProbe = (String, String, String, String);

post_wrath_ui_fixture! {
// Forever 1.60.1.70707 replaces this toggle: Camelot/Blizzard_AchievementUI.lua:13-17
// routes AchievementFrame_ToggleAchievementFrame to ToggleLegacyChallenges, so
// the Mainline AchievementFrame toggle contract does not apply there.
#[cfg(not(feature = "client-wowforever"))]
#[test]
fn toggle_dispatches_base_tab_onclick_with_index_2_for_guild_and_index_1_for_player() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: TabDispatchProbe = env
            .eval(
                r#"
                assert(AchievementFrame, "AchievementFrame must exist after addon load")
                assert(_G.AchievementFrame_ToggleAchievementFrame,
                    "AchievementFrame_ToggleAchievementFrame must exist (lua:195)")
                assert(_G.AchievementFrameBaseTab_OnClick,
                    "AchievementFrameBaseTab_OnClick must exist (lua:406)")

                local tab_calls = {indices = {}}
                local original_base_tab = _G.AchievementFrameBaseTab_OnClick
                _G.AchievementFrameBaseTab_OnClick = function(tab_index)
                    tab_calls.indices[#tab_calls.indices + 1] = tab_index
                end

                local original_set_tabs = _G.AchievementFrame_SetTabs
                _G.AchievementFrame_SetTabs = function() end
                local original_show = _G.ShowUIPanel
                _G.ShowUIPanel = function(self) end
                local original_hide = _G.HideUIPanel
                _G.HideUIPanel = function(self) end

                local original_is_active = _G.C_GameRules.IsGameRuleActive
                _G.C_GameRules.IsGameRuleActive = function() return false end

                AchievementFrame_ToggleAchievementFrame(nil, true)
                AchievementFrame_ToggleAchievementFrame(nil, false)

                local tab_signature = string.format(
                    "tab_calls=%d guild_index=%s player_index=%s",
                    #tab_calls.indices,
                    tostring(tab_calls.indices[1]),
                    tostring(tab_calls.indices[2]))

                _G.AchievementFrameBaseTab_OnClick = original_base_tab
                _G.AchievementFrame_SetTabs = original_set_tabs
                _G.ShowUIPanel = original_show
                _G.HideUIPanel = original_hide
                _G.C_GameRules.IsGameRuleActive = original_is_active

                return type(_G.AchievementFrame_ToggleAchievementFrame),
                       type(_G.AchievementFrameBaseTab_OnClick),
                       tab_signature
                "#,
            )
            .expect("setup + double-drive toggle must run cleanly");

        let (toggle_type, base_tab_type, tab_signature) = observations;

        assert_eq!(
            toggle_type, "function",
            "Expected `_G.AchievementFrame_ToggleAchievementFrame` to be a function (declared at \
             `Mainline/Blizzard_AchievementUI.lua:195`). Got `{toggle_type}`. A `nil` reading \
             means the panel-toggle entry point used by chat-link clicks, micro-button presses, \
             and `/run AchievementFrame_ToggleAchievementFrame()` would all silently no-op — \
             pressing the achievements micro-button would do nothing."
        );

        assert_eq!(
            base_tab_type, "function",
            "Expected `_G.AchievementFrameBaseTab_OnClick` to be a function (declared at \
             lua:406). Got `{base_tab_type}`. A `nil` reading means lua:201 \
             (`AchievementFrameTab_OnClick = AchievementFrameBaseTab_OnClick`) would copy nil \
             into `AchievementFrameTab_OnClick`, and the very next call at lua:209/lua:211 \
             would crash with `attempt to call a nil value`."
        );

        assert_eq!(
            tab_signature, EXPECTED_TAB_INDICES,
            "Expected `Toggle(nil, true)` followed by `Toggle(nil, false)` to dispatch \
             `BaseTab_OnClick(2)` then `BaseTab_OnClick(1)`, producing signature \
             `{EXPECTED_TAB_INDICES}`. Got `{tab_signature}`. A `tab_calls<2` reading means one \
             of the toggles short-circuited — likely `AchievementFrame:IsShown()` returned true \
             on the second drive (so `Toggle(nil, false)` hit the `selectedTab == 1` early \
             hide-branch at lua:203-204 instead of the dispatch branch at lua:208-212), or the \
             `IsGameRuleActive(AchievementsPanelDisabled)` early return at lua:197-199 fired. \
             A `guild_index` other than 2 means the `toggleGuildView` branch at lua:208-209 \
             was inverted or read the wrong constant — `GuildCategoryIndex` is a LOCAL at \
             lua:79 set to 2. A `player_index` other than 1 means the else branch at \
             lua:210-212 dispatched the wrong index — `AchievementCategoryIndex` is a LOCAL \
             at lua:78 set to 1."
        );
    });
}
}

post_wrath_ui_fixture! {
#[test]
fn refresh_view_and_summary_status_bar_propagate_in_guild_view_to_total_points_and_num_completed_globals()
 {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: ForwardingProbe = env
            .eval(
                r#"
                assert(_G.AchievementFrame_RefreshView,
                    "AchievementFrame_RefreshView must exist (lua:347)")
                assert(_G.AchievementFrameSummaryCategoriesStatusBar_Update,
                    "AchievementFrameSummaryCategoriesStatusBar_Update must exist (lua:2471)")
                assert(_G.GUILD_ACHIEVEMENT_FUNCTIONS,
                    "GUILD_ACHIEVEMENT_FUNCTIONS must exist (lua:149)")
                assert(_G.ACHIEVEMENT_FUNCTIONS,
                    "ACHIEVEMENT_FUNCTIONS must exist (lua:144)")

                local total_calls = {args = {}}
                local original_total = _G.GetTotalAchievementPoints
                _G.GetTotalAchievementPoints = function(in_guild)
                    total_calls.args[#total_calls.args + 1] = in_guild
                    return 0
                end

                local num_calls = {args = {}}
                local original_num = _G.GetNumCompletedAchievements
                _G.GetNumCompletedAchievements = function(in_guild)
                    num_calls.args[#num_calls.args + 1] = in_guild
                    return 100, 50
                end

                local original_get_logo = _G.GetGuildLogoInfo
                _G.GetGuildLogoInfo = function() return nil end
                local original_break = _G.BreakUpLargeNumbers
                _G.BreakUpLargeNumbers = function(n) return tostring(n) end

                local function find_upvalue_index(fn, name)
                    local i = 1
                    while true do
                        local up_name = debug.getupvalue(fn, i)
                        if not up_name then return nil end
                        if up_name == name then return i end
                        i = i + 1
                    end
                end
                local upvalue_idx = find_upvalue_index(
                    _G.AchievementFrameBaseTab_OnClick, "achievementFunctions")
                assert(upvalue_idx,
                    "expected `achievementFunctions` to be an upvalue of " ..
                    "`AchievementFrameBaseTab_OnClick` (closes over the local at lua:70)")
                local original_funcs = select(2,
                    debug.getupvalue(_G.AchievementFrameBaseTab_OnClick, upvalue_idx))

                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, _G.GUILD_ACHIEVEMENT_FUNCTIONS)
                AchievementFrame_RefreshView()
                AchievementFrameSummaryCategoriesStatusBar_Update()
                local probed_funcs = select(2,
                    debug.getupvalue(_G.AchievementFrameBaseTab_OnClick, upvalue_idx))
                local guild_view_after_first =
                    probed_funcs == _G.GUILD_ACHIEVEMENT_FUNCTIONS

                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, _G.ACHIEVEMENT_FUNCTIONS)
                AchievementFrame_RefreshView()
                AchievementFrameSummaryCategoriesStatusBar_Update()

                local total_signature = string.format(
                    "total_calls=%d first_arg=%s second_arg=%s guild_view_after_first=%s",
                    #total_calls.args,
                    tostring(total_calls.args[1]),
                    tostring(total_calls.args[2]),
                    tostring(guild_view_after_first))
                local num_signature = string.format(
                    "num_calls=%d first_arg=%s second_arg=%s guild_view_after_first=%s",
                    #num_calls.args,
                    tostring(num_calls.args[1]),
                    tostring(num_calls.args[2]),
                    tostring(guild_view_after_first))

                _G.GetTotalAchievementPoints = original_total
                _G.GetNumCompletedAchievements = original_num
                _G.GetGuildLogoInfo = original_get_logo
                _G.BreakUpLargeNumbers = original_break
                debug.setupvalue(_G.AchievementFrameBaseTab_OnClick,
                    upvalue_idx, original_funcs)

                return type(_G.GetTotalAchievementPoints),
                       type(_G.GetNumCompletedAchievements),
                       total_signature,
                       num_signature
                "#,
            )
            .expect("setup + guild-then-player drive must run cleanly");

        let (total_points_type, num_completed_type, total_signature, num_signature) = observations;

        assert_eq!(
            total_points_type, "function",
            "Expected `_G.{PLAN_DEPENDS_ON_TOTAL_POINTS}` to be a function (registered at \
             `register_summary_globals` in \
             `src/lua_api/globals/missing_surface/achievement_info.rs:367-389`). Got \
             `{total_points_type}`. PLAN's `depends-on: GetTotalAchievementPoints gap` is \
             stale — the global was wired before this task; a `nil` reading would mean the \
             registration regressed and `AchievementFrame_OnShow` at lua:272 would crash \
             trying to format the points header on first open."
        );

        assert_eq!(
            num_completed_type, "function",
            "Expected `_G.{PLAN_DEPENDS_ON_NUM_COMPLETED}` to be a function (registered at \
             `register_summary_globals` in `achievement_info.rs:367-389`). Got \
             `{num_completed_type}`. PLAN's `depends-on: GetNumCompletedAchievements gap` is \
             stale — a `nil` reading would mean the status bar update at lua:2472 would crash \
             on every summary view refresh."
        );

        assert_eq!(
            total_signature, EXPECTED_TOTAL_POINTS,
            "Expected `_RefreshView` driven once with `achievementFunctions = \
             GUILD_ACHIEVEMENT_FUNCTIONS` then once with `ACHIEVEMENT_FUNCTIONS` to forward \
             `true` then `false` to `GetTotalAchievementPoints`, producing signature \
             `{EXPECTED_TOTAL_POINTS}`. Got `{total_signature}`. A `first_arg=false` reading \
             with guild functions installed means `InGuildView()` at lua:103-105 (`return \
             achievementFunctions == GUILD_ACHIEVEMENT_FUNCTIONS`) is comparing against the \
             wrong sentinel — likely the upvalue capture saw a stale `GUILD_ACHIEVEMENT_FUNCTIONS` \
             from before lua:149's assignment ran. A `second_arg=true` reading with player \
             functions installed means the comparison logic is inverted. A `total_calls<2` \
             reading means `_RefreshView` short-circuited before reaching lua:378 — likely \
             `GetGuildLogoInfo()` or one of the `AchievementFrameGuildEmblem*` widget calls \
             at lua:358-365 raised; the guild-emblem path runs before the points-header write \
             at lua:378."
        );

        assert_eq!(
            num_signature, EXPECTED_NUM_COMPLETED,
            "Expected `_SummaryCategoriesStatusBar_Update` driven once with guild functions \
             then once with player functions to forward `true` then `false` to \
             `GetNumCompletedAchievements`, producing signature `{EXPECTED_NUM_COMPLETED}`. \
             Got `{num_signature}`. A `first_arg=false` reading with guild functions installed \
             means the `local total, completed = GetNumCompletedAchievements(InGuildView())` \
             at lua:2472 dropped the boolean before forwarding — likely `InGuildView` was \
             inlined or replaced with a literal. A `num_calls<2` reading means the status bar \
             function short-circuited before reaching the `GetNumCompletedAchievements` call \
             — likely `AchievementFrameSummaryCategoriesStatusBar` (the StatusBar widget) \
             does not exist at the global lookup, breaking the `:SetMinMaxValues` write at \
             lua:2473 before the count is read."
        );
    });
}
}
