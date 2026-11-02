//! Original capture-zone Setup over a supplied zone entry.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{CaptureZoneRecord, ZoneEntryRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_capture_zone_setup_consumes_supplied_zone() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.capture_zones.insert(WidgetId::from_number(762617.0), CaptureZoneRecord {
            shown_state: 1, mode: 0, leading_edge_type: 0, danger_flash_type: 0,
            zone_info: ZoneEntryRecord { state: 0, active_state: 1, fill_type: 0, min: 0.0, max: 100.0, current: 40.0, capture_point: 75.0, tooltip: "Captured zone".into() },
            tooltip_loc: 4, widget_size_setting: 180.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "capture-zone".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(430.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762617.0, widget_type: 17, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.CaptureZone == 17)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(430)
        local widget = assert(container.widgetFrames[762617], 'capture zone was not created')
        assert(widget.lastVals.current == 40 and widget.lastVals.capturePoint == 75)
        assert(widget.Zone.tooltip == 'Captured zone')
        assert(C_UIWidgetManager.GetCaptureZoneVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
