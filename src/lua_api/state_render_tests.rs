use super::*;
use crate::widget::{Frame, FrameStrata, WidgetType};

fn test_frame(id: u64, widget_type: WidgetType, parent_id: Option<u64>, visible: bool) -> Frame {
    let mut frame = Frame {
        id,
        widget_type,
        parent_id,
        visible,
        width: 10.0,
        height: 10.0,
        layout_rect: Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        }),
        ..Default::default()
    };
    frame.effective_alpha = if visible { 1.0 } else { 0.0 };
    frame
}

fn register_child(
    state: &mut SimState,
    id: u64,
    widget_type: WidgetType,
    parent_id: u64,
    visible: bool,
) {
    state
        .widgets
        .register(test_frame(id, widget_type, Some(parent_id), visible));
    state.widgets.add_child(parent_id, id);
}

fn medium_bucket(state: &mut SimState) -> Vec<u64> {
    state
        .get_strata_buckets()
        .unwrap()
        .get(FrameStrata::Medium.as_index())
        .cloned()
        .unwrap_or_default()
}

#[test]
fn raised_toplevel_root_keeps_cross_strata_descendants_in_one_segment() {
    let mut state = SimState::default();

    let mut ui_parent = test_frame(1, WidgetType::Frame, None, true);
    ui_parent.name = Some("UIParent".to_string());
    state.widgets.register(ui_parent);

    let mut panel_root = test_frame(10, WidgetType::Frame, Some(1), true);
    panel_root.name = Some("PanelRoot".to_string());
    panel_root.frame_level = 1;
    state.widgets.register(panel_root);
    state.widgets.add_child(1, 10);
    state.set_frame_toplevel(10, true);
    state.raise_frame(10);

    let mut low_wrapper = test_frame(11, WidgetType::Frame, Some(10), true);
    low_wrapper.frame_strata = FrameStrata::Low;
    state.widgets.register(low_wrapper);
    state.widgets.add_child(10, 11);

    let mut lower_descendant = test_frame(12, WidgetType::Frame, Some(11), true);
    lower_descendant.frame_level = 1;
    state.widgets.register(lower_descendant);
    state.widgets.add_child(11, 12);

    let mut higher_descendant = test_frame(13, WidgetType::Frame, Some(11), true);
    higher_descendant.frame_level = 20;
    state.widgets.register(higher_descendant);
    state.widgets.add_child(11, 13);

    let mut independent_root = test_frame(20, WidgetType::Frame, None, true);
    independent_root.frame_level = 7;
    state.widgets.register(independent_root);

    assert_eq!(medium_bucket(&mut state), vec![20, 11, 10, 12, 13]);
}

#[test]
fn explicit_raise_does_not_move_lower_level_frame_above_higher_level_frame() {
    let mut state = SimState::default();

    let mut lower = test_frame(30, WidgetType::Frame, None, true);
    lower.frame_level = 1;
    state.widgets.register(lower);

    let mut higher = test_frame(31, WidgetType::Frame, None, true);
    higher.frame_level = 7;
    state.widgets.register(higher);

    state.raise_frame(30);

    assert_eq!(medium_bucket(&mut state), vec![30, 31]);
}

#[test]
fn repeated_toplevel_hide_show_moves_latest_panel_segment_to_top() {
    let mut state = SimState::default();

    let mut ui_parent = test_frame(1, WidgetType::Frame, None, true);
    ui_parent.name = Some("UIParent".to_string());
    state.widgets.register(ui_parent);

    let mut first_panel = test_frame(40, WidgetType::Frame, Some(1), true);
    first_panel.frame_level = 1;
    state.widgets.register(first_panel);
    state.widgets.add_child(1, 40);
    state.set_frame_toplevel(40, true);
    state.raise_frame(40);

    let mut second_panel = test_frame(41, WidgetType::Frame, Some(1), true);
    second_panel.frame_level = 1;
    state.widgets.register(second_panel);
    state.widgets.add_child(1, 41);
    state.set_frame_toplevel(41, true);
    state.raise_frame(41);

    let mut regular = test_frame(42, WidgetType::Frame, Some(1), true);
    regular.frame_level = 7;
    state.widgets.register(regular);
    state.widgets.add_child(1, 42);

    assert_eq!(medium_bucket(&mut state), vec![42, 40, 41]);

    state.set_frame_visible(40, false);
    state.set_frame_visible(40, true);

    assert_eq!(medium_bucket(&mut state), vec![42, 41, 40]);
}

