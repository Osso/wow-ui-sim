use super::{collect_child_for_emit, dfs_emit, same_strata_subtree_segment_end};
use crate::widget::{DrawLayer, Frame, FrameStrata, WidgetRegistry, WidgetType};
use rustc_hash::FxHashSet;

fn test_frame(id: u64, widget_type: WidgetType, parent_id: Option<u64>) -> Frame {
    Frame {
        id,
        widget_type,
        parent_id,
        frame_strata: FrameStrata::Medium,
        visible: true,
        ..Default::default()
    }
}

#[test]
fn editbox_regions_render_before_internal_text_emitter() {
    let mut widgets = WidgetRegistry::default();
    widgets.register(test_frame(1, WidgetType::EditBox, None));
    widgets.register(Frame {
        draw_layer: DrawLayer::Background,
        ..test_frame(2, WidgetType::Texture, Some(1))
    });
    widgets.add_child(1, 2);

    let visible = FxHashSet::from_iter([1, 2]);
    let mut emitted = Vec::new();
    dfs_emit(
        1,
        FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut emitted,
    );

    assert_eq!(
        emitted,
        vec![2, 1],
        "EditBox child art must render below the EditBox internal text/caret emitter"
    );
}

#[test]
fn tooltip_texture_regions_render_before_internal_text_emitter() {
    let mut widgets = WidgetRegistry::default();
    widgets.register(test_frame(1, WidgetType::GameTooltip, None));
    widgets.register(Frame {
        draw_layer: DrawLayer::Background,
        ..test_frame(2, WidgetType::Texture, Some(1))
    });
    widgets.register(Frame {
        draw_layer: DrawLayer::Overlay,
        ..test_frame(3, WidgetType::FontString, Some(1))
    });
    widgets.add_child(1, 2);
    widgets.add_child(1, 3);

    let visible = FxHashSet::from_iter([1, 2, 3]);
    let mut emitted = Vec::new();
    dfs_emit(
        1,
        FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut emitted,
    );

    assert_eq!(
        emitted,
        vec![2, 1, 3],
        "tooltip-owned skin textures must render below internal tooltip text while FontString regions stay above"
    );
}

