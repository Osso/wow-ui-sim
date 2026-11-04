//! Real cached Lua/XML widget lifecycle over explicitly supplied host records.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common;
use common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::c_api::ui_widget_manager::{
    ButtonHeaderRecord, IconAndTextRecord, WidgetId, WidgetSetEntry, WidgetSetRecord,
    WidgetSpellButtonRecord, WidgetVisualization,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const SET_ID: f64 = 417.0;
const ICON_ID: f64 = 760_101.0;
const HIDDEN_ICON_ID: f64 = 760_102.0;
const HEADER_ID: f64 = 760_103.0;
// Cached UIWidgetManagerSharedDocumentation.lua, not ID-derived visualization types.
const ICON_AND_TEXT: u32 = 0;
const BUTTON_HEADER: u32 = 30;
const SHOWN: u32 = 1;
const HIDDEN: u32 = 0;

fn icon_record(text: &str, state: u32) -> IconAndTextRecord {
    IconAndTextRecord {
        state,
        text: text.into(),
        tooltip: "Supplied icon tooltip".into(),
        dynamic_tooltip: "Supplied dynamic tooltip".into(),
        tooltip_loc: 3,     // TopLeft
        shift_text_type: 0, // NoShift
        widget_size_setting: 160.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: true,
        order_index: 10.0,
        widget_tag: "lifecycle-icon".into(),
        in_anim_type: 0, // None: removal is synchronous, not an animation/tick test.
        out_anim_type: 0,
        widget_scale: 0,      // OneHundred
        layout_direction: 1,  // Vertical
        model_scene_layer: 0, // None
        scripted_animation_effect_id: 0.0,
    }
}

fn spell_button(spell_id: f64, tooltip: &str) -> WidgetSpellButtonRecord {
    WidgetSpellButtonRecord {
        spell_id,
        tooltip: tooltip.into(),
        icon: 3,          // Checkmark
        enabled_state: 1, // Enabled
        cooldown_type: 0, // HideCooldown
    }
}

fn header_record() -> ButtonHeaderRecord {
    ButtonHeaderRecord {
        shown_state: SHOWN,
        header_text: "Supplied header".into(),
        tooltip: "Supplied header tooltip".into(),
        tooltip_loc: 3,
        buttons: vec![
            spell_button(190_001.0, "Supplied first button"),
            spell_button(190_002.0, "Supplied second button"),
        ],
        widget_size_setting: 180.0,
        texture_kit: String::new(),
        frame_texture_kit: "evergreen-scenario-widget-short".into(),
        has_timer: false,
        order_index: 20.0,
        widget_tag: "lifecycle-header".into(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 1,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

fn widget_entry(widget_id: f64, widget_type: u32) -> WidgetSetEntry {
    WidgetSetEntry {
        widget_id,
        widget_type,
        unit_token: None,
    }
}

fn supply_inputs(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let entries = [
        (ICON_ID, ICON_AND_TEXT),
        (HIDDEN_ICON_ID, ICON_AND_TEXT),
        (HEADER_ID, BUTTON_HEADER),
    ]
    .into_iter()
    .map(|(widget_id, widget_type)| widget_entry(widget_id, widget_type))
    .collect();
    state.ui_widget_inputs.icon_and_text.insert(
        WidgetId::from_number(ICON_ID),
        icon_record("Supplied icon text", SHOWN),
    );
    state.ui_widget_inputs.icon_and_text.insert(
        WidgetId::from_number(HIDDEN_ICON_ID),
        icon_record("Initially hidden text", HIDDEN),
    );
    // Preserve the separate existing ButtonHeaderInputs contract.
    state
        .button_header_inputs
        .widgets
        .insert(WidgetId::from_number(HEADER_ID), header_record());
    drop(state);
    env.replace_ui_widget_set(
        SET_ID,
        Some(WidgetSetRecord {
            layout_direction: 0, // UIWidgetSetLayoutDirection.Vertical
            vertical_padding: 7.0,
            widgets: entries,
        }),
    )
    .expect("supply initial membership through host feed");
}

fn fire_widget_update(env: &WowLuaEnv, set_id: f64, widget_id: f64, widget_type: u32) {
    let payload = env
        .eval::<rilua::Val>(&format!(
            "return {{widgetSetID={set_id}, widgetID={widget_id}, widgetType={widget_type}}}"
        ))
        .expect("create actual UPDATE_UI_WIDGET payload table");
    env.fire_event_with_args("UPDATE_UI_WIDGET", &[payload])
        .expect("dispatch UPDATE_UI_WIDGET through registered frame events");
}

fn assert_lua(env: &WowLuaEnv, phase: &str, assertions: &str) {
    env.exec(assertions)
        .unwrap_or_else(|error| panic!("cached widget lifecycle [{phase}]: {error}"));
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "[{phase}] recorded Lua failures: {errors:?}"
    );
}

fn register_container(env: &WowLuaEnv) {
    assert_lua(
        env,
        "real XML container registration and template acquisition",
        r#"
        assert(Enum.UIWidgetVisualizationType.IconAndText == 0)
        assert(Enum.UIWidgetVisualizationType.ButtonHeader == 30)
        assert(UIWidgetManager and UIWidgetManager:GetWidgetTypeInfo(0))
        assert(UIWidgetManager:GetWidgetTypeInfo(30))
        LifecycleContainer = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
        LifecycleContainer:SetPoint('CENTER', UIParent, 'CENTER')
        LifecycleContainer:RegisterForWidgetSet(417)
        local container = LifecycleContainer
        assert(container:IsRegisteredForWidgetSet(417), 'supplied set must register')
        assert(container.widgetSetLayoutDirection == Enum.UIWidgetSetLayoutDirection.Vertical)
        assert(container.verticalAnchorYOffset == 7)
        assert(container:IsEventRegistered('UPDATE_UI_WIDGET'))
        assert(container:IsEventRegistered('UPDATE_ALL_UI_WIDGETS'))
        assert(UIWidgetManager.registeredWidgetContainers[container])
        LifecycleIcon = assert(container.widgetFrames[760101], 'acquire IconAndText XML frame')
        LifecycleHeader = assert(container.widgetFrames[760103], 'acquire ButtonHeader XML frame')
        assert(LifecycleIcon.Text:GetText() == 'Supplied icon text')
        assert(LifecycleIcon.tooltip == 'Supplied icon tooltip')
        assert(LifecycleIcon.DynamicIconButton.tooltip == 'Supplied dynamic tooltip')
        assert(LifecycleIcon:GetWidth() == 160)
        assert(LifecycleIcon:IsShown() and LifecycleHeader:IsShown())
        assert(LifecycleHeader.HeaderText:GetText() == 'Supplied header')
        assert(LifecycleHeader.tooltip == 'Supplied header tooltip')
        local buttons = {}
        for button in LifecycleHeader.buttonPool:EnumerateActive() do
            assert(button:IsShown())
            buttons[button.layoutIndex] = button
        end
        assert(LifecycleHeader.buttonPool:GetNumActive() == 2)
        assert(buttons[1].spellID == 190001 and buttons[1].tooltip == 'Supplied first button')
        assert(buttons[2].spellID == 190002 and buttons[2].tooltip == 'Supplied second button')
        assert(container.widgetFrames[760102] == nil, 'actual provider filters hidden input')
        assert(container:GetNumWidgetsShowing() == 2)
        assert(container.widgetPools:GetNumActive() == 2)
        assert(container.numTimers == 1 and container.ticker)
        "#,
    );
}

fn update_supplied_records(env: &WowLuaEnv) -> (IconAndTextRecord, ButtonHeaderRecord) {
    let state = env.state().borrow();
    let mut icon = state
        .ui_widget_inputs
        .icon_and_text
        .get(&WidgetId::from_number(ICON_ID))
        .unwrap()
        .clone();
    icon.text = "Updated supplied icon".into();
    icon.tooltip = "Updated icon tooltip".into();
    icon.order_index = 30.0;
    let mut header = state
        .button_header_inputs
        .widgets
        .get(&WidgetId::from_number(HEADER_ID))
        .unwrap()
        .clone();
    header.header_text = "Updated supplied header".into();
    header.buttons = vec![spell_button(290_003.0, "Updated supplied button")];
    (icon, header)
}

#[test]
fn cached_ui_widget_container_refreshes_removes_and_unregisters_supplied_widgets() {
    common::with_timeout(88, || {
        wow_ui_sim::xml::register_intrinsic_templates();
        // Real TOC closure only. No manager/provider/template/pool/Setup overrides.
        // Implicit cached prerequisites are intentionally not claimed runtime-proven.
        with_blizzard_addon_closure(
            &[
                "Blizzard_SharedXMLGame",
                "Blizzard_Fonts_Shared",
                "Blizzard_FrameEffects",
                "Blizzard_UIWidgets",
            ],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_UIWidgets"));
                supply_inputs(env);
                register_container(env);
                let (icon, mut header) = update_supplied_records(env);

                fire_widget_update(env, SET_ID + 1.0, ICON_ID, ICON_AND_TEXT);
                fire_widget_update(env, SET_ID + 1.0, HEADER_ID, BUTTON_HEADER);
                assert_lua(
                    env,
                    "wrong-set events ignored",
                    r#"
                    assert(LifecycleIcon.Text:GetText() == 'Supplied icon text')
                    assert(LifecycleHeader.HeaderText:GetText() == 'Supplied header')
                    assert(LifecycleHeader.buttonPool:GetNumActive() == 2)
                "#,
                );

                env.update_ui_widget(
                    SET_ID,
                    widget_entry(ICON_ID, ICON_AND_TEXT),
                    WidgetVisualization::IconAndText(icon),
                )
                .expect("update icon snapshot and notify through host feed");
                env.update_ui_widget(
                    SET_ID,
                    widget_entry(HEADER_ID, BUTTON_HEADER),
                    WidgetVisualization::ButtonHeader(header.clone()),
                )
                .expect("update header snapshot and notify through host feed");
                assert_lua(
                    env,
                    "event refresh executes real Setup and nested pool",
                    r#"
                    local container = LifecycleContainer
                    assert(container.widgetFrames[760101] == LifecycleIcon)
                    assert(container.widgetFrames[760103] == LifecycleHeader)
                    assert(LifecycleIcon.Text:GetText() == 'Updated supplied icon')
                    assert(LifecycleIcon.tooltip == 'Updated icon tooltip')
                    assert(LifecycleIcon.orderIndex == 30)
                    assert(LifecycleHeader.HeaderText:GetText() == 'Updated supplied header')
                    assert(LifecycleHeader.buttonPool:GetNumActive() == 1)
                    for button in LifecycleHeader.buttonPool:EnumerateActive() do
                        assert(button.spellID == 290003 and button:IsShown())
                        assert(button.tooltip == 'Updated supplied button' and button.layoutIndex == 1)
                    end
                "#,
                );

                assert!(
                    env.remove_ui_widget(SET_ID, ICON_ID)
                        .expect("remove membership and reconcile through host feed")
                );
                assert_lua(
                    env,
                    "membership removal releases existing frame and timer",
                    r#"
                    assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(760101).text
                        == 'Updated supplied icon', 'record still exists; only membership removed')
                    assert(LifecycleContainer.widgetFrames[760101] == nil)
                    assert(not LifecycleIcon:IsShown(), 'removed pool frame is hidden')
                    assert(LifecycleContainer.widgetFrames[760103] == LifecycleHeader)
                    assert(LifecycleContainer:GetNumWidgetsShowing() == 1)
                    assert(LifecycleContainer.widgetPools:GetNumActive() == 1)
                    assert(LifecycleContainer.numTimers == 0 and LifecycleContainer.ticker == nil)
                "#,
                );

                header.shown_state = HIDDEN;
                env.update_ui_widget(
                    SET_ID,
                    widget_entry(HEADER_ID, BUTTON_HEADER),
                    WidgetVisualization::ButtonHeader(header.clone()),
                )
                .expect("supply hidden header through host feed");
                assert_lua(
                    env,
                    "hidden ButtonHeader provider removes existing frame",
                    r#"
                    assert(C_UIWidgetManager.GetButtonHeaderWidgetVisualizationInfo(760103).shownState
                        == Enum.WidgetShownState.Hidden)
                    assert(LifecycleContainer.widgetFrames[760103] == nil)
                    assert(not LifecycleHeader:IsShown())
                    assert(LifecycleContainer.widgetPools:GetNumActive() == 0)
                "#,
                );

                env.update_ui_widget(
                    SET_ID,
                    widget_entry(HIDDEN_ICON_ID, ICON_AND_TEXT),
                    WidgetVisualization::IconAndText(icon_record("Initially hidden text", SHOWN)),
                )
                .expect("activate hidden icon through host feed");
                assert_lua(
                    env,
                    "previously hidden member acquires real template",
                    r#"
                    LifecycleRestoredIcon = assert(LifecycleContainer.widgetFrames[760102])
                    assert(LifecycleRestoredIcon.Text:GetText() == 'Initially hidden text')
                    assert(LifecycleRestoredIcon:IsShown())
                    assert(LifecycleContainer.widgetPools:GetNumActive() == 1)
                    assert(LifecycleContainer.numTimers == 1 and LifecycleContainer.ticker)
                "#,
                );
                env.update_ui_widget(
                    SET_ID,
                    widget_entry(HIDDEN_ICON_ID, ICON_AND_TEXT),
                    WidgetVisualization::IconAndText(icon_record("Initially hidden text", HIDDEN)),
                )
                .expect("hide icon through host feed");
                assert_lua(
                    env,
                    "hidden IconAndText releases frame",
                    r#"
                    assert(LifecycleContainer.widgetFrames[760102] == nil)
                    assert(not LifecycleRestoredIcon:IsShown())
                    assert(LifecycleContainer.widgetPools:GetNumActive() == 0)
                "#,
                );

                env.update_ui_widget(
                    SET_ID,
                    widget_entry(HIDDEN_ICON_ID, ICON_AND_TEXT),
                    WidgetVisualization::IconAndText(icon_record("Initially hidden text", SHOWN)),
                )
                .expect("restore icon through host feed before cleanup");
                header.shown_state = SHOWN;
                env.update_ui_widget(
                    SET_ID,
                    widget_entry(HEADER_ID, BUTTON_HEADER),
                    WidgetVisualization::ButtonHeader(header),
                )
                .expect("restore header through host feed before cleanup");
                // Incremental events mark cached layout dirty; counts refresh on OnUpdate.
                env.fire_on_update(0.0)
                    .expect("flush actual cached container layout before count assertions");
                assert_lua(
                    env,
                    "unregister cleans active pools events manager and timer",
                    r#"
                    local container = LifecycleContainer
                    assert(container:GetNumWidgetsShowing() == 2, 'restore showing count=' .. tostring(container:GetNumWidgetsShowing()))
                    local icon, header = container.widgetFrames[760102], container.widgetFrames[760103]
                    assert(icon:IsShown() and header:IsShown(), 'restored children shown')
                    assert(container.numTimers == 1 and container.ticker, 'restore timers=' .. tostring(container.numTimers))
                    container:UnregisterForWidgetSet()
                    assert(not container:IsRegisteredForWidgetSet(), 'unregistered set=' .. tostring(container.widgetSetID))
                    assert(not container:IsShown(), 'unregistered container hidden')
                    assert(not icon:IsShown() and not header:IsShown(), 'released children hidden')
                    assert(next(container.widgetFrames) == nil, 'widget map cleared')
                    assert(container:GetNumWidgetsShowing() == 0, 'unregistered showing count')
                    assert(container.widgetPools:GetNumActive() == 0, 'widget pool released')
                    assert(container.horizontalRowContainerPool:GetNumActive() == 0, 'row pool released')
                    assert(container.numTimers == 0 and container.ticker == nil, 'unregistered timers=' .. tostring(container.numTimers) .. ', ticker=' .. tostring(container.ticker))
                    assert(next(container.timerWidgets) == nil, 'timer widget map cleared')
                    assert(not container:IsEventRegistered('UPDATE_UI_WIDGET'), 'incremental event removed')
                    assert(not container:IsEventRegistered('UPDATE_ALL_UI_WIDGETS'), 'full refresh event removed')
                    assert(UIWidgetManager.registeredWidgetContainers[container] == nil, 'manager registration removed')
                    for _ in UIWidgetManager:EnumerateWidgetsByWidgetTag('lifecycle-icon') do
                        error('unregistered container must not publish tagged widgets')
                    end
                "#,
                );
                fire_widget_update(env, SET_ID, HIDDEN_ICON_ID, ICON_AND_TEXT);
                env.fire_event("UPDATE_ALL_UI_WIDGETS")
                    .expect("post-unregister refresh ignored");
                assert_lua(
                    env,
                    "unregistered events cannot reacquire widgets",
                    r#"
                    assert(next(LifecycleContainer.widgetFrames) == nil)
                    assert(LifecycleContainer.widgetPools:GetNumActive() == 0)
                    assert(not LifecycleContainer:IsShown())
                "#,
                );
            },
        );
    });
}

#[test]
fn cached_ui_widget_icon_and_text_dynamic_states_toggle_acquired_child() {
    common::with_timeout(88, || {
        wow_ui_sim::xml::register_intrinsic_templates();
        with_blizzard_addon_closure(
            &[
                "Blizzard_SharedXMLGame",
                "Blizzard_Fonts_Shared",
                "Blizzard_FrameEffects",
                "Blizzard_UIWidgets",
            ],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_UIWidgets"));
                assert_lua(
                    env,
                    "concrete IconAndText state enum guard",
                    r#"
                    assert(Enum.IconAndTextWidgetState.Hidden == 0)
                    assert(Enum.IconAndTextWidgetState.Shown == 1)
                    assert(Enum.IconAndTextWidgetState.ShownWithDynamicIconFlashing == 2)
                    assert(Enum.IconAndTextWidgetState.ShownWithDynamicIconNotFlashing == 3)
                    "#,
                );
                supply_inputs(env);
                register_container(env);
                // Reuse the existing fixtures; release the header through the normal
                // feed so the final zero-active count isolates the IconAndText member.
                assert!(
                    env.remove_ui_widget(SET_ID, HEADER_ID)
                        .expect("remove header membership through host feed")
                );
                assert_lua(
                    env,
                    "ordinary shown member starts with hidden dynamic child",
                    r#"
                    local container = LifecycleContainer
                    assert(container:IsRegisteredForWidgetSet(417))
                    assert(container.widgetFrames[760101] == LifecycleIcon)
                    assert(LifecycleIcon:IsShown())
                    assert(not LifecycleIcon.DynamicIconButton:IsShown())
                    assert(container.widgetFrames[760103] == nil)
                    assert(not LifecycleHeader:IsShown())
                    assert(container.widgetPools:GetNumActive() == 1)
                    "#,
                );

                // Only child visibility is observed; flash animation/timing is not.
                for (state, phase, dynamic_shown) in [
                    (3, "dynamic icon not flashing", true),
                    (2, "dynamic icon flashing", true),
                    (SHOWN, "ordinary shown after dynamic states", false),
                ] {
                    env.update_ui_widget(
                        SET_ID,
                        widget_entry(ICON_ID, ICON_AND_TEXT),
                        WidgetVisualization::IconAndText(icon_record(phase, state)),
                    )
                    .expect("supply IconAndText state and notify through host feed");
                    assert_lua(
                        env,
                        phase,
                        &format!(
                            r#"
                            local container = LifecycleContainer
                            assert(container:IsRegisteredForWidgetSet(417))
                            local frame = assert(container.widgetFrames[760101])
                            assert(frame == LifecycleIcon, 'refresh the acquired frame')
                            assert(frame:IsShown())
                            assert(frame.Text:GetText() == '{phase}')
                            assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(760101).state == {state})
                            assert(frame.DynamicIconButton:IsShown() == {dynamic_shown})
                            assert(container.widgetPools:GetNumActive() == 1)
                            "#,
                        ),
                    );
                }

                env.update_ui_widget(
                    SET_ID,
                    widget_entry(ICON_ID, ICON_AND_TEXT),
                    WidgetVisualization::IconAndText(icon_record("Hidden dynamic member", HIDDEN)),
                )
                .expect("hide IconAndText member through host feed");
                assert_lua(
                    env,
                    "hidden member releases the acquired frame",
                    r#"
                    local container = LifecycleContainer
                    assert(container:IsRegisteredForWidgetSet(417))
                    assert(C_UIWidgetManager.GetIconAndTextWidgetVisualizationInfo(760101).state == 0)
                    assert(container.widgetFrames[760101] == nil)
                    assert(not LifecycleIcon:IsShown(), 'released pool frame is hidden')
                    assert(container.widgetPools:GetNumActive() == 0)
                    container:UnregisterForWidgetSet()
                    assert(not container:IsRegisteredForWidgetSet())
                    "#,
                );
            },
        );
    });
}

