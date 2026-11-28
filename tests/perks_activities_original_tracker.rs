#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const ACTIVITIES_JSON: &str = r#"{
    "activePerksMonth": 11,
    "displayMonthName": "November",
    "secondsRemaining": 3600,
    "activities": [
        {"ID":901,"activityName":"Copper tasks","description":"","thresholdContributionAmount":0,"completed":false,"inProgress":false,"supersedes":0,"uiPriority":0,"areAllConditionsMet":false,"conditions":[],"requirementsList":[],"criteriaList":[],"tagNames":[],"chatLink":""},
        {"ID":902,"activityName":"Silver tasks","description":"","thresholdContributionAmount":0,"completed":false,"inProgress":false,"supersedes":0,"uiPriority":0,"areAllConditionsMet":false,"conditions":[],"requirementsList":[],"criteriaList":[],"tagNames":[],"chatLink":""}
    ],
    "thresholds": [],
    "tags": [],
    "pendingIDs": [],
    "themePrefix": "",
    "trackedIDs": []
}"#;

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    supply_activities(&env);
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env.exec("A_Admin.SetManualClock(0)")
        .expect("manual clock should schedule the original dirty-update callback");
    env
}

fn supply_activities(env: &WowLuaEnv) {
    env.exec(&format!("A_Admin.SetPerksActivities([==[{ACTIVITIES_JSON}]==])"))
        .expect("A_Admin should accept the complete activity snapshot");
}

fn activity_visibility_snapshot(
    env: &WowLuaEnv,
) -> (String, String, bool, bool, i64, i64, String, String, bool, bool) {
    env.eval(
        r#"
        local module = MonthlyActivitiesObjectiveTracker
        local contents = module and module.ContentsFrame
        local blocks = contents and {contents:GetChildren()} or {}
        local parent = module and module:GetParent()
        local tracked = C_PerksActivities.GetTrackedPerksActivities().trackedIDs
        local first = C_PerksActivities.GetPerksActivityInfo(901)
        local second = C_PerksActivities.GetPerksActivityInfo(902)
        local function visible(name)
            for _, block in ipairs(blocks) do
                if block:IsShown() and block.HeaderText:GetText() == name then
                    return true
                end
            end
            return false
        end
        return type(module), parent and parent:GetName() or "",
            module and module:IsShown() or false,
            ObjectiveTrackerFrame and ObjectiveTrackerFrame:IsShown() or false,
            #blocks, #tracked, first and first.activityName or "",
            second and second.activityName or "",
            visible("Copper tasks"), visible("Silver tasks")
        "#,
    )
    .expect("original ObjectiveTracker state should be readable")
}

fn assert_visible_activity_names(env: &WowLuaEnv, copper: bool, silver: bool) {
    let observed = activity_visibility_snapshot(env);
    assert!(
        observed.8 == copper && observed.9 == silver,
        "activity visibility mismatch: expected ({copper}, {silver}), observed {observed:?}"
    );
}

#[test]
fn original_tracker_displays_and_selectively_untracks_two_activities() {
    let env = seeded_env();
    assert!(env.exec("A_Admin.SetPerksActivities('{}')").is_err());
    env.exec("C_PerksActivities.AddTrackedPerksActivity(901); C_PerksActivities.AddTrackedPerksActivity(902)")
        .expect("the original tracker should receive two public tracking changes");
    env.exec("A_Admin.AdvanceTime(0)")
        .expect("next-frame tracker callback should lay out supplied rows");
    assert_visible_activity_names(&env, true, true);

    env.exec("C_PerksActivities.RemoveTrackedPerksActivity(901)")
        .expect("the original tracker should receive the public untracking change");
    env.exec("A_Admin.AdvanceTime(0)")
        .expect("next-frame tracker callback should remove the untracked row");
    assert_visible_activity_names(&env, false, true);
}
