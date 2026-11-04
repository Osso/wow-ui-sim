//! `AnimaDiversionPinMixin:RefreshTooltip` branch probes.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::state::{GarrisonTalentCurrencyCostInfo, GarrisonTalentInfo};

const ROOT: &str = "Blizzard_AnimaDiversionUI";
const IMPLICIT_DEPS: &[&str] = &["Blizzard_MapCanvas", "Blizzard_SharedMapDataProviders"];
const CLOSURE_OVERRIDES: &[BlizzardAddonOverride<'_>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: IMPLICIT_DEPS,
}];
const TOOLTIP_PROBE: &str = r#"
local nodeState = Enum.AnimaDiversionNodeState
local originalGetAnimaInfo = C_CovenantSanctumUI.GetAnimaInfo
local originalGetCurrencyInfo = C_CurrencyInfo.GetCurrencyInfo
local originalHaveQuestRewardData = HaveQuestRewardData
local originalAddQuestRewards = GameTooltip_AddQuestRewardsToTooltip
local currencyQuantity = 300
local canReinforce = false
local rewardCallCount = 0
local rewardQuestID = nil

C_CovenantSanctumUI.GetAnimaInfo = function()
    return 1820, 1000
end
C_CurrencyInfo.GetCurrencyInfo = function(currencyID)
    return { quantity = currencyQuantity, iconFileID = currencyID }
end
HaveQuestRewardData = function()
    return false
end
GameTooltip_AddQuestRewardsToTooltip = function(_tooltip, questID)
    rewardCallCount = rewardCallCount + 1
    rewardQuestID = questID
end

local owner = {
    CanReinforceNode = function()
        return canReinforce
    end,
}

local function buildPin(nodeData)
    local pin = CreateFrame("Frame", nil, UIParent)
    pin.owner = owner
    pin.nodeData = nodeData
    Mixin(pin, AnimaDiversionPinMixin)
    return pin
end

local function findLine(text)
    for index = 1, GameTooltip:NumLines() do
        local line = GameTooltip:GetLeftLine(index)
        if line and line:GetText() == text then
            return line
        end
    end
    return nil
end

local function lineMatchesColor(line, color)
    if not line then
        return false
    end

    local red, green, blue = line:GetTextColor()
    local expectedRed, expectedGreen, expectedBlue = color:GetRGB()
    return math.abs(red - expectedRed) < 0.001
        and math.abs(green - expectedGreen) < 0.001
        and math.abs(blue - expectedBlue) < 0.001
end

local function refresh(nodeData)
    local pin = buildPin(nodeData)
    -- Original OnMouseEnter sets the owner before RefreshTooltip (:278-279).
    pin:OnMouseEnter()
    return pin
end

refresh(nil)
local originMatches = GameTooltip:NumLines() == 1
    and GameTooltip:GetLeftLine(1):GetText() == ANIMA_DIVERSION_ORIGIN_TOOLTIP

refresh({
    name = "Dormant Mirror",
    description = "Unavailable branch",
    state = nodeState.Unavailable,
    talentID = 9001,
})
local unavailableLine = findLine(ANIMA_DIVERSION_NODE_UNAVAILABLE)
local unavailableMatches = lineMatchesColor(unavailableLine, RED_FONT_COLOR)

canReinforce = false
refresh({
    name = "Cooling Mirror",
    description = "Cooldown branch",
    state = nodeState.Cooldown,
    talentID = 9001,
})
local cooldownLine = findLine(ANIMA_DIVERSION_NODE_COOLDOWN)
local cooldownMatches = lineMatchesColor(cooldownLine, RED_FONT_COLOR)

refresh({
    name = "Reinforced Mirror",
    description = "Permanent branch",
    state = nodeState.SelectedPermanent,
    talentID = 9001,
})
local reinforcedLine = findLine(ANIMA_DIVERSION_POI_REINFORCED)
local reinforcedMatches = lineMatchesColor(reinforcedLine, GREEN_FONT_COLOR)