#[test]
fn cached_ui_widget_container_filters_supplied_unit_updates() {
    common::with_timeout(88, || {
        wow_ui_sim::xml::register_intrinsic_templates();
        with_blizzard_addon_closure(
            &[
                "Blizzard_SharedXMLGame",
                "Blizzard_Fonts_Shared",
                "Blizzard_FrameEffects",
                "Blizzard_UIWidgets",
            ],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_UIWidgets"));
                supply_inputs(env);
                for (unit, text) in [
                    ("player", "Supplied icon text"),
                    ("target", "Target supplied icon text"),
                ] {
                    let mut entry = widget_entry(ICON_ID, ICON_AND_TEXT);
                    entry.unit_token = Some(unit.into());
                    env.update_ui_widget(
                        SET_ID,
                        entry,
                        WidgetVisualization::IconAndText(icon_record(text, SHOWN)),
                    )
                    .expect("supply distinct unit records before container registration");
                }
                assert_lua(
                    env,
                    "two real containers register their attached units",
                    r#"
                    UnitFilterPlayer = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
                    UnitFilterTarget = CreateFrame('Frame', nil, UIParent, 'UIWidgetContainerTemplate')
                    UnitFilterPlayer:RegisterForWidgetSet(417, nil, nil, 'player')
                    UnitFilterTarget:RegisterForWidgetSet(417, nil, nil, 'target')
                    assert(UnitFilterPlayer.widgetFrames[760101].Text:GetText() == 'Supplied icon text')
                    assert(UnitFilterTarget.widgetFrames[760101].Text:GetText() == 'Target supplied icon text')
                    "#,
                );
                let mut icon = env
                    .state()
                    .borrow()
                    .ui_widget_inputs
                    .icon_and_text
                    .get(&WidgetId::from_number(ICON_ID))
                    .expect("initial supplied icon record")
                    .clone();
                icon.text = "Player-only supplied icon text".into();
                let mut entry = widget_entry(ICON_ID, ICON_AND_TEXT);
                entry.unit_token = Some("player".into());
                env.update_ui_widget(SET_ID, entry, WidgetVisualization::IconAndText(icon))
                    .expect("update supplied player icon through host feed");
                assert_lua(
                    env,
                    "real OnEvent selects each container's supplied unit record",
                    r#"
                    local playerText = UnitFilterPlayer.widgetFrames[760101].Text:GetText()
                    local targetText = UnitFilterTarget.widgetFrames[760101].Text:GetText()
                    UnitFilterPlayer:UnregisterForWidgetSet()
                    UnitFilterTarget:UnregisterForWidgetSet()
                    assert(not UnitFilterPlayer:IsRegisteredForWidgetSet())
                    assert(not UnitFilterTarget:IsRegisteredForWidgetSet())
                    assert(playerText == 'Player-only supplied icon text', 'player widget must refresh')
                    assert(targetText == 'Target supplied icon text', 'target widget must retain its own supplied text')
                    "#,
                );
            },
        );
    });
}
