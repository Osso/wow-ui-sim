use super::discrete_fixtures::assert_lua;
use wow_ui_sim::c_api::ui_widget_manager::ProcessingUnit;
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

#[test]
fn widget_unit_context_tracks_independent_keys_and_processing_selection() {
    let env = WowLuaEnv::new().expect("environment");
    let other = WowLuaEnv::new().expect("independent environment");
    assert_lua(
        &env,
        "register",
        r#"
        C_UIWidgetManager.RegisterUnitForWidgetUpdates('player')
        C_UIWidgetManager.RegisterUnitForWidgetUpdates('player', false)
        C_UIWidgetManager.RegisterUnitForWidgetUpdates('player', true)
        C_UIWidgetManager.RegisterUnitForWidgetUpdates('target')
        C_UIWidgetManager.UnregisterUnitForWidgetUpdates('unknown')
        C_UIWidgetManager.SetProcessingUnit('target')
    "#,
    );
    {
        let state = env.state().borrow();
        let context = &state.ui_widget_inputs.unit_context;
        assert_eq!(context.registrations.len(), 3);
        assert!(context.registrations.contains(&("player".into(), false)));
        assert!(context.registrations.contains(&("player".into(), true)));
        assert_eq!(
            context.processing_unit,
            Some(ProcessingUnit::Token("target".into()))
        );
    }
    assert!(
        other
            .state()
            .borrow()
            .ui_widget_inputs
            .unit_context
            .registrations
            .is_empty()
    );
    assert_lua(
        &env,
        "guid selection",
        r#"
        C_UIWidgetManager.UnregisterUnitForWidgetUpdates('player', true)
        C_UIWidgetManager.SetProcessingUnitGuid('Creature-1')
    "#,
    );
    {
        let state = env.state().borrow();
        let context = &state.ui_widget_inputs.unit_context;
        assert_eq!(context.registrations.len(), 2);
        assert!(context.registrations.contains(&("player".into(), false)));
        assert_eq!(
            context.processing_unit,
            Some(ProcessingUnit::Guid("Creature-1".into()))
        );
    }
    assert_lua(
        &env,
        "clear",
        "C_UIWidgetManager.SetProcessingUnitGuid(nil)",
    );
    assert_eq!(
        env.state()
            .borrow()
            .ui_widget_inputs
            .unit_context
            .processing_unit,
        None
    );
}
