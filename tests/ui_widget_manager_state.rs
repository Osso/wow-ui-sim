#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::c_api::ui_widget_manager::{
    IconAndTextRecord, UiWidgetInputs, WidgetId, WidgetSetEntry, WidgetSetRecord,
    WidgetVisualization,
};
use wow_ui_sim::lua_api::WowLuaEnv;

// All IDs and records in this file are synthetic supplied inputs, not native fixtures.
fn supplied_set() -> WidgetSetRecord {
    WidgetSetRecord {
        layout_direction: 1,
        vertical_padding: 7.5,
        widgets: vec![
            WidgetSetEntry {
                widget_id: 923.0,
                widget_type: 4,
                unit_token: Some("target".into()),
            },
            WidgetSetEntry {
                widget_id: 812.0,
                widget_type: 0,
                unit_token: Some("player".into()),
            },
            WidgetSetEntry {
                widget_id: 701.0,
                widget_type: 9,
                unit_token: None,
            },
        ],
    }
}

fn supplied_icon() -> IconAndTextRecord {
    IconAndTextRecord {
        state: 2,
        text: "Supplied text".into(),
        tooltip: "Supplied tooltip".into(),
        dynamic_tooltip: "Dynamic detail".into(),
        tooltip_loc: 3,
        shift_text_type: 1,
        widget_size_setting: 6.5,
        texture_kit: "SyntheticIcon".into(),
        frame_texture_kit: "SyntheticFrame".into(),
        has_timer: true,
        order_index: 12.5,
        widget_tag: "SyntheticTag".into(),
        in_anim_type: 2,
        out_anim_type: 3,
        widget_scale: 1,
        layout_direction: 2,
        model_scene_layer: 1,
        scripted_animation_effect_id: 73.0,
    }
}

fn supplied_env(inputs: UiWidgetInputs) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().ui_widget_inputs = inputs;
    env.exec(
        r#"
        function AssertWidgetFields(actual, expected)
            assert(type(actual) == 'table', 'expected query table')
            for key, value in pairs(expected) do
                assert(actual[key] == value, 'wrong field: ' .. key)
                assert(type(actual[key]) == type(value), 'wrong type: ' .. key)
            end
            for key in pairs(actual) do
                assert(expected[key] ~= nil, 'unexpected field: ' .. tostring(key))
            end
        end
        "#,
    )
    .unwrap();
    env
}

