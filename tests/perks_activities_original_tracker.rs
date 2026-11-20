#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::perks_activities::PerksActivity;
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
    env.state().borrow_mut().perks_activities.activities =
        [(901, "Copper tasks"), (902, "Silver tasks")]
            .into_iter()
            .map(|(id, name)| PerksActivity {
                id,
                name: name.to_owned(),
                ..Default::default()
            })
            .collect();
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

#[test]
fn original_tracker_fanfare_uses_the_new_activity_id() {
    seeded_env()
        .exec(
            r#"
        local module = assert(MonthlyActivitiesObjectiveTracker)
        ObjectiveTrackerFrame:AddModule(module)
        module:ClearFanfares()
        C_PerksActivities.AddTrackedPerksActivity(901)
        assert(module:NeedsFanfare(901) and not module:NeedsFanfare(902))
        module:ClearFanfares()
        C_PerksActivities.AddTrackedPerksActivity(902)
        assert(module:NeedsFanfare(902) and not module:NeedsFanfare(901),
            'only the second added activity should receive fanfare')
        module:Update(1000, false)
        module:ClearFanfares()
        assert(not module:IsDirty(), 'initial layout should settle')
        module:UntrackPerksActivity(901)
        assert(module:IsDirty(), 'removal event should invalidate original layout')
        assert(not module:NeedsFanfare(901) and not module:NeedsFanfare(902))
        module:Update(1000, false)
        assert(not module:GetExistingBlock(901))
        local silver = assert(module:GetExistingBlock(902))
        assert(silver:IsShown() and silver.HeaderText:GetText() == 'Silver tasks')
    "#,
        )
        .expect("original tracker event attributes fanfare to each added activity");
}
