use super::*;
use crate::widget::{Anchor, Frame};

fn make_frame(
    id: u64,
    parent: Option<u64>,
    w: f32,
    h: f32,
    children: Vec<u64>,
    anchors: Vec<Anchor>,
) -> Frame {
    let mut f = Frame::default();
    f.id = id;
    f.parent_id = parent;
    f.width = w;
    f.height = h;
    f.children = children;
    f.anchors = anchors;
    f
}

fn build_three_slice_registry() -> WidgetRegistry {
    let mut reg = WidgetRegistry::new();
    register_ui_parent(&mut reg);
    register_button_frame(&mut reg);
    register_edge_frame(&mut reg, 20, AnchorPoint::Left);
    register_edge_frame(&mut reg, 21, AnchorPoint::Right);
    register_center_frame(&mut reg);
    reg
}

fn register_ui_parent(reg: &mut WidgetRegistry) {
    let mut uip = make_frame(1, None, 1024.0, 768.0, vec![10], vec![]);
    uip.name = Some("UIParent".to_string());
    reg.register(uip);
}

fn register_button_frame(reg: &mut WidgetRegistry) {
    reg.register(make_frame(
        10,
        Some(1),
        200.0,
        36.0,
        vec![20, 21, 22],
        vec![Anchor::from_relative_id(
            AnchorPoint::Center,
            Some(1),
            AnchorPoint::Center,
        )],
    ));
}

fn register_edge_frame(reg: &mut WidgetRegistry, id: u64, point: AnchorPoint) {
    reg.register(make_frame(
        id,
        Some(10),
        32.0,
        39.0,
        vec![],
        vec![Anchor::from_relative_id(point, Some(10), point)],
    ));
}

fn register_center_frame(reg: &mut WidgetRegistry) {
    reg.register(make_frame(
        22,
        Some(10),
        0.0,
        0.0,
        vec![],
        vec![
            Anchor::from_relative_id(AnchorPoint::TopLeft, Some(20), AnchorPoint::TopRight),
            Anchor::from_relative_id(AnchorPoint::BottomRight, Some(21), AnchorPoint::BottomLeft),
        ],
    ));
}

#[test]
fn test_cross_frame_anchor_center_texture() {
    let registry = build_three_slice_registry();
    let rect = compute_frame_rect(&registry, 22, 1024.0, 768.0);
    assert!(
        rect.width > 100.0,
        "Center width should be ~136, got {}",
        rect.width
    );
    assert!(
        rect.height > 30.0,
        "Center height should be ~39, got {}",
        rect.height
    );
}

#[test]
fn test_nil_relative_to_anchors_multi_anchor_frame_to_screen() {
    let mut registry = WidgetRegistry::new();
    let mut root = make_frame(1, None, 1024.0, 768.0, vec![2], vec![]);
    root.name = Some("UIParent".to_string());
    registry.register(root);
    let mut frame = make_frame(
        2,
        Some(1),
        100.0,
        40.0,
        vec![],
        vec![
            Anchor::from_relative_id(AnchorPoint::TopLeft, None, AnchorPoint::TopLeft),
            Anchor::from_relative_id(AnchorPoint::BottomRight, None, AnchorPoint::BottomRight),
        ],
    );
    frame.anchors[0].x_offset = 10.0;
    frame.anchors[0].y_offset = -20.0;
    frame.anchors[1].x_offset = -30.0;
    frame.anchors[1].y_offset = 40.0;
    registry.register(frame);

    let rect = compute_frame_rect(&registry, 2, 1024.0, 768.0);
    assert_eq!(rect.x, 10.0);
    assert_eq!(rect.y, 20.0);
    assert_eq!(rect.width, 984.0);
    assert_eq!(rect.height, 708.0);
}

#[test]
fn duplicate_left_edge_uses_frame_point_priority_not_anchor_order() {
    for points in [
        [AnchorPoint::TopLeft, AnchorPoint::BottomLeft],
        [AnchorPoint::BottomLeft, AnchorPoint::TopLeft],
    ] {
        let rect = compute_duplicate_left_edge_rect(points);

        assert_eq!(rect.x, 18.0);
        assert_eq!(rect.width, 199.0);
        assert_eq!(rect.y, 76.0);
        assert_eq!(rect.height, 602.0);
    }
}

