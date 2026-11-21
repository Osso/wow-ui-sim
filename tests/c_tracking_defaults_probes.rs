use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
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

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
prefork_full_ui_case! {
fn collection_wardrobe_consumer_reflects_supplied_tracking_entries(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FrameXMLUtil')")
        .expect("FrameXMLUtil load state should be available");
    assert!(
        loaded,
        "the selected CollectionsUtil consumer must be loaded"
    );

    let appearance_type: i32 = env
        .eval("return Enum.ContentTrackingType.Appearance")
        .expect("Appearance tracking type should be available");
    env.state()
        .borrow_mut()
        .content_tracking
        .tracked_ids_by_type
        .insert(appearance_type as u32, vec![301, 302]);
    env.state()
        .borrow_mut()
        .content_tracking
        .collectable_source_tracking_enabled = true;
    env.exec("A_Admin.FireEvent('CONTENT_TRACKING_IS_ENABLED_UPDATE', true)")
        .expect("enable the selected content-tracking consumer");

    let consumer_states = consumer_tracking_states(&env);
    let enabled: bool = env
        .eval("return C_ContentTracking.GetCollectableSourceTrackingEnabled()")
        .expect("content-tracking capability query should run");
    assert_eq!(consumer_states, (true, true, false));
    assert!(
        enabled,
        "the C API must return the supplied capability state"
    );
}
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn consumer_tracking_states(env: &WowLuaEnv) -> (bool, bool, bool) {
    env.eval(
        r#"
        local tooltip = { lines = {} }
        function tooltip:SetText() end
        function tooltip:AddDoubleLine(left) table.insert(self.lines, left) end
        function tooltip:AddLine() end
        function tooltip:Show() end
        CollectionWardrobeUtil.SetAppearanceTooltip(tooltip, {
            sources = {
                { sourceID = 301, visualID = 900, isCollected = false },
                { sourceID = 302, visualID = 900, isCollected = false },
                { sourceID = 303, visualID = 900, isCollected = false },
            },
            primarySourceID = 301,
            showTrackingInfo = true,
        })
        return tooltip.lines[1]:find("checkmark%-minimal") ~= nil,
               tooltip.lines[2]:find("checkmark%-minimal") ~= nil,
               tooltip.lines[3]:find("checkmark%-minimal") ~= nil
        "#,
    )
    .expect("CollectionsUtil should render the supplied tracked state")
}

post_wrath_ui_fixture! {
#[test]
fn neighborhood_initiative_defaults_have_empty_tracked_ids_shape() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if type(C_NeighborhoodInitiative.IsInitiativeEnabled) ~= "function" then return "enabled" end
            local tracked = C_NeighborhoodInitiative.GetTrackedInitiativeTasks()
            if type(tracked) ~= "table" then return "tracked-table" end
            if type(tracked.trackedIDs) ~= "table" or #tracked.trackedIDs ~= 0 then return "ids" end
            if type(C_NeighborhoodInitiative.GetInitiativeTaskInfo) ~= "function" then return "info" end
            if C_NeighborhoodInitiative.RemoveTrackedInitiativeTask(1) ~= nil then return "remove" end
            if C_NeighborhoodInitiative.AddTrackedInitiativeTask(1) ~= nil then return "add" end
            return "ok"
            "#,
        )
        .expect("neighborhood initiative defaults should be callable");

    assert_eq!(result, "ok");
}
}
