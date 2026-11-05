//! Host visualization feed for the four remaining widget getters.
//!
//! Each case supplies an empty set, then publishes typed records through
//! `WowLuaEnv::update_ui_widget` and reads the getter inside the real
//! `UPDATE_UI_WIDGET` callback. The expected tables hold every documented field
//! so a missing or extra projection fails the comparison. A second update
//! proves the getter returns fresh records instead of a cached table.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::c_api::ui_widget_manager::{
    CaptureZoneRecord, DoubleStateIconRowRecord, TextureWithAnimationRecord, WidgetSetEntry,
    WidgetSetRecord, WidgetStateIconRecord, WidgetVisualization, ZoneControlRecord,
    ZoneEntryRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const SET_ID: f64 = 7801.0;

fn assert_lua(env: &WowLuaEnv, context: &str, code: &str) {
    env.exec(code)
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
}

fn new_feed_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("widget feed environment");
    env.replace_ui_widget_set(
        SET_ID,
        Some(WidgetSetRecord {
            layout_direction: 0,
            vertical_padding: 3.0,
            widgets: Vec::new(),
        }),
    )
    .expect("supplied empty widget set");
    env
}

/// Registers a real event observer that snapshots membership and the getter
/// result at notification time.
fn install_observer(env: &WowLuaEnv, getter: &str, widget_type: u32) {
    assert_lua(
        env,
        "install UPDATE_UI_WIDGET observer",
        &format!(
            r#"
            FeedRows = {{}}
            FeedMembership = nil
            FeedObserver = CreateFrame('Frame')
            FeedObserver:RegisterEvent('UPDATE_UI_WIDGET')
            FeedObserver:SetScript('OnEvent', function(_, event, info)
                assert(event == 'UPDATE_UI_WIDGET')
                assert(info.widgetSetID == 7801 and info.widgetID == 780101
                    and info.widgetType == {widget_type})
                FeedMembership = C_UIWidgetManager.GetAllWidgetsBySetID(7801)
                FeedRows[#FeedRows + 1] = C_UIWidgetManager.{getter}(info.widgetID)
            end)
            "#
        ),
    );
}

fn entry(widget_type: u32) -> WidgetSetEntry {
    WidgetSetEntry {
        widget_id: 780101.0,
        widget_type,
        unit_token: None,
    }
}

const DEEP_EQUAL: &str = r#"
    function DeepEqual(left, right)
        if type(left) ~= type(right) then return false end
        if type(left) ~= 'table' then return left == right end
        for key, value in pairs(left) do
            if not DeepEqual(value, right[key]) then return false end
        end
        for key in pairs(right) do
            if left[key] == nil then return false end
        end
        return true
    end
"#;

fn capture_zone_record(tooltip: &str, current: f64) -> CaptureZoneRecord {
    CaptureZoneRecord {
        shown_state: 1,
        mode: 1,
        leading_edge_type: 1,
        danger_flash_type: 2,
        zone_info: ZoneEntryRecord {
            state: 1,
            active_state: 0,
            fill_type: 2,
            min: 0.0,
            max: 100.0,
            current,
            capture_point: 75.0,
            tooltip: tooltip.into(),
        },
        tooltip_loc: 4,
        widget_size_setting: 180.0,
        texture_kit: "capture-kit".into(),
        frame_texture_kit: "capture-frame".into(),
        has_timer: true,
        order_index: 10.0,
        widget_tag: "capture-zone".into(),
        in_anim_type: 1,
        out_anim_type: 2,
        widget_scale: 0,
        layout_direction: 1,
        model_scene_layer: 1,
        scripted_animation_effect_id: 73.0,
    }
}

#[test]
fn capture_zone_feed_commits_record_and_membership_before_notification() {
    let env = new_feed_env();
    assert_lua(
        &env,
        "absent capture zone",
        "assert(C_UIWidgetManager.GetCaptureZoneVisualizationInfo(780101) == nil)",
    );
    assert_lua(
        &env,
        "expected capture zone",
        &format!(
            r#"
            {DEEP_EQUAL}
            expectedCaptureZone = {{
                shownState = 1, mode = 1, leadingEdgeType = 1, dangerFlashType = 2,
                zoneInfo = {{ state = 1, activeState = 0, fillType = 2, min = 0, max = 100,
                    current = 40, capturePoint = 75, tooltip = 'Captured zone' }},
                tooltipLoc = 4, widgetSizeSetting = 180, textureKit = 'capture-kit',
                frameTextureKit = 'capture-frame', hasTimer = true, orderIndex = 10,
                widgetTag = 'capture-zone', inAnimType = 1, outAnimType = 2, widgetScale = 0,
                layoutDirection = 1, modelSceneLayer = 1, scriptedAnimationEffectID = 73,
            }}
            "#
        ),
    );
    install_observer(&env, "GetCaptureZoneVisualizationInfo", 17);
    env.update_ui_widget(
        SET_ID,
        entry(17),
        WidgetVisualization::CaptureZone(capture_zone_record("Captured zone", 40.0)),
    )
    .expect("capture zone feed");
    assert_lua(
        &env,
        "capture zone event snapshot",
        r#"
        assert(#FeedRows == 1)
        assert(DeepEqual(FeedRows[1], expectedCaptureZone))
        assert(#FeedMembership == 1 and FeedMembership[1].widgetID == 780101
            and FeedMembership[1].widgetType == 17)
        assert(C_UIWidgetManager.GetZoneControlVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetTextureWithAnimationVisualizationInfo(780101) == nil)
        "#,
    );
    env.update_ui_widget(
        SET_ID,
        entry(17),
        WidgetVisualization::CaptureZone(capture_zone_record("Updated zone", 60.0)),
    )
    .expect("capture zone replacement feed");
    assert_lua(
        &env,
        "capture zone fresh replacement",
        r#"
        assert(#FeedRows == 2 and FeedRows[1] ~= FeedRows[2])
        assert(FeedRows[1].zoneInfo.current == 40 and FeedRows[2].zoneInfo.current == 60)
        assert(FeedRows[2].zoneInfo.tooltip == 'Updated zone')
        assert(#FeedMembership == 1)
        "#,
    );
}

fn zone_control_record(tooltip: &str, current: f64) -> ZoneControlRecord {
    ZoneControlRecord {
        shown_state: 1,
        mode: 2,
        leading_edge_type: 0,
        danger_flash_type: 1,
        zone_entries: vec![ZoneEntryRecord {
            state: 0,
            active_state: 0,
            fill_type: 3,
            min: 10.0,
            max: 90.0,
            current,
            capture_point: 50.0,
            tooltip: tooltip.into(),
        }],
        tooltip_loc: 2,
        widget_size_setting: 200.0,
        texture_kit: "zone-kit".into(),
        frame_texture_kit: "zone-frame".into(),
        has_timer: false,
        order_index: 11.0,
        widget_tag: "zone-control".into(),
        in_anim_type: 0,
        out_anim_type: 3,
        widget_scale: 1,
        layout_direction: 2,
        model_scene_layer: 0,
        scripted_animation_effect_id: 91.0,
    }
}

#[test]
fn zone_control_feed_commits_record_and_membership_before_notification() {
    let env = new_feed_env();
    assert_lua(
        &env,
        "absent zone control",
        "assert(C_UIWidgetManager.GetZoneControlVisualizationInfo(780101) == nil)",
    );
    assert_lua(
        &env,
        "expected zone control",
        &format!(
            r#"
            {DEEP_EQUAL}
            expectedZoneControl = {{
                shownState = 1, mode = 2, leadingEdgeType = 0, dangerFlashType = 1,
                zoneEntries = {{
                    {{ state = 0, activeState = 0, fillType = 3, min = 10, max = 90,
                        current = 55, capturePoint = 50, tooltip = 'Supplied zone' }},
                }},
                tooltipLoc = 2, widgetSizeSetting = 200, textureKit = 'zone-kit',
                frameTextureKit = 'zone-frame', hasTimer = false, orderIndex = 11,
                widgetTag = 'zone-control', inAnimType = 0, outAnimType = 3, widgetScale = 1,
                layoutDirection = 2, modelSceneLayer = 0, scriptedAnimationEffectID = 91,
            }}
            "#
        ),
    );
    install_observer(&env, "GetZoneControlVisualizationInfo", 16);
    env.update_ui_widget(
        SET_ID,
        entry(16),
        WidgetVisualization::ZoneControl(zone_control_record("Supplied zone", 55.0)),
    )
    .expect("zone control feed");
    assert_lua(
        &env,
        "zone control event snapshot",
        r#"
        assert(#FeedRows == 1)
        assert(DeepEqual(FeedRows[1], expectedZoneControl))
        assert(#FeedMembership == 1 and FeedMembership[1].widgetType == 16)
        assert(C_UIWidgetManager.GetCaptureZoneVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetTextureWithAnimationVisualizationInfo(780101) == nil)
        "#,
    );
    env.update_ui_widget(
        SET_ID,
        entry(16),
        WidgetVisualization::ZoneControl(zone_control_record("Updated zone", 65.0)),
    )
    .expect("zone control replacement feed");
    assert_lua(
        &env,
        "zone control fresh replacement",
        r#"
        assert(#FeedRows == 2 and FeedRows[1] ~= FeedRows[2])
        assert(FeedRows[1].zoneEntries[1].current == 55
            and FeedRows[2].zoneEntries[1].current == 65)
        assert(FeedRows[2].zoneEntries[1].tooltip == 'Updated zone')
        assert(#FeedMembership == 1)
        "#,
    );
}

fn double_state_icon_record() -> DoubleStateIconRowRecord {
    DoubleStateIconRowRecord {
        shown_state: 1,
        left_icons: vec![WidgetStateIconRecord {
            icon_state: 1,
            state1_tooltip: "Left state one".into(),
            state2_tooltip: "Left state two".into(),
        }],
        right_icons: vec![WidgetStateIconRecord {
            icon_state: 2,
            state1_tooltip: "Right state one".into(),
            state2_tooltip: "Right state two".into(),
        }],
        tooltip_loc: 3,
        widget_size_setting: 160.0,
        texture_kit: "state-kit".into(),
        frame_texture_kit: "state-frame".into(),
        has_timer: true,
        order_index: 12.0,
        widget_tag: "double-state".into(),
        in_anim_type: 2,
        out_anim_type: 1,
        widget_scale: 2,
        layout_direction: 0,
        model_scene_layer: 2,
        scripted_animation_effect_id: 47.0,
    }
}

#[test]
fn double_state_icon_feed_commits_record_and_membership_before_notification() {
    let env = new_feed_env();
    assert_lua(
        &env,
        "absent double state icon row",
        "assert(C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo(780101) == nil)",
    );
    assert_lua(
        &env,
        "expected double state icon row",
        &format!(
            r#"
            {DEEP_EQUAL}
            expectedDoubleStateIconRow = {{
                shownState = 1,
                leftIcons = {{{{ iconState = 1, state1Tooltip = 'Left state one',
                    state2Tooltip = 'Left state two' }}}},
                rightIcons = {{{{ iconState = 2, state1Tooltip = 'Right state one',
                    state2Tooltip = 'Right state two' }}}},
                tooltipLoc = 3, widgetSizeSetting = 160, textureKit = 'state-kit',
                frameTextureKit = 'state-frame', hasTimer = true, orderIndex = 12,
                widgetTag = 'double-state', inAnimType = 2, outAnimType = 1, widgetScale = 2,
                layoutDirection = 0, modelSceneLayer = 2, scriptedAnimationEffectID = 47,
            }}
            "#
        ),
    );
    install_observer(&env, "GetDoubleStateIconRowVisualizationInfo", 14);
    env.update_ui_widget(
        SET_ID,
        entry(14),
        WidgetVisualization::DoubleStateIconRow(double_state_icon_record()),
    )
    .expect("double state icon feed");
    assert_lua(
        &env,
        "double state icon event snapshot",
        r#"
        assert(#FeedRows == 1)
        assert(DeepEqual(FeedRows[1], expectedDoubleStateIconRow))
        assert(#FeedMembership == 1 and FeedMembership[1].widgetType == 14)
        assert(C_UIWidgetManager.GetCaptureZoneVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetZoneControlVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetTextureWithAnimationVisualizationInfo(780101) == nil)
        "#,
    );
    let mut updated = double_state_icon_record();
    updated.left_icons[0].state1_tooltip = "Updated left state one".into();
    env.update_ui_widget(
        SET_ID,
        entry(14),
        WidgetVisualization::DoubleStateIconRow(updated),
    )
    .expect("double state icon replacement feed");
    assert_lua(
        &env,
        "double state icon fresh replacement",
        r#"
        assert(#FeedRows == 2 and FeedRows[1] ~= FeedRows[2])
        assert(FeedRows[1].leftIcons[1].state1Tooltip == 'Left state one')
        assert(FeedRows[2].leftIcons[1].state1Tooltip == 'Updated left state one')
        assert(FeedRows[2].rightIcons[1].state2Tooltip == 'Right state two')
        assert(#FeedMembership == 1)
        "#,
    );
}

fn texture_with_animation_record(tooltip: &str) -> TextureWithAnimationRecord {
    TextureWithAnimationRecord {
        shown_state: 1,
        tooltip: tooltip.into(),
        tooltip_loc: 1,
        widget_size_setting: 140.0,
        texture_kit: "animation-kit".into(),
        frame_texture_kit: "animation-frame".into(),
        has_timer: false,
        order_index: 13.0,
        widget_tag: "animated-texture".into(),
        in_anim_type: 3,
        out_anim_type: 0,
        widget_scale: 1,
        layout_direction: 1,
        model_scene_layer: 1,
        scripted_animation_effect_id: 12.0,
    }
}

#[test]
fn texture_with_animation_feed_commits_record_and_membership_before_notification() {
    let env = new_feed_env();
    assert_lua(
        &env,
        "absent texture with animation",
        "assert(C_UIWidgetManager.GetTextureWithAnimationVisualizationInfo(780101) == nil)",
    );
    assert_lua(
        &env,
        "expected texture with animation",
        &format!(
            r#"
            {DEEP_EQUAL}
            expectedTextureWithAnimation = {{
                shownState = 1, tooltip = 'Supplied animated texture', tooltipLoc = 1,
                widgetSizeSetting = 140, textureKit = 'animation-kit',
                frameTextureKit = 'animation-frame', hasTimer = false, orderIndex = 13,
                widgetTag = 'animated-texture', inAnimType = 3, outAnimType = 0, widgetScale = 1,
                layoutDirection = 1, modelSceneLayer = 1, scriptedAnimationEffectID = 12,
            }}
            "#
        ),
    );
    install_observer(&env, "GetTextureWithAnimationVisualizationInfo", 18);
    env.update_ui_widget(
        SET_ID,
        entry(18),
        WidgetVisualization::TextureWithAnimation(texture_with_animation_record(
            "Supplied animated texture",
        )),
    )
    .expect("texture with animation feed");
    assert_lua(
        &env,
        "texture with animation event snapshot",
        r#"
        assert(#FeedRows == 1)
        assert(DeepEqual(FeedRows[1], expectedTextureWithAnimation))
        assert(#FeedMembership == 1 and FeedMembership[1].widgetType == 18)
        assert(C_UIWidgetManager.GetCaptureZoneVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetZoneControlVisualizationInfo(780101) == nil)
        assert(C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo(780101) == nil)
        "#,
    );
    env.update_ui_widget(
        SET_ID,
        entry(18),
        WidgetVisualization::TextureWithAnimation(texture_with_animation_record(
            "Updated animated texture",
        )),
    )
    .expect("texture with animation replacement feed");
    assert_lua(
        &env,
        "texture with animation fresh replacement",
        r#"
        assert(#FeedRows == 2 and FeedRows[1] ~= FeedRows[2])
        assert(FeedRows[1].tooltip == 'Supplied animated texture')
        assert(FeedRows[2].tooltip == 'Updated animated texture')
        assert(#FeedMembership == 1)
        "#,
    );
}
