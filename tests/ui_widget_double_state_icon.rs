//! Original double-state icon row Setup over supplied records.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{DoubleStateIconRowRecord, WidgetStateIconRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_double_state_icon_setup_consumes_supplied_icons() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.double_state_icons.insert(WidgetId::from_number(762614.0), DoubleStateIconRowRecord {
            shown_state: 1,
            left_icons: vec![WidgetStateIconRecord { icon_state: 1, state1_tooltip: "Left state one".into(), state2_tooltip: "Left state two".into() }],
            right_icons: vec![WidgetStateIconRecord { icon_state: 2, state1_tooltip: "Right state one".into(), state2_tooltip: "Right state two".into() }],
            tooltip_loc: 4, widget_size_setting: 180.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "double-state".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(428.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762614.0, widget_type: 14, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.DoubleStateIconRow == 14)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(428)
        local widget = assert(container.widgetFrames[762614], 'double state icon row was not created')
        assert(widget.iconPool:GetNumActive() == 2)
        local found = {}
        for frame in widget.iconPool:EnumerateActive() do found[frame.tooltip] = true end
        assert(found['Left state one'] and found['Right state two'])
        assert(C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
