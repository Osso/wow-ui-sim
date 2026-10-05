//! Retail 12.0.5 globals published by the publication-sweep follow-up:
//! host state in, Lua-visible result out.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua environment")
}

fn drained_event_names(env: &WowLuaEnv) -> Vec<String> {
    let events = env.state().borrow_mut().events.drain();
    events.into_iter().map(|event| event.name).collect()
}

#[test]
fn delves_tiered_entrance_type_reads_host_entrance() {
    let env = env();
    env.exec(
        r#"
        assert(C_DelvesUI.GetTieredEntranceType() == Enum.TieredEntranceType.Delve)
        assert(select('#', C_DelvesUI.GetTieredEntranceType()) == 1)
        "#,
    )
    .expect("seeded entrance is a delve");
    env.state().borrow_mut().tiered_entrance_type = 4;
    env.exec("assert(C_DelvesUI.GetTieredEntranceType() == Enum.TieredEntranceType.Lairs)")
        .expect("live host entrance type");
}

#[test]
fn scenario_display_info_wraps_theme_color_only_inside_themed_scenario() {
    let env = env();
    env.exec("assert(select('#', C_ScenarioInfo.GetDisplayInfo()) == 0)")
        .expect("no scenario, no display info");
    {
        let mut state = env.state().borrow_mut();
        state.scenario.in_scenario = true;
    }
    env.exec("assert(select('#', C_ScenarioInfo.GetDisplayInfo()) == 0)")
        .expect("unthemed scenario has no display info");
    env.state().borrow_mut().scenario.display_theme_color = Some((0.25, 0.5, 1.0));
    env.exec(
        r#"
        local info = C_ScenarioInfo.GetDisplayInfo()
        local r, g, b = info.themeColor:GetRGB()
        assert(r == 0.25 and g == 0.5 and b == 1.0)
        "#,
    )
    .expect("theme color is a ColorMixin");
    env.state().borrow_mut().scenario.in_scenario = false;
    env.exec("assert(select('#', C_ScenarioInfo.GetDisplayInfo()) == 0)")
        .expect("leaving the scenario drops display info");
}

#[test]
fn scenario_tiered_entrance_spells_require_tiered_entrance_scenario() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario.in_scenario = true;
        state.scenario.tiered_entrance_active_spells = vec![1239523, 1239524];
    }
    env.exec("assert(C_ScenarioInfo.GetTieredEntranceActiveSpells() == nil)")
        .expect("ordinary scenarios have no challenge spells");
    env.state().borrow_mut().scenario.is_tiered_entrance = true;
    env.exec(
        r#"
        local spells = C_ScenarioInfo.GetTieredEntranceActiveSpells()
        assert(#spells == 2 and spells[1] == 1239523 and spells[2] == 1239524)
        "#,
    )
    .expect("tiered entrance scenario lists its active spells");
}

#[test]
fn housing_inspect_mode_transitions_fire_state_events_and_gate_hover() {
    let env = env();
    drained_event_names(&env);
    env.state().borrow_mut().housing.inspect_mode.hovered_decor_guid =
        Some("Housing-4-1-100-4F2A".into());
    env.exec(
        r#"
        assert(C_HousingInspectMode.IsInInspectMode() == false)
        assert(C_HousingInspectMode.IsHoveringDecor() == false)
        assert(C_HousingInspectMode.GetHoveredDecorGUID() == '')
        C_HousingInspectMode.EnterInspectMode()
        C_HousingInspectMode.EnterInspectMode()
        assert(C_HousingInspectMode.IsInInspectMode() == true)
        assert(C_HousingInspectMode.IsHoveringDecor() == true)
        assert(C_HousingInspectMode.GetHoveredDecorGUID() == 'Housing-4-1-100-4F2A')
        "#,
    )
    .expect("entering inspect mode reports the hovered decor");
    assert_eq!(
        drained_event_names(&env),
        ["HOUSING_INSPECT_MODE_STATE_UPDATED"],
        "only the real transition fires"
    );
    env.exec(
        r#"
        C_HousingInspectMode.ExitInspectMode()
        assert(C_HousingInspectMode.IsInInspectMode() == false)
        assert(C_HousingInspectMode.IsHoveringDecor() == false)
        "#,
    )
    .expect("exiting hides the hover");
    assert_eq!(
        drained_event_names(&env),
        ["HOUSING_INSPECT_MODE_STATE_UPDATED"]
    );
}

#[test]
fn photo_sharing_status_and_clear_authorization_follow_account_flags() {
    let env = env();
    env.exec(
        r#"
        local status = Enum.PhotoSharingStatus
        assert(C_PhotoSharing.GetStatus() == status.Disabled)
        A_Admin.SetPhotoSharingEnabled(true)
        assert(C_PhotoSharing.GetStatus() == status.NotConfigured)
        A_Admin.SetPhotoSharingAuthorized(true)
        assert(C_PhotoSharing.GetStatus() == status.Configured)
        "#,
    )
    .expect("status derives from the enabled and authorized flags");
    drained_event_names(&env);
    env.exec(
        r#"
        C_PhotoSharing.ClearAuthorization()
        C_PhotoSharing.ClearAuthorization()
        assert(C_PhotoSharing.IsAuthorized() == false)
        assert(C_PhotoSharing.GetStatus() == Enum.PhotoSharingStatus.NotConfigured)
        "#,
    )
    .expect("clearing unlinks the account");
    assert_eq!(
        drained_event_names(&env),
        ["PHOTO_SHARING_AUTHORIZATION_UPDATED"]
    );
}

#[test]
fn photo_sharing_service_members_never_progress_the_flow() {
    let env = env();
    drained_event_names(&env);
    env.exec(
        r#"
        C_PhotoSharing.BeginAuthorizationFlow()
        C_PhotoSharing.CompleteAuthorizationFlow('https://example.invalid/callback')
        C_PhotoSharing.TakePhoto()
        C_PhotoSharing.UploadPhotoToService('title', 'description')
        assert(C_PhotoSharing.IsAuthorized() == false)
        assert(C_PhotoSharing.GetPhotoSharingAuthURL() == '')
        assert(C_PhotoSharing.GetCropRatio() > 1)
        "#,
    )
    .expect("service members are accepted without effect");
    assert!(drained_event_names(&env).is_empty());
}

#[test]
fn unmodeled_cinematic_and_commentator_globals_return_documented_shapes() {
    env()
        .exec(
            r#"
            assert(GetCurrentCinematicSummary() == '')
            local result = C_Commentator.SendAddonMessageLogged('Prefix', 'message', 'PARTY')
            assert(result == Enum.SendAddonMessageResult.Success)
            "#,
        )
        .expect("temporary workarounds keep the documented result types");
}
