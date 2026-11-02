//! Original unit-power bar Setup over supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{UnitPowerBarRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_unit_power_bar_setup_consumes_supplied_values() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.unit_power_bars.insert(WidgetId::from_number(762623.0), UnitPowerBarRecord {
            shown_state: 1, bar_min: 10.0, bar_max: 100.0, bar_value: 60.0,
            tooltip: "Supplied power".into(), bar_value_text_type: 0, override_bar_text: String::new(),
            override_bar_text_shown_type: 0, tooltip_loc: 4, fill_motion_type: 0, flash_blend_mode_type: 0,
            spark_blend_mode_type: 0, flash_moment_type: 0,
            widget_size_setting: 2.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "power-bar".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(434.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762623.0, widget_type: 23, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.UnitPowerBar == 23)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(434)
        local widget = assert(container.widgetFrames[762623], 'power bar was not created')
        assert(widget.barMin == 10 and widget.barMax == 100 and widget.lastBarValue == 60)
        assert(widget.tooltip == 'Supplied power')
        assert(C_UIWidgetManager.GetUnitPowerBarWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