fn compute_duplicate_left_edge_rect(points: [AnchorPoint; 2]) -> LayoutRect {
    let mut registry = WidgetRegistry::new();
    registry.register(make_frame(1, None, 920.0, 724.0, vec![2], vec![]));
    let anchors = points.map(duplicate_left_anchor).to_vec();
    registry.register(make_frame(2, Some(1), 199.0, 569.0, vec![], anchors));
    compute_frame_rect(&registry, 2, 920.0, 724.0)
}

fn duplicate_left_anchor(point: AnchorPoint) -> Anchor {
    let (x_offset, y_offset) = match point {
        AnchorPoint::TopLeft => (18.0, -76.0),
        AnchorPoint::BottomLeft => (178.0, 46.0),
        _ => unreachable!("test only uses left-edge anchors"),
    };
    let mut anchor = Anchor::from_relative_id(point, Some(1), point);
    anchor.x_offset = x_offset;
    anchor.y_offset = y_offset;
    anchor
}

#[test]
fn parent_cycle_does_not_recurse_until_stack_overflow() {
    let mut registry = WidgetRegistry::new();
    registry.register(make_frame(10, Some(11), 100.0, 40.0, vec![11], vec![]));
    registry.register(make_frame(11, Some(10), 200.0, 80.0, vec![10], vec![]));

    let rect = compute_frame_rect(&registry, 10, 1024.0, 768.0);

    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 40.0);
}

#[test]
fn anchor_cycle_does_not_recurse_until_stack_overflow() {
    let mut registry = WidgetRegistry::new();
    register_ui_parent(&mut registry);
    registry.register(make_frame(
        20,
        Some(1),
        100.0,
        40.0,
        vec![],
        vec![Anchor::from_relative_id(
            AnchorPoint::Center,
            Some(21),
            AnchorPoint::Center,
        )],
    ));
    registry.register(make_frame(
        21,
        Some(1),
        100.0,
        40.0,
        vec![],
        vec![Anchor::from_relative_id(
            AnchorPoint::Center,
            Some(20),
            AnchorPoint::Center,
        )],
    ));

    let rect = compute_frame_rect(&registry, 20, 1024.0, 768.0);

    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 40.0);
}

#[test]
fn scroll_offset_follows_the_anchor_not_the_parent() {
    // 2 is a scroll frame scrolled down 30; 3 its scroll child. 4 and 5 are both
    // children of the scroll child: 4 is anchored to it and scrolls, 5 is anchored
    // to the scroll frame itself and stays where its anchor puts it.
    let mut registry = WidgetRegistry::new();
    register_ui_parent(&mut registry);
    let to = |id| {
        vec![Anchor::from_relative_id(
            AnchorPoint::TopLeft,
            Some(id),
            AnchorPoint::TopLeft,
        )]
    };
    let mut scroll = make_frame(2, Some(1), 100.0, 100.0, vec![3], to(1));
    scroll.widget_type = WidgetType::ScrollFrame;
    scroll.scroll_child_id = Some(3);
    scroll.scroll_vertical = 30.0;
    for (frame, id) in [
        (scroll, 2),
        (make_frame(3, Some(2), 200.0, 200.0, vec![4, 5], to(2)), 3),
        (make_frame(4, Some(3), 20.0, 20.0, vec![], to(3)), 4),
        (make_frame(5, Some(3), 20.0, 20.0, vec![], to(2)), 5),
    ] {
        let mut frame = frame;
        frame.effective_scale = 1.0;
        assert_eq!(frame.id, id);
        registry.register(frame);
    }
    let origin = LayoutRect {
        x: 0.0,
        y: 0.0,
        width: 20.0,
        height: 20.0,
    };

    assert_eq!(apply_scroll_offsets(&registry, 4, origin).y, -30.0);
    assert_eq!(apply_scroll_offsets(&registry, 5, origin).y, 0.0);
}
