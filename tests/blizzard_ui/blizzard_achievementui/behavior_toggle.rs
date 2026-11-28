//! Behavior pin: `AchievementFrame_ToggleAchievementFrame()` toggles
//! `AchievementFrame` show/hide and the second arg `toggleGuildView` flips
//! the panel into guild mode.
//!
//! Source (`Mainline/Blizzard_AchievementUI.lua:195-223`):
//!
//! ```lua
//! function AchievementFrame_ToggleAchievementFrame(toggleStatFrame, toggleGuildView)
//!     AchievementFrameComparison:Hide();
//!     if C_GameRules.IsGameRuleActive(Enum.GameRule.AchievementsPanelDisabled) then
//!         return;
//!     end
//!
//!     AchievementFrameTab_OnClick = AchievementFrameBaseTab_OnClick;
//!     if ( not toggleStatFrame ) then
//!         if ( AchievementFrame:IsShown() and AchievementFrame.selectedTab == 1 ) then
//!             HideUIPanel(AchievementFrame);
//!         else
//!             AchievementFrame_SetTabs();
//!             ShowUIPanel(AchievementFrame);
//!             if ( toggleGuildView ) then
//!                 AchievementFrameTab_OnClick(2);
//!             else
//!                 AchievementFrameTab_OnClick(1);
//!             end
//!         end
//!         return;
//!     end
//!     -- toggleStatFrame branch omitted (selectedTab == 3, Stats tab); not pinned by PLAN.
//! end
//! ```
//!
//! Three contracts are pinned in a single test that drives the toggle
//! through a hidden→shown→hidden→shown-guild cycle:
//!
//!   1. Toggle from hidden → frame is shown, `selectedTab == 1`
//!      (Achievements tab).
//!   2. Toggle from shown-tab-1 → frame is hidden.
//!   3. Toggle from hidden with `toggleGuildView = true` → frame is shown,
//!      `selectedTab == 2` (Guild tab).
//!
//! All three sub-steps run in one Lua block and return a tuple of
//! observed states so the precondition for each step is the postcondition
//! of the previous step. Splitting into three tests would re-load the
//! addon three times for no behavioral gain.
//!
//! Side-contract observed at every call: `AchievementFrameComparison:Hide()`
//! is the first statement of the function, so the comparison frame is
//! never visible after `ToggleAchievementFrame` returns. The test pins
//! this by pre-showing the comparison frame before step 1 and asserting
//! it's hidden after the call.
//!
//! `Enum.GameRule.AchievementsPanelDisabled` is registered at value 40
//! (`src/lua_api/globals/enum_data/missing_enums.lua:5793`); the default
//! `SimState::game_rules` map has no entry for it so
//! `C_GameRules.IsGameRuleActive(40)` returns false and the guard at line
//! 197 does not short-circuit. The test does NOT pin the
//! AchievementsPanelDisabled-true short-circuit branch — that's a separate
//! contract worth its own test if PLAN ever names it.

#[cfg(not(feature = "client-wowforever"))]
use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

#[cfg(not(feature = "client-wowforever"))]
const ROOT: &str = "Blizzard_AchievementUI";

