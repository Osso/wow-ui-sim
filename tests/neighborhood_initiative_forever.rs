#![cfg(feature = "client-wowforever")]
//! Forever publishes the Retail `C_NeighborhoodInitiative` model (the pinned
//! NeighborhoodInitiativeDocumentation.lua is byte-identical to Retail's).
//! Original consumer: `InitiativeTasksObjectiveTrackerMixin:OnEvent`
//! (Blizzard_ObjectiveTracker/Blizzard_InitiativeTasksObjectiveTracker.lua:14-31)
//! requests the viewed neighborhood's record on `ZONE_CHANGED_NEW_AREA` when a
//! task is tracked; the deferred service reply marks that record loaded.

use wow_ui_sim::lua_api::WowLuaEnv;

const SELECT_AND_REQUEST: &str = r#"
local active, viewing = ...
local N = C_NeighborhoodInitiative
N.SetActiveNeighborhood(active)
N.SetViewingNeighborhood(viewing)
N.AddTrackedInitiativeTask(7)
A_Admin.FireEvent("ZONE_CHANGED_NEW_AREA")
"#;

const READ_VIEWED: &str = r#"
local N = C_NeighborhoodInitiative
local info = N.GetNeighborhoodInitiativeInfo()
return info.neighborhoodGUID, info.isLoaded, N.IsViewingActiveNeighborhood(),
    N.GetActiveNeighborhood(), N.GetInitiativeTaskRewardScaling(7, ...)
"#;

type Viewed = (String, bool, bool, String, f64);

fn request_through_tracker(env: &WowLuaEnv, active: &str, viewing: &str, items: f64) -> Viewed {
    let chunk = format!("return (function(...) {SELECT_AND_REQUEST} end)('{active}', '{viewing}')");
    env.exec(&chunk).expect("the tracker should request the viewed record");
    env.process_timers().expect("the deferred reply should run");
    let read = format!("return (function(...) {READ_VIEWED} end)({items})");
    env.eval(&read).expect("the viewed record should be readable")
}

#[test]
fn forever_initiative_tracker_loads_the_viewed_neighborhood() {
    crate::common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("create Forever Lua environment");
        crate::common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(&env, wow_ui_sim::screen::ScreenKind::Game);
        let a = ("Neighborhood-A".to_string(), true, true, "Neighborhood-A".to_string(), 3.0);
        assert_eq!(request_through_tracker(&env, "Neighborhood-A", "Neighborhood-A", 3.0), a);
        let b = ("Neighborhood-B".to_string(), true, false, "Neighborhood-A".to_string(), 5.0);
        assert_eq!(request_through_tracker(&env, "Neighborhood-A", "Neighborhood-B", 5.0), b);
    });
}
