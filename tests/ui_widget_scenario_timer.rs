//! Original scenario-header timer Setup over supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{ScenarioHeaderTimerRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_scenario_timer_setup_consumes_supplied_values() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.scenario_timers.insert(WidgetId::from_number(762620.0), ScenarioHeaderTimerRecord {
            shown_state: 1, timer_min: 10.0, timer_max: 200.0, timer_value: 100.0,
            header_text: "Supplied timer header".into(), timer_tooltip: "Supplied timer tooltip".into(),
            widget_size_setting: 180.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "scenario-timer".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(433.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762620.0, widget_type: 20, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.ScenarioHeaderTimer == 20)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(433)
        local widget = assert(container.widgetFrames[762620], 'scenario timer was not created')
        assert(widget.HeaderText:GetText() == 'Supplied timer header')
        assert(widget.TimerBar:GetValue() == 100)
        local min, max = widget.TimerBar:GetMinMaxValues()
        assert(min == 10 and max == 200)
        assert(widget.Timer.tooltip == 'Supplied timer tooltip')
        assert(C_UIWidgetManager.GetScenarioHeaderTimerWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