// Forever 1.60.1.70707 replaces this toggle: Camelot/Blizzard_AchievementUI.lua:13-17
// routes AchievementFrame_ToggleAchievementFrame to ToggleLegacyChallenges, so
// the Mainline AchievementFrame toggle contract does not apply there.
#[cfg(not(feature = "client-wowforever"))]
#[test]
fn toggle_achievement_frame_cycles_show_hide_and_honors_guild_view_arg() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(not AchievementFrame:IsShown())
        AchievementFrame_ToggleAchievementFrame()
        assert(AchievementFrame:IsShown() and AchievementFrame.selectedTab == 1)
        assert(achievementFunctions == ACHIEVEMENT_FUNCTIONS)
        AchievementFrame_ToggleAchievementFrame()
        assert(not AchievementFrame:IsShown())
        AchievementFrame_ToggleAchievementFrame(true)
        assert(AchievementFrame:IsShown() and AchievementFrame.selectedTab == 2)
        assert(achievementFunctions == STAT_FUNCTIONS and AchievementFrameStats:IsShown())
        AchievementFrame_ToggleAchievementFrame(true)
        assert(not AchievementFrame:IsShown())
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: (bool, i64, bool, bool, i64, bool) = env
            .eval(
                r#"
                assert(AchievementFrame, "AchievementFrame global must exist after Blizzard_AchievementUI load")
                assert(AchievementFrameComparison, "AchievementFrameComparison global must exist")
                assert(type(AchievementFrame_ToggleAchievementFrame) == "function",
                    "AchievementFrame_ToggleAchievementFrame must be a function global")

                AchievementFrameComparison:Show()
                AchievementFrame_ToggleAchievementFrame()
                local shown_after_first_toggle = AchievementFrame:IsShown()
                local tab_after_first_toggle = AchievementFrame.selectedTab or -1
                local comparison_hidden_after_first_toggle = not AchievementFrameComparison:IsShown()

                AchievementFrame_ToggleAchievementFrame()
                local shown_after_second_toggle = AchievementFrame:IsShown()

                AchievementFrame_ToggleAchievementFrame(nil, true)
                local shown_after_guild_toggle = AchievementFrame:IsShown()
                local tab_after_guild_toggle = AchievementFrame.selectedTab or -1

                return shown_after_first_toggle,
                       tab_after_first_toggle,
                       comparison_hidden_after_first_toggle,
                       shown_after_second_toggle,
                       tab_after_guild_toggle,
                       shown_after_guild_toggle
                "#,
            )
            .expect("AchievementFrame_ToggleAchievementFrame probe must run cleanly");

        let (
            shown_after_first_toggle,
            tab_after_first_toggle,
            comparison_hidden_after_first_toggle,
            shown_after_second_toggle,
            tab_after_guild_toggle,
            shown_after_guild_toggle,
        ) = observations;

        assert!(
            shown_after_first_toggle,
            "AchievementFrame must be shown after the first \
             AchievementFrame_ToggleAchievementFrame() call from the hidden default state \
             (Mainline/Blizzard_AchievementUI.lua:202-213 — the `not toggleStatFrame` arm \
             with `AchievementFrame:IsShown() == false` falls into the else branch and \
             calls `ShowUIPanel(AchievementFrame)`). A `false` here means either the \
             toggle never reached the show branch (perhaps the gameRule short-circuit at \
             line 197 fired even though no rule is set, or the IsShown precondition was \
             misread), or `ShowUIPanel` no longer flips the visibility flag."
        );

        assert_eq!(
            tab_after_first_toggle, 1,
            "AchievementFrame.selectedTab must be 1 (Achievements tab) after the first \
             toggle from the hidden default state (Mainline/Blizzard_AchievementUI.lua:211 \
             — `AchievementFrameTab_OnClick(1)`). Got `selectedTab = {tab_after_first_toggle}`. \
             A `2` here would mean the toggleGuildView branch fired without the arg being \
             passed; a `0`/`-1` would mean the tab click handler never ran."
        );

        assert!(
            comparison_hidden_after_first_toggle,
            "AchievementFrameComparison must be hidden after every \
             AchievementFrame_ToggleAchievementFrame() call \
             (Mainline/Blizzard_AchievementUI.lua:196 — `AchievementFrameComparison:Hide()` \
             is the first statement of the function, executed unconditionally before any \
             other branch). The probe explicitly Show()s the comparison frame before the \
             first toggle to drive this side-contract; a `false` here means the \
             unconditional Hide() call was removed or moved behind a conditional."
        );

        assert!(
            !shown_after_second_toggle,
            "AchievementFrame must be hidden after the second \
             AchievementFrame_ToggleAchievementFrame() call following the first \
             show-toggle (Mainline/Blizzard_AchievementUI.lua:203-204 — \
             `AchievementFrame:IsShown() and AchievementFrame.selectedTab == 1` triggers \
             `HideUIPanel(AchievementFrame)`). A `true` here would mean the toggle stopped \
             flipping when the frame is shown on tab 1, perhaps because the IsShown/selectedTab \
             guard inverted or the call routed into the toggleStatFrame branch by mistake."
        );

        assert!(
            shown_after_guild_toggle,
            "AchievementFrame must be shown after \
             AchievementFrame_ToggleAchievementFrame(nil, true) called from a hidden state \
             (Mainline/Blizzard_AchievementUI.lua:206-209 — the same `not toggleStatFrame` \
             arm hits the else branch since AchievementFrame is hidden, calls ShowUIPanel, \
             then dispatches `AchievementFrameTab_OnClick(2)` because `toggleGuildView` is \
             truthy). A `false` here means either the show branch didn't fire or \
             `toggleGuildView` was misread as `toggleStatFrame` and routed into the \
             selectedTab==3 branch."
        );

        assert_eq!(
            tab_after_guild_toggle, 2,
            "AchievementFrame.selectedTab must be 2 (Guild tab) after \
             AchievementFrame_ToggleAchievementFrame(nil, true) from a hidden state \
             (Mainline/Blizzard_AchievementUI.lua:208-209 — \
             `if ( toggleGuildView ) then AchievementFrameTab_OnClick(2)`). Got \
             `selectedTab = {tab_after_guild_toggle}`. A `1` here would mean the \
             toggleGuildView arg was dropped (the guard inverted or the arg name changed); \
             a `3` would mean the call was misrouted into the toggleStatFrame branch."
        );
    });
}
