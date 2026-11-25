//! GroupLootFrame uses FunctionContainer callbacks with C_Timer.After.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use std::path::Path;
use wow_ui_sim::loader::{find_toc_file, load_addon};

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
    load_retail_game_event_router(&env);
    env.exec(GROUP_LOOT_EVENT_LIFECYCLE).unwrap();
}

#[cfg(feature = "client-retail")]
fn load_retail_game_event_router(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!("CARGO_MANIFEST_DIR")));
    let game_addon = ui.join("Blizzard_Game");
    let toc = find_toc_file(&game_addon).expect("Retail Blizzard_Game TOC must exist");
    load_addon(&env.loader_env(), &toc)
        .unwrap_or_else(|error| panic!("load Blizzard_Game from {}: {error}", toc.display()));
}

#[cfg(feature = "client-wowforever")]
fn load_retail_game_event_router(_: &wow_ui_sim::lua_api::WowLuaEnv) {}
