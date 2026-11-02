//! Cached StackedResourceTracker template consumes supplied records.
//! Retail/PTR UIWidgetManagerDocumentation.lua:368-379, 1767-1786; UIWidgetManagerSharedDocumentation.lua:141.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::ui_widget_manager::{
    StackedResourceTrackerRecord, UiWidgetCurrencyRecord, WidgetId, WidgetSetEntry, WidgetSetRecord,
};

fn resource(text: &str, icon_file_id: u32) -> UiWidgetCurrencyRecord {
    UiWidgetCurrencyRecord {
        icon_file_id,
        leading_text: String::new(),
        text: text.into(),
        tooltip: format!("{text} tooltip"),
        is_currency_maxed: false,
        text_font_type: 0,
        text_size_type: 1,
        text_enabled_state: 1,
        icon_size_type: 0,
        update_anim_type: 0,
    }
}

#[test]
fn cached_stacked_resource_setup_consumes_supplied_rows() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.stacked_resources.insert(
            WidgetId::from_number(762601.0),
            StackedResourceTrackerRecord {
                shown_state: 1,
                resources: vec![resource("First resource", 135891), resource("Second resource", 135892)],
                tooltip_loc: 4,
                widget_size_setting: 180.0,
                texture_kit: String::new(),
                frame_texture_kit: String::new(),
                has_timer: false,
                order_index: 10.0,
                widget_tag: "stacked-resource".into(),
                in_anim_type: 0,
                out_anim_type: 0,
                widget_scale: 0,
                layout_direction: 1,
                model_scene_layer: 0,
                scripted_animation_effect_id: 0.0,
            },
        );
        state.ui_widget_inputs.sets.insert(
            WidgetId::from_number(426.0),
            WidgetSetRecord {
                layout_direction: 0,
                vertical_padding: 0.0,
                widgets: vec![WidgetSetEntry {
                    widget_id: 762601.0,
                    widget_type: 6,
                    unit_token: None,
                }],
            },
        );
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(
        &env,
        &cache,
        &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"],
        &[],
    );
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.StackedResourceTracker == 6)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(426)
        local widget = assert(container.widgetFrames[762601], 'stacked resource was not created')
        assert(widget.resourcePool:GetNumActive() == 2)
        local found = {}
        for frame in widget.resourcePool:EnumerateActive() do
            found[frame.Text:GetText()] = frame.tooltip
        end
        assert(found['First resource'] == 'First resource tooltip')
        assert(found['Second resource'] == 'Second resource tooltip')
        assert(C_UIWidgetManager.GetStackedResourceTrackerWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
