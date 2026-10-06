#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1127_housing_queries_read_empty_state_without_legacy_fixtures() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_Housing.IsInsidePlot() == false)
        assert(C_Housing.IsOnNeighborhoodMap() == false)
        assert(C_Housing.GetCurrentNeighborhoodGUID() == nil)
        C_Housing.SetTrackedHouseGuid('House-42')
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().housing.tracked_house_guid.as_deref(),
        Some("House-42")
    );
    env.exec("C_Housing.SetTrackedHouseGuid(nil)").unwrap();
    assert_eq!(env.state().borrow().housing.tracked_house_guid, None);
}

#[test]
fn p1127_housing_host_location_is_not_owned_or_tracked_location() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.housing.location.inside_plot = true;
        sim.housing.location.on_neighborhood_map = true;
        sim.housing.location.current_neighborhood_guid = Some("Neighborhood-73".into());
    }
    env.exec(
        r#"
        assert(C_Housing.IsInsidePlot())
        assert(C_Housing.IsOnNeighborhoodMap())
        assert(C_Housing.GetCurrentNeighborhoodGUID() == 'Neighborhood-73')
        assert(not C_Housing.IsInsideOwnedPlot())
        C_Housing.SetTrackedHouseGuid('House-Other')
        assert(C_Housing.GetTrackedHouseGuid() == 'House-Other')
        assert(C_Housing.GetCurrentNeighborhoodGUID() == 'Neighborhood-73')
    "#,
    )
    .unwrap();
    env.state().borrow_mut().housing.location.inside_plot = false;
    env.state().borrow_mut().housing.inside_owned_plot = true;
    env.exec("assert(C_Housing.IsInsidePlot())").unwrap();
}

#[test]
fn p1127_housing_exterior_door_hover_uses_existing_host_snapshot() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_HousingDecor.IsHouseExteriorDoorHovered() == false)")
        .unwrap();
    env.state().borrow_mut().housing.exterior.entry_door_hovered = true;
    env.exec("assert(C_HousingDecor.IsHouseExteriorDoorHovered() == true)")
        .unwrap();
}

#[test]
fn p1127_layout_active_room_count_uses_explicit_floor_records() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_HousingLayout.GetNumActiveRooms() == 0)")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .base_room_floors
        .extend([(8101, -1), (8102, 0), (8103, 0)]);
    env.exec("assert(C_HousingLayout.GetNumActiveRooms() == 3)")
        .unwrap();
}
