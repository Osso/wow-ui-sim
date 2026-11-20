#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    supply_activities(&env);
    env
}

fn supply_activities(env: &WowLuaEnv) {
    env.exec(
        r#"
        local rows = C_PerksActivities._state.activityInfoByID
        rows[901] = { ID = 901, activityName = 'Copper tasks', requirementsList = {} }
        rows[902] = { ID = 902, activityName = 'Silver tasks', requirementsList = {} }
        "#,
    )
    .expect("supply baseline activity records");
}

#[test]
fn original_tracker_displays_and_selectively_untracks_two_activities() {
    seeded_env()
        .exec(
            r#"
            local module = assert(MonthlyActivitiesObjectiveTracker)
            ObjectiveTrackerFrame:AddModule(module)
            C_PerksActivities.AddTrackedPerksActivity(901)
            C_PerksActivities.AddTrackedPerksActivity(902)
            module:Update(1000, false)
            local copper = assert(module:GetExistingBlock(901))
            local silver = assert(module:GetExistingBlock(902))
            assert(copper:IsShown() and copper.HeaderText:GetText() == 'Copper tasks')
            assert(silver:IsShown() and silver.HeaderText:GetText() == 'Silver tasks')
            module:UntrackPerksActivity(901)
            module:Update(1000, false)
            assert(not copper:IsShown(), 'untracked activity should be hidden')
            assert(silver:IsShown() and silver.HeaderText:GetText() == 'Silver tasks')
            "#,
        )
        .expect("original Traveler's Log tracker displays supplied activities");
}
