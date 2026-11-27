//! C_Housing market, charter and current-house queries over supplied host
//! state. Retail and Forever HousingUIDocumentation.lua and
//! HousingUISharedDocumentation.lua are byte-identical. Moved out of the
//! Retail-only housing_current_house_info.rs, patch_12_0_1_policy.rs,
//! mainline_season_profession_queries.rs and the prefork charter case so
//! Forever runs them. Fixture values do not establish native housing policy.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::c_housing::current_house::HouseInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

const MARKET_PROBE: &str = "return C_Housing.IsHousingMarketEnabled(), \
     C_Housing.IsHousingMarketCartFullRemoveEnabled()";

const FULL_HOUSE_INFO_CHECK: &str = r#"
    assert(select('#', C_Housing.GetCurrentHouseInfo()) == 1)
    local info = C_Housing.GetCurrentHouseInfo()
    assert(type(info) == 'table' and info.plotID == 27)
    assert(info.houseName == 'Fixture House' and info.ownerName == 'Fixture Owner')
    assert(info.plotCost == 12345 and info.neighborhoodName == 'Fixture Neighborhood')
    assert(info.moveOutTime == 1800000000 and info.plotReserved == false)
    assert(info.neighborhoodGUID == 'fixture-neighborhood' and info.houseGUID == 'fixture-house')
"#;

const SPARSE_HOUSE_INFO_CHECK: &str = r#"
    local info = C_Housing.GetCurrentHouseInfo()
    assert(info.plotID == 41 and info.plotReserved == true)
    for _, key in ipairs({'houseName', 'ownerName', 'plotCost', 'neighborhoodName',
        'moveOutTime', 'neighborhoodGUID', 'houseGUID'}) do
        assert(info[key] == nil, key)
    end
"#;

fn full_house_info() -> HouseInfo {
    HouseInfo {
        plot_id: 27.0,
        house_name: Some("Fixture House".into()),
        owner_name: Some("Fixture Owner".into()),
        plot_cost: Some(12345.0),
        neighborhood_name: Some("Fixture Neighborhood".into()),
        move_out_time: Some(1800000000),
        plot_reserved: Some(false),
        neighborhood_guid: Some("fixture-neighborhood".into()),
        house_guid: Some("fixture-house".into()),
    }
}

fn sparse_house_info() -> HouseInfo {
    HouseInfo {
        plot_id: 41.0,
        house_name: None,
        owner_name: None,
        plot_cost: None,
        neighborhood_name: None,
        move_out_time: None,
        plot_reserved: Some(true),
        neighborhood_guid: None,
        house_guid: None,
    }
}

#[test]
fn current_house_info_preserves_documented_nullable_shape() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "assert(select('#', C_Housing.GetCurrentHouseInfo()) == 1); \
         assert(C_Housing.GetCurrentHouseInfo() == nil)",
    )
    .expect("an unsupplied nullable snapshot is one nil, not zero returns");
    env.state().borrow_mut().housing.current_house_info = Some(full_house_info());
    env.exec(FULL_HOUSE_INFO_CHECK)
        .expect("all supplied documented fields project without invented values");
    env.state().borrow_mut().housing.current_house_info = Some(sparse_house_info());
    env.exec(SPARSE_HOUSE_INFO_CHECK)
        .expect("nullable fields stay absent rather than receiving placeholder defaults");
    env.state().borrow_mut().housing.current_house_info = None;
    env.exec("assert(C_Housing.GetCurrentHouseInfo() == nil)")
        .expect("clearing the supplied snapshot clears the query");
}

#[test]
fn market_policies_report_each_supplied_flag_independently() {
    let env = WowLuaEnv::new().unwrap();
    let read = |env: &WowLuaEnv| env.eval::<(bool, bool)>(MARKET_PROBE).unwrap();
    assert_eq!(read(&env), (false, false), "unconfigured policies stay off");
    env.state().borrow_mut().housing.market_enabled = true;
    assert_eq!(read(&env), (true, false));
    {
        let mut sim = env.state().borrow_mut();
        sim.housing.market_enabled = false;
        sim.housing.market_cart_full_remove_enabled = true;
    }
    assert_eq!(read(&env), (false, true));
    assert_eq!(read(&WowLuaEnv::new().unwrap()), (false, false));
}

#[test]
fn charter_editability_reports_supplied_capability() {
    let env = WowLuaEnv::new().unwrap();
    for can_edit in [false, true, false] {
        env.state().borrow_mut().housing.charter_can_edit = can_edit;
        let (reported, count): (bool, i64) = env
            .eval("return C_Housing.CanEditCharter(), select('#', C_Housing.CanEditCharter())")
            .expect("charter query should return one boolean");
        assert_eq!((reported, count), (can_edit, 1));
    }
}
