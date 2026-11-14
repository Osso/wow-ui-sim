#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::c_neighborhood_initiative::model::{
    InitiativeTaskInfo, NeighborhoodInitiativeInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn ensure_loaded(env: &WowLuaEnv, addon: &str) {
    let loaded: bool = env
        .eval(&format!("return C_AddOns.IsAddOnLoaded('{addon}')"))
        .expect("Blizzard addon load state should be queryable");
    if !loaded {
        crate::common::load_required_blizzard_addon(env, &crate::common::blizzard_ui_dir(), addon);
    }
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn neighborhood_initiative_available_house_xp_defaults_to_zero() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if type(C_NeighborhoodInitiative.GetAvailableHouseXP) ~= "function" then
                return "missing_get_available_house_xp"
            end
            if type(C_NeighborhoodInitiative.IsInitiativeEnabled) ~= "function" then
                return "missing_is_initiative_enabled"
            end
            if C_NeighborhoodInitiative.GetAvailableHouseXP() ~= 0 then
                return "available_house_xp_should_default_zero"
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_NeighborhoodInitiative.GetAvailableHouseXP should exist and default to zero"
    );
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn r88e5_initiative_enabled_dashboard_reads_host_value(env: &WowLuaEnv) {
    for dependency in [
        "Blizzard_HousingTemplates",
        "Blizzard_HousingModelPreview",
        "Blizzard_FrameXMLUtil",
        "Blizzard_HousingBlueprint",
    ] {
        ensure_loaded(env, dependency);
    }
    ensure_loaded(env, "Blizzard_HousingDashboard");
    {
        let mut state = env.state().borrow_mut();
        state.housing.initiative.player_has_access = true;
        state.housing.initiative.enabled = Some(true);
    }

    let enabled: (bool, bool) = env
        .eval(
            r#"
            function _G.__r88e5UpdateInitiativeTabs()
                local accepted
                local tabs = {
                    SetTabShown = function() end,
                    SetTabEnabled = function(_, tabID, value)
                        if tabID == 2 then accepted = value end
                    end,
                }
                local dashboard = {
                    houseUpgradeTabID = 1,
                    endeavorTabID = 2,
                    TabSystem = tabs,
                }
                function dashboard:IsTabAvailable() return true end
                function dashboard:GetTab() return self.houseUpgradeTabID end
                function dashboard:SetToDefaultAvailableTab() end

                HousingDashboardHouseInfoContentFrameMixin.UpdateTabs(dashboard)
                return accepted
            end
            return __r88e5UpdateInitiativeTabs(),
                C_NeighborhoodInitiative.IsInitiativeEnabled()
            "#,
        )
        .expect("housing dashboard and C API should consume initiative enablement");

    assert_eq!(enabled, (true, true));

    env.state().borrow_mut().housing.initiative.enabled = Some(false);
    let disabled: (bool, bool) = env
        .eval(
            "return __r88e5UpdateInitiativeTabs(), C_NeighborhoodInitiative.IsInitiativeEnabled()",
        )
        .expect("disabled host value should reach the dashboard and C API");
    assert_eq!(disabled, (false, false));
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn r88e5_initiative_task_consumer_uses_matching_host_records(env: &WowLuaEnv) {
    let tracker_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ObjectiveTracker')")
        .expect("Retail objective tracker load state should be queryable");
    assert!(tracker_loaded, "ObjectiveTracker has no LoD or profile exclusion");

    {
        let mut state = env.state().borrow_mut();
        let initiative = &mut state.housing.initiative;
        initiative.active_neighborhood = "Neighborhood-A".to_string();
        initiative.viewing_neighborhood = "Neighborhood-A".to_string();
        initiative.neighborhoods.insert(
            "Neighborhood-A".to_string(),
            NeighborhoodInitiativeInfo {
                tasks: vec![
                    InitiativeTaskInfo {
                        id: 7,
                        in_progress: true,
                        ..Default::default()
                    },
                    InitiativeTaskInfo {
                        id: 8,
                        completed: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
        state.neighborhood_tracked_tasks.extend([7, 8]);
    }

    let results: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local tracker = { AddTask = function() return true end }
            InitiativeTasksObjectiveTrackerMixin.LayoutContents(tracker)
            local fractional = C_NeighborhoodInitiative.GetInitiativeTaskInfo(7.9)
            local completed = C_NeighborhoodInitiative.GetInitiativeTaskInfo(8)
            local nilAccepted = pcall(function()
                C_NeighborhoodInitiative.GetInitiativeTaskInfo(nil)
            end)
            local upperFractionAccepted = pcall(function()
                C_NeighborhoodInitiative.GetInitiativeTaskInfo(2147483647.9)
            end)
            local lowerFractionAccepted = pcall(function()
                C_NeighborhoodInitiative.GetInitiativeTaskInfo(-2147483648.9)
            end)
            return fractional ~= nil, completed ~= nil, nilAccepted,
                upperFractionAccepted, lowerFractionAccepted
            "#,
        )
        .expect("objective tracker should consume initiative task records");

    assert_eq!(results, (true, true, false, false, false));
}
}
