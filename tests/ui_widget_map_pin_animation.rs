//! Original map-pin animation Setup over supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use wow_ui_sim::c_api::ui_widget_manager::{MapPinAnimationRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_map_pin_animation_setup_consumes_supplied_padding() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.map_pin_animations.insert(WidgetId::from_number(762626.0), MapPinAnimationRecord {
            shown_state: 1, anim_type: 0, widget_size_setting: 5.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "map-pin".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(437.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762626.0, widget_type: 26, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.MapPinAnimation == 26)
        local pin = CreateFrame('Frame', nil, UIParent)
        pin:SetSize(30, 40)
        local container = CreateFrame('Frame', nil, pin, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(437)
        local widget = assert(container.widgetFrames[762626], 'map pin animation was not created')
        assert(widget.Background:GetWidth() == 51 and widget.Background:GetHeight() == 61)
        assert(not widget.MapPinTextureCopy:IsShown())
        assert(C_UIWidgetManager.GetMapPinAnimationWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
