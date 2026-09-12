//! Behavior pin: paragon reputation tooltips consume seeded paragon state.
//!
//! Retail defines `ReputationParagonWatchBar_OnEnter` in
//! `Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua`; the ActionBar
//! reputation bar drives the same paragon state and reward quest id.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::{FactionParagonInfo, WowLuaEnv};

const ROOTS: &[&str] = &[
    "Blizzard_ActionBar",
    "Blizzard_GameTooltip",
    "Blizzard_FrameXMLUtil",
    "Blizzard_UIPanels_Game",
];

const WATCHED_FACTION_ID: i64 = 2590;
const PARAGON_CURRENT: i32 = 12_345;
const PARAGON_THRESHOLD: i32 = 10_000;
const PARAGON_PROGRESS_VALUE: i64 = 2_345;
const PARAGON_REWARD_QUEST_ID: i32 = 53_982;
const PARAGON_TOOLTIP_PROBE_LUA: &str = r#"
local captured = {}
local originalRewards = GameTooltip_AddQuestRewardsToTooltip
local originalProgress = GameTooltip_ShowProgressBar

GameTooltip_AddQuestRewardsToTooltip = function(tooltip, questID)
    captured.rewardTooltipIsEmbedded = tooltip == EmbeddedItemTooltip
    captured.rewardQuestID = questID
    tooltip:AddLine("reward quest " .. tostring(questID))
end

GameTooltip_ShowProgressBar = function(tooltip, minValue, maxValue, value)
    captured.progressTooltipIsEmbedded = tooltip == EmbeddedItemTooltip
    captured.progressMin = minValue
    captured.progressMax = maxValue
    captured.progressValue = value
    tooltip:AddLine("progress " .. tostring(value))
end

local bar = CreateFrame("Frame", "ParagonTooltipProbeBar", UIParent)
bar.factionID = 2590
ReputationParagonWatchBar_OnEnter(bar)

GameTooltip_AddQuestRewardsToTooltip = originalRewards
GameTooltip_ShowProgressBar = originalProgress

return EmbeddedItemTooltip:IsShown(),
    EmbeddedItemTooltip.factionID,
    bar.UpdateTooltip == ReputationParagonFrame_SetupParagonTooltip,
    captured.rewardTooltipIsEmbedded,
    captured.rewardQuestID,
    captured.progressTooltipIsEmbedded,
    captured.progressMin,
    captured.progressMax,
    captured.progressValue
"#;

#[test]
fn paragon_watch_bar_tooltip_uses_seeded_reward_quest_and_progress() {
    test_timeout! {
    with_blizzard_addon_closure(ROOTS, &[], |env, _loaded| {
        seed_paragon(env);

        let result = open_paragon_tooltip(env);

        assert!(result.is_shown, "EmbeddedItemTooltip must be shown on enter");
        assert_eq!(result.faction_id, WATCHED_FACTION_ID);
        assert!(result.update_tooltip_set, "bar.UpdateTooltip must be installed");
        assert!(
            result.reward_tooltip_is_embedded,
            "reward block must be added to EmbeddedItemTooltip"
        );
        assert_eq!(result.reward_quest_id, PARAGON_REWARD_QUEST_ID as i64);
        assert!(
            result.progress_tooltip_is_embedded,
            "progress block must be added to EmbeddedItemTooltip"
        );
        assert_eq!(result.progress_min, 0);
        assert_eq!(result.progress_max, PARAGON_THRESHOLD as i64);
        assert_eq!(result.progress_value, PARAGON_PROGRESS_VALUE);
    });
    }
}

fn seed_paragon(env: &WowLuaEnv) {
    env.state().borrow_mut().faction_paragon.insert(
        WATCHED_FACTION_ID,
        FactionParagonInfo {
            current_value: PARAGON_CURRENT,
            threshold: PARAGON_THRESHOLD,
            reward_quest_id: PARAGON_REWARD_QUEST_ID,
            has_reward_pending: false,
            too_low_level_for_paragon: false,
            paragon_storage_level: 2,
        },
    );
}

fn open_paragon_tooltip(env: &WowLuaEnv) -> ParagonTooltipProbe {
    let (
        is_shown,
        faction_id,
        update_tooltip_set,
        reward_tooltip_is_embedded,
        reward_quest_id,
        progress_tooltip_is_embedded,
        progress_min,
        progress_max,
        progress_value,
    ): (bool, i64, bool, bool, i64, bool, i64, i64, i64) = env
        .eval(PARAGON_TOOLTIP_PROBE_LUA)
        .expect("paragon tooltip probe must run cleanly");

    ParagonTooltipProbe {
        is_shown,
        faction_id,
        update_tooltip_set,
        reward_tooltip_is_embedded,
        reward_quest_id,
        progress_tooltip_is_embedded,
        progress_min,
        progress_max,
        progress_value,
    }
}

struct ParagonTooltipProbe {
    is_shown: bool,
    faction_id: i64,
    update_tooltip_set: bool,
    reward_tooltip_is_embedded: bool,
    reward_quest_id: i64,
    progress_tooltip_is_embedded: bool,
    progress_min: i64,
    progress_max: i64,
    progress_value: i64,
}