#[test]
fn collect_child_for_emit_routes_regions_and_same_strata_frames() {
    let mut widgets = WidgetRegistry::default();
    widgets.register(Frame {
        id: 2,
        widget_type: WidgetType::Texture,
        ..Default::default()
    });
    widgets.register(Frame {
        id: 3,
        widget_type: WidgetType::Frame,
        frame_strata: FrameStrata::Medium,
        ..Default::default()
    });
    widgets.register(Frame {
        id: 4,
        widget_type: WidgetType::Frame,
        frame_strata: FrameStrata::High,
        ..Default::default()
    });
    widgets.register(Frame {
        id: 5,
        widget_type: WidgetType::Texture,
        ..Default::default()
    });

    let visible = FxHashSet::from_iter([2, 3, 4]);
    let mut regions = Vec::new();
    let mut child_frames = Vec::new();

    for child_id in [2, 3, 4, 5] {
        collect_child_for_emit(
            child_id,
            FrameStrata::Medium.as_index(),
            &widgets,
            &visible,
            0,
            &mut regions,
            &mut child_frames,
        );
    }

    assert_eq!(
        regions.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(child_frames, vec![3]);
}

#[test]
fn dfs_emit_renders_tooltip_nineslice_before_tooltip_frame() {
    let mut widgets = WidgetRegistry::default();

    let tooltip_id = 100;
    let nineslice_id = 101;
    let border_tex_id = 102;

    let mut tooltip = test_frame(tooltip_id, WidgetType::GameTooltip, None);
    tooltip.children = vec![nineslice_id];
    tooltip
        .children_keys
        .insert("NineSlice".to_string(), nineslice_id);
    widgets.register(tooltip);

    let mut nineslice = test_frame(nineslice_id, WidgetType::Frame, Some(tooltip_id));
    nineslice.children = vec![border_tex_id];
    widgets.register(nineslice);

    let border = test_frame(border_tex_id, WidgetType::Texture, Some(nineslice_id));
    widgets.register(border);

    let visible = FxHashSet::from_iter([tooltip_id, nineslice_id, border_tex_id]);
    let mut out = Vec::new();
    dfs_emit(
        tooltip_id,
        crate::widget::FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut out,
    );

    let tooltip_pos = out
        .iter()
        .position(|&id| id == tooltip_id)
        .expect("tooltip should be emitted");
    let nineslice_pos = out
        .iter()
        .position(|&id| id == nineslice_id)
        .expect("nineslice should be emitted");
    assert!(
        nineslice_pos < tooltip_pos,
        "NineSlice should render before tooltip frame so tooltip text stays on top"
    );
}

#[test]
fn same_strata_subtree_segment_end_stops_at_first_non_subtree_id() {
    let bucket = vec![10, 11, 12, 99, 13];
    let subtree_ids = FxHashSet::from_iter([10, 11, 12, 13]);

    assert_eq!(same_strata_subtree_segment_end(&bucket, 0, &subtree_ids), 3);
}

#[test]
fn dfs_emit_keeps_transparent_wrapper_regions_after_wrapper_frame_and_parent_text_last() {
    let mut widgets = WidgetRegistry::default();
    widgets.register(test_frame(1, WidgetType::Frame, None));
    widgets.register(test_frame(2, WidgetType::Texture, Some(1)));
    widgets.register(test_frame(3, WidgetType::Frame, Some(1)));
    widgets.register(test_frame(4, WidgetType::Texture, Some(3)));
    widgets.register(test_frame(5, WidgetType::FontString, Some(1)));
    widgets.add_child(1, 2);
    widgets.add_child(1, 3);
    widgets.add_child(1, 5);
    widgets.add_child(3, 4);

    let visible = FxHashSet::from_iter([1, 2, 3, 4, 5]);
    let mut bucket = Vec::new();

    dfs_emit(
        1,
        FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut bucket,
    );

    assert_eq!(bucket, vec![1, 2, 3, 4, 5]);
}

#[test]
fn dfs_emit_keeps_wrapper_owned_regions_before_child_frames() {
    let mut widgets = WidgetRegistry::default();
    widgets.register(test_frame(1, WidgetType::Frame, None));
    widgets.register(test_frame(2, WidgetType::Frame, Some(1)));
    widgets.register(test_frame(3, WidgetType::Texture, Some(2)));
    widgets.register(test_frame(4, WidgetType::Frame, Some(2)));
    widgets.register(test_frame(5, WidgetType::Texture, Some(4)));
    widgets.add_child(1, 2);
    widgets.add_child(2, 3);
    widgets.add_child(2, 4);
    widgets.add_child(4, 5);

    let visible = FxHashSet::from_iter([1, 2, 3, 4, 5]);
    let mut bucket = Vec::new();

    dfs_emit(
        1,
        FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut bucket,
    );

    assert_eq!(
        bucket,
        vec![1, 2, 3, 4, 5],
        "wrapper-owned regions should render before descendant child frames"
    );
}

#[test]
fn higher_level_cover_occludes_text_hoisted_from_nested_wrappers() {
    for wrapper_type in [WidgetType::Frame, WidgetType::ScrollFrame] {
        let mut widgets = WidgetRegistry::default();
        for (id, kind, parent, level) in [
            (1, WidgetType::Frame, None, 100),
            (2, wrapper_type, Some(1), 101),
            (3, WidgetType::Frame, Some(2), 102),
            (4, WidgetType::FontString, Some(3), 0),
            (5, WidgetType::Frame, Some(1), 120),
            (6, WidgetType::Texture, Some(5), 0),
            (7, WidgetType::FontString, Some(5), 0),
        ] {
            widgets.register(Frame {
                frame_level: level,
                ..test_frame(id, kind, parent)
            });
            if let Some(parent) = parent {
                widgets.add_child(parent, id);
            }
        }
        let visible = FxHashSet::from_iter(1..=7);
        let mut emitted = Vec::new();
        dfs_emit(
            1,
            FrameStrata::Medium.as_index(),
            &widgets,
            &visible,
            &mut emitted,
        );
        let position = |id| emitted.iter().rposition(|&value| value == id).unwrap();
        assert!(
            position(4) < position(6),
            "lower text must precede opaque cover: {emitted:?}"
        );
        assert!(
            position(6) < position(7),
            "cover's own label must remain visible: {emitted:?}"
        );
        let mut unique = emitted.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 7);
        assert_eq!(emitted.len(), 7, "each widget must be emitted exactly once");
    }
}

#[test]
fn higher_level_child_occludes_its_parents_text() {
    let mut widgets = WidgetRegistry::default();
    for (id, kind, parent, level) in [
        (1, WidgetType::Frame, None, 100),
        (2, WidgetType::FontString, Some(1), 0),
        (3, WidgetType::Frame, Some(1), 120),
        (4, WidgetType::Texture, Some(3), 0),
    ] {
        widgets.register(Frame {
            frame_level: level,
            ..test_frame(id, kind, parent)
        });
        if let Some(parent) = parent {
            widgets.add_child(parent, id);
        }
    }
    let visible = FxHashSet::from_iter(1..=4);
    let mut emitted = Vec::new();
    dfs_emit(
        1,
        FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut emitted,
    );
    assert_eq!(emitted, vec![1, 2, 3, 4]);
}

#[test]
fn same_level_hoisting_emits_each_region_once_and_preserves_button_labels() {
    let mut widgets = WidgetRegistry::default();
    for (id, kind, parent) in [
        (1, WidgetType::Frame, None),
        (2, WidgetType::ScrollFrame, Some(1)),
        (3, WidgetType::Frame, Some(2)),
        (4, WidgetType::FontString, Some(3)),
        (5, WidgetType::Button, Some(2)),
        (6, WidgetType::FontString, Some(5)),
    ] {
        widgets.register(test_frame(id, kind, parent));
        if let Some(parent) = parent {
            widgets.add_child(parent, id);
        }
    }
    let visible = FxHashSet::from_iter(1..=6);
    let mut emitted = Vec::new();
    dfs_emit(
        1,
        FrameStrata::Medium.as_index(),
        &widgets,
        &visible,
        &mut emitted,
    );
    assert_eq!(emitted, vec![1, 2, 3, 5, 6, 4]);
}
