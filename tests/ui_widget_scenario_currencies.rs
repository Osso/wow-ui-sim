//! Original scenario header currencies Setup over explicitly supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use crate::ui_widget_stacked_resource::resource;
use wow_ui_sim::c_api::ui_widget_manager::{ScenarioHeaderCurrenciesRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_scenario_header_setup_consumes_supplied_currencies() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.scenario_currencies.insert(WidgetId::from_number(762611.0), ScenarioHeaderCurrenciesRecord {
            shown_state: 1,
            currencies: vec![resource("First currency", 135891), resource("Second currency", 135892)],
            header_text: "Supplied scenario".into(), widget_size_setting: 180.0,
            texture_kit: String::new(), frame_texture_kit: String::new(), has_timer: false,
            order_index: 10.0, widget_tag: "scenario-currencies".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0,
            scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(427.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762611.0, widget_type: 11, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.ScenarioHeaderCurrenciesAndBackground == 11)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(427)
        local widget = assert(container.widgetFrames[762611], 'scenario header was not created')
        assert(widget.HeaderText:GetText() == 'Supplied scenario')
        assert(widget.currencyPool:GetNumActive() == 2)
        local found = {}
        for frame in widget.currencyPool:EnumerateActive() do found[frame.Text:GetText()] = true end
        assert(found['First currency'] and found['Second currency'])
        assert(C_UIWidgetManager.GetScenarioHeaderCurrenciesAndBackgroundWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
