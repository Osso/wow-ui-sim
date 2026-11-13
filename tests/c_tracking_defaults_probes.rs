use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(feature = "client-retail")]
#[test]
fn content_tracking_defaults_are_empty_and_inactive() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local tracked = C_ContentTracking.GetTrackedIDs(0)
            if type(tracked) ~= "table" or #tracked ~= 0 then return "tracked" end
            if C_ContentTracking.IsTracking(0, 42) ~= false then return "tracking" end
            return "ok"
            "#,
        )
        .expect("content tracking defaults should be callable");

    assert_eq!(result, "ok");
}

post_wrath_ui_fixture! {
#[test]
fn neighborhood_initiative_defaults_have_empty_tracked_ids_shape() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_NeighborhoodInitiative.IsInitiativeEnabled() ~= false then return "enabled" end
            local tracked = C_NeighborhoodInitiative.GetTrackedInitiativeTasks()
            if type(tracked) ~= "table" then return "tracked-table" end
            if type(tracked.trackedIDs) ~= "table" or #tracked.trackedIDs ~= 0 then return "ids" end
            if C_NeighborhoodInitiative.GetInitiativeTaskInfo(1) ~= nil then return "info" end
            if C_NeighborhoodInitiative.RemoveTrackedInitiativeTask(1) ~= nil then return "remove" end
            if C_NeighborhoodInitiative.AddTrackedInitiativeTask(1) ~= nil then return "add" end
            return "ok"
            "#,
        )
        .expect("neighborhood initiative defaults should be callable");

    assert_eq!(result, "ok");
}
}
