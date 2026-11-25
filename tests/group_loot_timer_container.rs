//! GroupLootFrame uses FunctionContainer callbacks with C_Timer.After.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

const GROUP_LOOT_EVENT_LIFECYCLE: &str = r#"
A_Admin.SetManualClock(0)
A_Admin.StartLootRoll(71, 60, "First", "", 4, 0)
A_Admin.StartLootRoll(72, 60, "Second", "", 4, 0)
A_Admin.FireEvent("MAIN_SPEC_NEED_ROLL", 71, 3, true)
A_Admin.FireEvent("MAIN_SPEC_NEED_ROLL", 72, 4, false)
local first, second = GroupLootFrame1, GroupLootFrame2
local firstTimer = first.NeedRollAnimFinishedCallback
local secondTimer = second.NeedRollAnimFinishedCallback
assert(first and second, "two distinct rolls must use two frames")
assert(
    firstTimer and secondTimer and firstTimer ~= secondTimer,
    ("timer mismatch: first=%s second=%s rollIDs=%s/%s animations=%s/%s"):format(
        tostring(firstTimer), tostring(secondTimer), tostring(first.rollID),
        tostring(second.rollID), tostring(first.NeedRollAnim:IsShown()),
        tostring(second.NeedRollAnim:IsShown())
    )
)
assert(first.NeedRollAnim:IsShown() and second.NeedRollAnim:IsShown(), "both roll animations must start")
first:Hide()
assert(firstTimer:IsCancelled(), "frame hide must cancel After's container")
assert(not first.NeedRollAnim:IsShown(), "hidden frame must stop its animation")
first:Show()
A_Admin.AdvanceTime(5)
assert(not second:IsShown(), "uncancelled roll must finish through its timer")
assert(first:IsShown(), "cancelled timer must not remove the first frame")
"#;

#[test]
fn group_loot_events_cancel_only_the_matching_container_timer() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(GROUP_LOOT_EVENT_LIFECYCLE).unwrap();
}