#[test]
fn nearest_active_toplevel_ancestor_owns_nested_segment() {
    let mut state = SimState::default();

    let mut ui_parent = test_frame(1, WidgetType::Frame, None, true);
    ui_parent.name = Some("UIParent".to_string());
    state.widgets.register(ui_parent);

    let outer = test_frame(50, WidgetType::Frame, Some(1), true);
    state.widgets.register(outer);
    state.widgets.add_child(1, 50);

    let mut low_wrapper = test_frame(51, WidgetType::Frame, Some(50), true);
    low_wrapper.frame_strata = FrameStrata::Low;
    state.widgets.register(low_wrapper);
    state.widgets.add_child(50, 51);

    let inner = test_frame(52, WidgetType::Frame, Some(51), true);
    state.widgets.register(inner);
    state.widgets.add_child(51, 52);
    state.set_frame_toplevel(52, true);
    state.raise_frame(52);

    register_child(&mut state, 53, WidgetType::Frame, 52, true);
    state.set_frame_toplevel(50, true);
    state.raise_frame(50);

    let mut regular = test_frame(54, WidgetType::Frame, Some(1), true);
    regular.frame_level = 7;
    state.widgets.register(regular);
    state.widgets.add_child(1, 54);

    assert_eq!(medium_bucket(&mut state), vec![54, 52, 53, 51, 50]);
}

#[test]
fn show_visible_region_repairs_parent_subtree_without_invalidating_buckets() {
    let mut state = SimState::default();
    state
        .widgets
        .register(test_frame(1, WidgetType::Frame, None, true));
    register_child(&mut state, 2, WidgetType::Texture, 1, true);
    register_child(&mut state, 3, WidgetType::Texture, 1, false);
    register_child(&mut state, 4, WidgetType::Frame, 1, true);
    register_child(&mut state, 5, WidgetType::Texture, 4, true);
    register_child(&mut state, 6, WidgetType::FontString, 1, true);

    assert_eq!(medium_bucket(&mut state), vec![1, 2, 4, 5, 6]);
    assert!(state.strata_buckets.is_some());

    state.set_frame_visible(3, true);

    assert!(state.strata_buckets.is_some());
    assert_eq!(medium_bucket(&mut state), vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn show_visible_child_frame_repairs_parent_subtree_without_invalidating_buckets() {
    let mut state = SimState::default();
    state
        .widgets
        .register(test_frame(10, WidgetType::Frame, None, true));
    register_child(&mut state, 11, WidgetType::Texture, 10, true);
    register_child(&mut state, 12, WidgetType::Frame, 10, false);
    register_child(&mut state, 13, WidgetType::Texture, 12, true);
    register_child(&mut state, 14, WidgetType::FontString, 10, true);

    assert_eq!(medium_bucket(&mut state), vec![10, 11, 14]);
    assert!(state.strata_buckets.is_some());

    state.set_frame_visible(12, true);

    assert!(state.strata_buckets.is_some());
    assert_eq!(medium_bucket(&mut state), vec![10, 11, 12, 13, 14]);
}

#[test]
fn show_root_frame_still_falls_back_to_full_invalidation() {
    let mut state = SimState::default();
    state
        .widgets
        .register(test_frame(20, WidgetType::Frame, None, false));
    let _ = medium_bucket(&mut state);

    state.set_frame_visible(20, true);

    assert!(state.strata_buckets.is_none());
}

#[test]
fn show_tooltip_root_appends_without_invalidating_buckets() {
    let mut state = SimState::default();
    state
        .widgets
        .register(test_frame(30, WidgetType::Frame, None, true));

    let mut tooltip = test_frame(31, WidgetType::GameTooltip, Some(30), false);
    tooltip.frame_strata = FrameStrata::Tooltip;
    state.widgets.register(tooltip);
    state.widgets.add_child(30, 31);

    let _ = state.get_strata_buckets();
    assert!(state.strata_buckets.is_some());

    state.set_frame_visible(31, true);

    assert!(
        state.strata_buckets.is_some(),
        "showing a tooltip root should append to the tooltip bucket without full invalidation"
    );
    let tooltip_bucket = state
        .strata_buckets
        .as_ref()
        .unwrap()
        .get(FrameStrata::Tooltip.as_index())
        .unwrap();
    assert!(
        tooltip_bucket.contains(&31),
        "shown tooltip should be present in the cached tooltip bucket"
    );
}
