#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_addon_loaded(env: &WowLuaEnv, addon: &str) {
    let loaded: bool = env
        .eval(&format!("return C_AddOns.IsAddOnLoaded({addon:?})"))
        .unwrap_or_else(|error| panic!("query load state for {addon}: {error}"));
    assert!(
        loaded,
        "selected Retail consumer addon {addon} is not loaded"
    );
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn expansion_landing_page_consumes_supplied_unlock(env: &WowLuaEnv) {
    assert_addon_loaded(env, "Blizzard_ExpansionLandingPage");

    let locked: bool = env
        .eval("return ExpansionLandingPageMixin:GetNewestExpansionOverlayForPlayer() == nil")
        .expect("landing-page consumer query should run");
    assert!(locked, "the unsupplied landing-page unlock input is false");

    env.state().borrow_mut().expansion_landing_page_unlocked = true;
    let unlocked: bool = env
        .eval("return ExpansionLandingPageMixin:GetNewestExpansionOverlayForPlayer() ~= nil")
        .expect("landing-page consumer should observe the supplied unlock");
    assert!(unlocked, "Midnight overlay consumer did not observe the supplied unlock");

    env.state().borrow_mut().expansion_landing_page_unlocked = false;
    let locked_again: bool = env
        .eval("return ExpansionLandingPageMixin:GetNewestExpansionOverlayForPlayer() == nil")
        .expect("landing-page consumer should observe a cleared unlock");
    assert!(locked_again, "Midnight overlay consumer retained a cleared unlock");
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn content_tracking_element_consumes_membership_changes(env: &WowLuaEnv) {
    assert_addon_loaded(env, "Blizzard_ContentTracking");
    env.state()
        .borrow_mut()
        .content_tracking
        .tracked_ids_by_type
        .insert(0, vec![31]);

    let checkmark_updates: bool = env
        .eval(
            r#"
            local shown
            local element = {
                trackables = {{ trackableType = 0, id = 31 }},
                SetTrackingCheckmarkShown = function(_, value) shown = value end,
            }
            ContentTrackingElementMixin.UpdateTrackingCheckmark(element)
            if shown ~= true then return false end
            element.trackables = {{ trackableType = 0, id = 99 }}
            ContentTrackingElementMixin.UpdateTrackingCheckmark(element)
            return shown == false
            "#,
        )
        .expect("ContentTrackingElementMixin should consume IsTracking results");
    assert!(
        checkmark_updates,
        "the original tracking-element consumer must show a match and hide a miss"
    );
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn content_tracking_utility_consumes_ordered_duplicate_ids(env: &WowLuaEnv) {
    assert_addon_loaded(env, "Blizzard_ContentTracking");
    env.exec(
        r#"
        _G.__r88e5_tracking_visits = {}
        C_ContentTracking.GetCollectableSourceTypes = function() return { 0 } end
        C_ContentTracking.GetCurrentTrackingTarget = function(_, id)
            table.insert(_G.__r88e5_tracking_visits, tostring(id))
            return nil
        end
        "#,
    )
    .expect("install unrelated tracking-consumer inputs");
    env.state()
        .borrow_mut()
        .content_tracking
        .tracked_ids_by_type
        .insert(0, vec![9, 9, 4]);

    let first: String = env
        .eval(
            r#"
            local tracked = ContentTrackingUtil.IsContentTrackedInEncounter(700)
            return tostring(tracked) .. ":" .. table.concat(_G.__r88e5_tracking_visits, ",")
            "#,
        )
        .expect("ContentTrackingUtil should consume GetTrackedIDs");
    assert_eq!(first, "false:9,9,4");

    env.state()
        .borrow_mut()
        .content_tracking
        .tracked_ids_by_type
        .insert(0, vec![2, 8]);
    let second: String = env
        .eval(
            r#"
            _G.__r88e5_tracking_visits = {}
            local tracked = ContentTrackingUtil.IsContentTrackedInEncounter(700)
            return tostring(tracked) .. ":" .. table.concat(_G.__r88e5_tracking_visits, ",")
            "#,
        )
        .expect("ContentTrackingUtil should observe the replacement tracking list");
    assert_eq!(second, "false:2,8");
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn housing_dashboard_consumes_available_house_xp(env: &WowLuaEnv) {
    env.exec("C_AddOns.LoadAddOn('Blizzard_HousingDashboard')")
        .expect("load the Retail housing dashboard LoD consumer");
    assert_addon_loaded(env, "Blizzard_HousingDashboard");

    let no_neighborhood: bool = env
        .eval(
            r#"
            local icon = {}
            function icon:SetShown(value) self.shown = value end
            HouseXPCapIconMixin.UpdateVisibility(icon)
            return icon.shown == false
            "#,
        )
        .expect("housing dashboard should receive zero with no active neighborhood");
    assert!(no_neighborhood, "the cap icon must hide with no active neighborhood");

    env.state()
        .borrow_mut()
        .housing
        .initiative
        .active_neighborhood = "neighborhood-a".to_string();
    env.state()
        .borrow_mut()
        .housing
        .initiative
        .house_experience
        .insert(
            "neighborhood-a".to_string(),
            wow_ui_sim::c_api::c_neighborhood_initiative::model::HouseExperienceInput {
                total_xp: 500,
                used_xp: 100,
            },
        );
    let visible: bool = env
        .eval(
            r#"
            local icon = {}
            function icon:SetShown(value) self.shown = value end
            HouseXPCapIconMixin.UpdateVisibility(icon)
            return icon.shown == true
            "#,
        )
        .expect("housing dashboard should observe positive available XP");
    assert!(visible, "the cap icon must show when total XP exceeds used XP");

    env.state()
        .borrow_mut()
        .housing
        .initiative
        .house_experience
        .insert(
            "neighborhood-a".to_string(),
            wow_ui_sim::c_api::c_neighborhood_initiative::model::HouseExperienceInput {
                total_xp: 100,
                used_xp: 100,
            },
        );
    let hidden: bool = env
        .eval(
            r#"
            local icon = {}
            function icon:SetShown(value) self.shown = value end
            HouseXPCapIconMixin.UpdateVisibility(icon)
            return icon.shown == false
            "#,
        )
        .expect("housing dashboard should observe zero available XP");
    assert!(hidden, "the cap icon must hide when used XP reaches total XP");
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn pvp_match_cell_consumes_honor_reward_catalog(env: &WowLuaEnv) {
    assert_addon_loaded(env, "Blizzard_PVPMatch");
    {
        let mut sim = env.state().borrow_mut();
        sim.pvp_catalog.honor_rewards.insert(
            2,
            wow_ui_sim::c_api::c_pvp::catalog::HonorRewardInfo {
                honor_level_name: "Two".to_string(),
                badge_file_data_id: 202,
                achievement_rewarded_id: 2002,
            },
        );
        sim.pvp_catalog.honor_rewards.insert(
            6,
            wow_ui_sim::c_api::c_pvp::catalog::HonorRewardInfo {
                honor_level_name: "Six".to_string(),
                badge_file_data_id: 606,
                achievement_rewarded_id: 6006,
            },
        );
        sim.pvp_catalog.honor_rewards.insert(
            9,
            wow_ui_sim::c_api::c_pvp::catalog::HonorRewardInfo {
                honor_level_name: "Nine".to_string(),
                badge_file_data_id: 909,
                achievement_rewarded_id: 9009,
            },
        );
    }

    let badges: (i64, i64) = env
        .eval(
            r#"
            local function populate(level)
                local cell = {
                    rowData = { honorLevel = level },
                    icon = { SetTexture = function(self, fileID) self.fileID = fileID end },
                }
                PVPCellHonorLevelMixin.Populate(cell, cell.rowData, 1)
                return cell.icon.fileID or -1
            end
            return populate(3), populate(8)
            "#,
        )
        .expect("PVPMatch consumer should query honor reward rows");
    assert_eq!(badges, (202, 606));
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn combat_audio_consumer_stops_when_manager_is_unavailable(env: &WowLuaEnv) {
    assert_addon_loaded(env, "Blizzard_CombatAudioAlerts");
    {
        let mut sim = env.state().borrow_mut();
        sim.sound_manager = None;
        sim.cvars.set_imported("CAAEnabled", "1");
    }

    let stopped: bool = env
        .eval(
            r#"
            local addon = C_AddOns.GetAddOnLocalTable("Blizzard_CombatAudioAlerts")
            if type(addon) ~= "table" or type(addon.TrySpeakText) ~= "function" then
                return false
            end
            if addon.IsEnabled() ~= false then return false end

            local throttleChecked, speechStarted = false, false
            local oldCheckThrottle, oldSpeakText = addon.CheckThrottle, addon.SpeakText
            addon.CheckThrottle = function() throttleChecked = true; return true end
            addon.SpeakText = function() speechStarted = true end
            addon:TrySpeakText({})
            addon.CheckThrottle, addon.SpeakText = oldCheckThrottle, oldSpeakText
            return not throttleChecked and not speechStarted
            "#,
        )
        .expect("CombatAudioAlert manager should stop on unavailable audio manager");
    assert!(stopped, "the original CAA consumer must not continue without its manager");
}
}