currencyQuantity = 300
refresh({
    name = "Hungry Mirror",
    description = "Insufficient branch",
    state = nodeState.Available,
    talentID = 9001,
})
local insufficientLine = findLine(ANIMA_DIVERSION_NOT_ENOUGH_CURRENCY)
local insufficientMatches = lineMatchesColor(insufficientLine, RED_FONT_COLOR)

currencyQuantity = 800
local rewardPin = refresh({
    name = "Ready Mirror",
    description = "Sufficient branch",
    state = nodeState.Available,
    talentID = 9001,
})
local sufficientLine = findLine(ANIMA_DIVERSION_CLICK_CHANNEL)
local sufficientMatches = lineMatchesColor(sufficientLine, GREEN_FONT_COLOR)

C_CovenantSanctumUI.GetAnimaInfo = originalGetAnimaInfo
C_CurrencyInfo.GetCurrencyInfo = originalGetCurrencyInfo
HaveQuestRewardData = originalHaveQuestRewardData
GameTooltip_AddQuestRewardsToTooltip = originalAddQuestRewards

return originMatches,
       unavailableMatches,
       cooldownMatches,
       reinforcedMatches,
       insufficientMatches,
       sufficientMatches,
       rewardCallCount,
       rewardQuestID,
       rewardPin.UpdateTooltip == rewardPin.RefreshTooltip
"#;

#[test]
fn pin_refresh_tooltip_branches_on_state_currency_and_quest_rewards() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        seed_tooltip_talent(env);

        let state: TooltipBranchState = env
            .eval(TOOLTIP_PROBE)
            .expect("pin tooltip probe must run cleanly");

        assert_tooltip_branch_state(state);
    });
}

type TooltipBranchState = (bool, bool, bool, bool, bool, bool, i64, i64, bool);

fn seed_tooltip_talent(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.garrison_talents.talents.insert(
        9001,
        GarrisonTalentInfo {
            id: 9001,
            name: "Tooltip Probe Talent".into(),
            description: "Tooltip probe description".into(),
            research_currency_costs: vec![GarrisonTalentCurrencyCostInfo {
                currency_type: 1820,
                currency_quantity: 500,
            }],
            ..Default::default()
        },
    );
    state
        .garrison_talents
        .unlock_world_quests
        .insert(9001, 4242);
}

fn assert_tooltip_branch_state(state: TooltipBranchState) {
    assert_text_branches((state.0, state.1, state.2, state.3));
    assert_currency_branches((state.4, state.5));
    assert_reward_refresh((state.6, state.7, state.8));
}

fn assert_text_branches(state: (bool, bool, bool, bool)) {
    let (origin_matches, unavailable_matches, cooldown_matches, reinforced_matches) = state;

    assert!(
        origin_matches,
        "Origin pin must show the origin tooltip only"
    );
    assert!(
        unavailable_matches,
        "Unavailable pin must add the unavailable line in red"
    );
    assert!(
        cooldown_matches,
        "Cooldown pin must add the cooldown line in red when not reinforcing"
    );
    assert!(
        reinforced_matches,
        "Permanently selected pin must add the reinforced line in green"
    );
}

fn assert_currency_branches(state: (bool, bool)) {
    let (insufficient_matches, sufficient_matches) = state;

    assert!(
        insufficient_matches,
        "Available pin with insufficient anima must add the not-enough-currency line in red"
    );
    assert!(
        sufficient_matches,
        "Available pin with enough anima must add the click-to-channel line in green"
    );
}

fn assert_reward_refresh(state: (i64, i64, bool)) {
    let (reward_call_count, reward_quest_id, update_tooltip_armed) = state;

    assert_eq!(
        reward_call_count, 5,
        "Every non-origin pin with an unlock world quest must append quest rewards"
    );
    assert_eq!(
        reward_quest_id, 4242,
        "Reward helper must receive the world quest unlocked by the talent"
    );
    assert!(
        update_tooltip_armed,
        "Missing quest reward data must re-arm UpdateTooltip"
    );
}