#[test]
fn widget_set_info_returns_exact_supplied_fields_for_nonthreshold_id() {
    let mut inputs = UiWidgetInputs::default();
    inputs
        .sets
        .insert(WidgetId::from_number(417.0), supplied_set());
    let env = supplied_env(inputs);
    env.exec(
        r#"
        AssertWidgetFields(C_UIWidgetManager.GetWidgetSetInfo(417), {
            layoutDirection = 1, verticalPadding = 7.5,
        })
        assert(C_UIWidgetManager.GetWidgetSetInfo(418) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn widgets_by_set_preserve_supplied_order_and_documented_entry_fields() {
    let mut inputs = UiWidgetInputs::default();
    inputs
        .sets
        .insert(WidgetId::from_number(417.0), supplied_set());
    let env = supplied_env(inputs);
    env.exec(
        r#"
        local widgets = C_UIWidgetManager.GetAllWidgetsBySetID(417)
        assert(type(widgets) == 'table' and #widgets == 3)
        AssertWidgetFields(widgets[1], {
            widgetID = 923, widgetSetID = 417, widgetType = 4, unitToken = 'target',
        })
        AssertWidgetFields(widgets[2], {
            widgetID = 812, widgetSetID = 417, widgetType = 0, unitToken = 'player',
        })
        AssertWidgetFields(widgets[3], {
            widgetID = 701, widgetSetID = 417, widgetType = 9,
        })
        assert(widgets[3].unitToken == nil)
        local missing = C_UIWidgetManager.GetAllWidgetsBySetID(418)
        assert(type(missing) == 'table' and next(missing) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn absent_high_ids_do_not_invent_widget_sets_or_icons() {
    let env = supplied_env(UiWidgetInputs::default());
    env.exec(
        r#"
        for _, id in ipairs({417, 1000000, 1000417, 2000000}) do
            assert(C_UIWidgetManager.GetWidgetSetInfo(id) == nil)
            local widgets = C_UIWidgetManager.GetAllWidgetsBySetID(id)
            assert(type(widgets) == 'table' and next(widgets) == nil)
            assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(id) == nil)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn icon_and_text_returns_complete_fresh_isolated_supplied_snapshots() {
    let mut inputs = UiWidgetInputs::default();
    let record = supplied_icon();
    inputs
        .icon_and_text
        .insert(WidgetId::from_number(812.0), record.clone());
    let env = supplied_env(inputs);
    env.exec(
        r#"
        expectedIcon = {
            state = 2, text = 'Supplied text', tooltip = 'Supplied tooltip',
            dynamicTooltip = 'Dynamic detail', tooltipLoc = 3, shiftTextType = 1,
            widgetSizeSetting = 6.5, textureKit = 'SyntheticIcon',
            frameTextureKit = 'SyntheticFrame', hasTimer = true, orderIndex = 12.5,
            widgetTag = 'SyntheticTag', inAnimType = 2, outAnimType = 3,
            widgetScale = 1, layoutDirection = 2, modelSceneLayer = 1,
            scriptedAnimationEffectID = 73,
        }
        firstIcon = C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(812)
        AssertWidgetFields(firstIcon, expectedIcon)
        firstIcon.text = 'Caller mutation'
        firstIcon.hasTimer = false
        firstIcon.extra = 'Caller-only field'
        secondIcon = C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(812)
        assert(firstIcon ~= secondIcon)
        AssertWidgetFields(secondIcon, expectedIcon)
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().ui_widget_inputs.icon_and_text[&WidgetId::from_number(812.0)],
        record
    );
    let mut updated = record;
    updated.text = "Updated supplied text".into();
    updated.order_index = 21.5;
    env.state()
        .borrow_mut()
        .ui_widget_inputs
        .icon_and_text
        .insert(WidgetId::from_number(812.0), updated);
    env.exec(
        r#"
        AssertWidgetFields(secondIcon, expectedIcon)
        expectedIcon.text = 'Updated supplied text'
        expectedIcon.orderIndex = 21.5
        local third = C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(812)
        assert(third ~= secondIcon)
        AssertWidgetFields(third, expectedIcon)
        assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(813) == nil)
        "#,
    )
    .unwrap();
    let other = supplied_env(UiWidgetInputs::default());
    other
        .exec("assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(812) == nil)")
        .unwrap();
}

#[test]
fn host_widget_feed_publishes_state_before_events_and_rejects_invalid_updates() {
    let env = supplied_env(UiWidgetInputs::default());
    env.exec(r#"
        feedAll, feedUpdates = 0, 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('UPDATE_ALL_UI_WIDGETS')
        listener:RegisterEvent('UPDATE_UI_WIDGET')
        listener:SetScript('OnEvent', function(_, event, info)
            if event == 'UPDATE_ALL_UI_WIDGETS' then
                feedAll = feedAll + 1
                assert(C_UIWidgetManager.GetWidgetSetInfo(417).verticalPadding == 7.5)
            else
                feedUpdates = feedUpdates + 1
                assert(info.widgetSetID == 417 and info.widgetID == 812 and info.widgetType == 0)
                assert(info.unitToken == nil and info.unit == nil)
                assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(812).text == 'Supplied text')
                assert(#C_UIWidgetManager.GetAllWidgetsBySetID(417) == 1)
            end
        end)
    "#).unwrap();
    let mut set = supplied_set();
    set.widgets.clear();
    env.replace_ui_widget_set(417.0, Some(set)).unwrap();
    let entry = WidgetSetEntry {
        widget_id: 812.0,
        widget_type: 0,
        unit_token: None,
    };
    env.update_ui_widget(
        417.0,
        entry.clone(),
        WidgetVisualization::IconAndText(supplied_icon()),
    )
    .unwrap();
    let mut wrong_type = entry.clone();
    wrong_type.widget_type = 30;
    assert!(
        env.update_ui_widget(
            417.0,
            wrong_type,
            WidgetVisualization::IconAndText(supplied_icon())
        )
        .is_err()
    );
    assert!(
        env.update_ui_widget(
            418.0,
            entry,
            WidgetVisualization::IconAndText(supplied_icon())
        )
        .is_err()
    );
    assert!(!env.remove_ui_widget(417.0, 999.0).unwrap());
    env.exec("assert(feedAll == 1 and feedUpdates == 1); assert(#C_UIWidgetManager.GetAllWidgetsBySetID(417) == 1)").unwrap();
    assert!(env.remove_ui_widget(417.0, 812.0).unwrap());
    env.exec(
        r#"
        assert(feedAll == 2 and feedUpdates == 1)
        assert(#C_UIWidgetManager.GetAllWidgetsBySetID(417) == 0)
        assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(812).text == 'Supplied text')
    "#,
    )
    .unwrap();
}

#[test]
fn quest_widget_sets_use_exact_supplied_quest_and_type_mapping() {
    let mut inputs = UiWidgetInputs::default();
    inputs.quest_sets = [((731, 1), 417.0), ((731, 2), 509.5), ((732, 1), 618.0)].into();
    let env = supplied_env(inputs);
    env.exec(
        r#"
        local function CheckMapping(quest, kind, expected)
            local actual = C_TaskQuest.GetQuestUIWidgetSetByType(quest, kind)
            assert(type(actual) == 'number' and actual == expected)
        end
        CheckMapping(731, 1, 417)
        CheckMapping(731, 2, 509.5)
        CheckMapping(732, 1, 618)
        assert(C_TaskQuest.GetQuestUIWidgetSetByType(731, 3) == nil)
        assert(C_TaskQuest.GetQuestUIWidgetSetByType(733, 1) == nil)
        "#,
    )
    .unwrap();
}
