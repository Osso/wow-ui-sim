//! Original tug-of-war Setup over supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{TugOfWarRecord, WidgetIconRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

fn icon(tooltip: &str) -> WidgetIconRecord {
    WidgetIconRecord { source_type: 0, source_id: 0.0, size_type: 0, tooltip: tooltip.into(), tooltip_loc: 4 }
}

#[test]
fn cached_tug_of_war_setup_consumes_supplied_values() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.tug_of_war.insert(WidgetId::from_number(762628.0), TugOfWarRecord {
            shown_state: 1, min_value: 0.0, max_value: 100.0, current_value: 30.0,
            neutral_zone_center: 50.0, neutral_zone_size: 20.0, left_icon_info: icon("Left"), right_icon_info: icon("Right"),
            glow_anim_type: 0, tooltip: "Supplied tug".into(), tooltip_loc: 4, neutral_fill_style: 0, marker_arrow_shown_state: 0,
            widget_size_setting: 0.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "tug".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(436.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762628.0, widget_type: 28, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.TugOfWar == 28)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(436)
        local widget = assert(container.widgetFrames[762628], 'tug of war was not created')
        assert(widget.currentValue == 30 and widget.neutralZoneCenter == 50 and widget.neutralZoneSize == 20)
        assert(widget.tooltip == 'Supplied tug')
        assert(C_UIWidgetManager.GetTugOfWarWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
