//! Original scenario-header delves Setup over supplied data.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{load_blizzard_addon_closure_into_env, new_blizzard_addon_env};
use crate::ui_widget_stacked_resource::resource;
use wow_ui_sim::c_api::ui_widget_manager::{ScenarioHeaderDelvesRecord, WidgetRewardRecord, WidgetId, WidgetSetEntry, WidgetSetRecord};

#[test]
fn cached_scenario_delves_setup_consumes_supplied_values() {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    {
        let mut state = env.state().borrow_mut();
        state.ui_widget_inputs.scenario_delves.insert(WidgetId::from_number(762629.0), ScenarioHeaderDelvesRecord {
            shown_state: 1, header_text: "Supplied delve".into(), tooltip: "Delve tooltip".into(), tooltip_loc: 4,
            tier_text: "Tier 7".into(), tier_tooltip_spell_id: Some(35395.0),
            currencies: vec![resource("Delve currency", 135891)], spells: vec![],
            reward_info: WidgetRewardRecord { shown_state: 0, earned_tooltip: "Earned".into(), unearned_tooltip: "Unearned".into() },
            widget_size_setting: 180.0, texture_kit: String::new(), frame_texture_kit: String::new(),
            has_timer: false, order_index: 10.0, widget_tag: "delves".into(), in_anim_type: 0,
            out_anim_type: 0, widget_scale: 0, layout_direction: 1, model_scene_layer: 0, scripted_animation_effect_id: 0.0,
        });
        state.ui_widget_inputs.sets.insert(WidgetId::from_number(438.0), WidgetSetRecord {
            layout_direction: 0, vertical_padding: 0.0,
            widgets: vec![WidgetSetEntry { widget_id: 762629.0, widget_type: 29, unit_token: None }],
        });
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLGame", "Blizzard_Fonts_Shared", "Blizzard_FrameEffects", "Blizzard_UIWidgets"], &[]);
    env.exec(r#"
        assert(Enum.UIWidgetVisualizationType.ScenarioHeaderDelves == 29)
        local container = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        container:RegisterForWidgetSet(438)
        local widget = assert(container.widgetFrames[762629], 'delves header was not created')
        assert(widget.HeaderText:GetText() == 'Supplied delve')
        assert(widget.TierFrame.Text:GetText() == 'Tier 7' and widget.TierFrame.tooltipSpellID == 35395)
        assert(widget.currencyPool:GetNumActive() == 1 and widget.spellPool:GetNumActive() == 0)
        assert(not widget.RewardFrame:IsShown())
        assert(C_UIWidgetManager.GetScenarioHeaderDelvesWidgetVisualizationInfo(999999) == nil)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
