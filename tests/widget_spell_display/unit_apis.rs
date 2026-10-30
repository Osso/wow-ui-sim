use super::discrete_fixtures::assert_lua;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn widget_unit_api_arguments_and_return_values() {
    let env = WowLuaEnv::new().expect("environment");
    assert_lua(
        &env,
        "widget unit API contract",
        r#"
        local api = C_UIWidgetManager
        assert(select('#', api.RegisterUnitForWidgetUpdates('player')) == 0)
        assert(select('#', api.RegisterUnitForWidgetUpdates('Creature-1', true)) == 0)
        assert(select('#', api.RegisterUnitForWidgetUpdates('player', false)) == 0)
        assert(select('#', api.UnregisterUnitForWidgetUpdates('missing')) == 0)
        assert(select('#', api.UnregisterUnitForWidgetUpdates('Creature-1', true)) == 0)
        assert(select('#', api.SetProcessingUnit('player')) == 0)
        assert(select('#', api.SetProcessingUnitGuid('Creature-1')) == 0)
        assert(select('#', api.SetProcessingUnit(nil)) == 0)
        assert(select('#', api.SetProcessingUnitGuid(nil)) == 0)
        assert(not pcall(api.RegisterUnitForWidgetUpdates, nil))
        assert(not pcall(api.UnregisterUnitForWidgetUpdates, {}))
        assert(not pcall(api.RegisterUnitForWidgetUpdates, 'player', {}))
        assert(not pcall(api.SetProcessingUnit, {}))
        assert(not pcall(api.SetProcessingUnitGuid, {}))
    "#,
    );
}
