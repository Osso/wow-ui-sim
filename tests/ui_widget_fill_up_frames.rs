//! Original fill-up frames Setup over supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{FillUpFramesRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_fill_up_frames_setup_consumes_supplied_values() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.fill_up_frames.insert(WidgetId::from_number(762624.0), FillUpFramesRecord {
            shown_state: 1, fill_min: 0.0, fill_max: 100.0, fill_value: 45.0,
            num_total_frames: 3.0, num_full_frames: 1.0, pulse_filling_frame: false,
            tooltip: "Supplied vigor".into(), tooltip_loc: 4, widget_size_setting: 0.0,
            texture_kit: "dragonriding_vigor".into(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "fill-up".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(435.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762624.0, widget_type: 24, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.FillUpFrames == 24)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(435)
        local widget = assert(container.widgetFrames[762624], 'fill-up frames were not created')
        assert(widget.fillUpFramePool:GetNumActive() == 3 and widget.lastNumFullFrames == 1)
        assert(widget.tooltip == 'Supplied vigor')
        assert(C_UIWidgetManager.GetFillUpFramesWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
