//! Tests for positioning: GetLeft, GetRight, GetTop, GetBottom, GetCenter,
//! GetRect, IsRectValid, coordinate system, invariants.

use super::*;

#[test]
fn test_no_anchors_return_empty() {
    let (t, _) = load_test_lua(
        "layout-pos-empty",
        r#"
        local f = CreateFrame("Frame")
        LEFT_N = select('#', f:GetLeft())
        RIGHT_N = select('#', f:GetRight())
        TOP_N = select('#', f:GetTop())
        BOTTOM_N = select('#', f:GetBottom())
        CENTER_N = select('#', f:GetCenter())
        RECT_N = select('#', f:GetRect())
    "#,
    );
    for var in &[
        "LEFT_N", "RIGHT_N", "TOP_N", "BOTTOM_N", "CENTER_N", "RECT_N",
    ] {
        let n: i32 = t.env.eval(&format!("return {}", var)).unwrap();
        assert_eq!(n, 0, "{} should be 0 with no anchors", var);
    }
}

#[test]
fn test_ui_parent_get_rect_returns_screen_rect() {
    let (t, _) = load_test_lua(
        "layout-pos-ui-parent-rect",
        r#"
        COUNT = select('#', UIParent:GetRect())
        local left, bottom, width, height = UIParent:GetRect()
        LEFT = left
        BOTTOM = bottom
        WIDTH = width
        HEIGHT = height
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return COUNT").unwrap(), 4);
    assert_f64_near(&t, "LEFT", 0.0);
    assert_f64_near(&t, "BOTTOM", 0.0);
    assert_f64_near(&t, "WIDTH", 1024.0);
    assert_f64_near(&t, "HEIGHT", 768.0);
}

#[test]
fn test_get_rect_recovers_pending_layout_for_anchored_frame() {
    let (t, _) = load_test_lua(
        "layout-pos-pending-rect",
        r#"
        local f = CreateFrame("Frame", "PendingRectFrame", UIParent)
        f:SetSize(100, 50)
        f:SetPoint("CENTER", UIParent, "CENTER", 10, -20)
    "#,
    );

    {
        let mut state = t.env.state().borrow_mut();
        let id = state
            .widgets
            .get_id_by_name("PendingRectFrame")
            .expect("PendingRectFrame should exist");
        state
            .widgets
            .get_mut(id)
            .expect("pending frame should exist")
            .layout_rect = None;
        state.widgets.clear_rect_dirty(id);
    }

    let count: i32 = t
        .env
        .eval("return select('#', PendingRectFrame:GetRect())")
        .unwrap();
    assert_eq!(count, 4);
    let width: f64 = t
        .env
        .eval("return select(3, PendingRectFrame:GetRect())")
        .unwrap();
    let height: f64 = t
        .env
        .eval("return select(4, PendingRectFrame:GetRect())")
        .unwrap();
    assert!(
        (width - 100.0).abs() < 0.01,
        "expected width 100, got {}",
        width
    );
    assert!(
        (height - 50.0).abs() < 0.01,
        "expected height 50, got {}",
        height
    );
}

#[test]
fn test_scroll_child_without_explicit_anchor_has_queryable_rect() {
    let (t, _) = load_test_lua(
        "layout-scroll-child-implicit-anchor",
        r#"
        local scrollFrame = CreateFrame("ScrollFrame", "ImplicitAnchorScrollFrame", UIParent)
        scrollFrame:SetSize(200, 100)
        scrollFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 40, -60)

        local scrollChild = CreateFrame("Frame", "ImplicitAnchorScrollChild")
        scrollChild:SetSize(180, 300)
        scrollFrame:SetScrollChild(scrollChild)

        CHILD_TOP_COUNT = select('#', scrollChild:GetTop())
        CHILD_BOTTOM_COUNT = select('#', scrollChild:GetBottom())
        TOP_DIFF = math.abs(scrollChild:GetTop() - scrollFrame:GetTop())
    "#,
    );

    assert_eq!(t.env.eval::<i32>("return CHILD_TOP_COUNT").unwrap(), 1);
    assert_eq!(t.env.eval::<i32>("return CHILD_BOTTOM_COUNT").unwrap(), 1);
    assert!(t.env.eval::<f64>("return TOP_DIFF").unwrap() < 0.01);
}

#[test]
fn test_topleft_at_origin() {
    let (t, _) = load_test_lua(
        "layout-pos-tl-origin",
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        root:SetSize(800, 600)
        root:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local f = CreateFrame("Frame", nil, root)
        f:SetSize(100, 50)
        f:SetPoint("TOPLEFT", root, "TOPLEFT", 0, 0)
        LEFT = f:GetLeft()
        RIGHT = f:GetRight()
        -- Top/bottom use WoW Y-up coords relative to screen, check invariants
        TOP_MINUS_BOTTOM = f:GetTop() - f:GetBottom()
    "#,
    );
    assert_f64_near(&t, "LEFT", 0.0);
    assert_f64_near(&t, "RIGHT", 100.0);
    assert_f64_near(&t, "TOP_MINUS_BOTTOM", 50.0);
}

#[test]
fn test_topleft_with_offset() {
    let (t, _) = load_test_lua(
        "layout-pos-tl-offset",
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        root:SetSize(800, 600)
        root:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local f = CreateFrame("Frame", nil, root)
        f:SetSize(100, 50)
        f:SetPoint("TOPLEFT", root, "TOPLEFT", 20, -10)
        LEFT = f:GetLeft()
        RIGHT = f:GetRight()
        -- Offset 20 right, 10 down from root's top-left
        DIFF_LEFT = math.abs(f:GetLeft() - root:GetLeft() - 20)
        WIDTH_INV = math.abs(f:GetRight() - f:GetLeft() - 100)
    "#,
    );
    assert_f64_near(&t, "LEFT", 20.0);
    assert_f64_near(&t, "RIGHT", 120.0);
    assert_f64_near(&t, "DIFF_LEFT", 0.0);
    assert_f64_near(&t, "WIDTH_INV", 0.0);
}

#[test]
fn test_center_on_parent() {
    let (t, _) = load_test_lua(
        "layout-pos-center",
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        root:SetSize(400, 300)
        root:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local f = CreateFrame("Frame", nil, root)
        f:SetSize(100, 50)
        f:SetPoint("CENTER", root, "CENTER", 0, 0)
        local cx, cy = f:GetCenter()
        local rx, ry = root:GetCenter()
        -- Child center should match parent center
        DIFF_CX = math.abs(cx - rx)
        DIFF_CY = math.abs(cy - ry)
    "#,
    );
    assert_f64_near(&t, "DIFF_CX", 0.0);
    assert_f64_near(&t, "DIFF_CY", 0.0);
}

#[test]
fn test_center_with_offset() {
    let (t, _) = load_test_lua(
        "layout-pos-center-off",
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        root:SetSize(400, 300)
        root:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local f = CreateFrame("Frame", nil, root)
        f:SetSize(100, 50)
        f:SetPoint("CENTER", root, "CENTER", 10, 20)
        local cx, cy = f:GetCenter()
        local rx, ry = root:GetCenter()
        -- Offset shifts center by (10, 20) in UI coords
        DIFF_CX = math.abs(cx - rx - 10)
        DIFF_CY = math.abs(cy - ry - 20)
    "#,
    );
    assert_f64_near(&t, "DIFF_CX", 0.0);
    assert_f64_near(&t, "DIFF_CY", 0.0);
}

#[test]
fn test_bottomright_at_corner() {
    let (t, _) = load_test_lua(
        "layout-pos-br",
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        root:SetSize(400, 300)
        root:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local f = CreateFrame("Frame", nil, root)
        f:SetSize(100, 50)
        f:SetPoint("BOTTOMRIGHT", root, "BOTTOMRIGHT", 0, 0)
        -- Frame's right edge should match root's right edge
        DIFF_RIGHT = math.abs(f:GetRight() - root:GetRight())
        DIFF_BOTTOM = math.abs(f:GetBottom() - root:GetBottom())
        WIDTH = f:GetRight() - f:GetLeft()
        HEIGHT = f:GetTop() - f:GetBottom()
    "#,
    );
    assert_f64_near(&t, "DIFF_RIGHT", 0.0);
    assert_f64_near(&t, "DIFF_BOTTOM", 0.0);
    assert_f64_near(&t, "WIDTH", 100.0);
    assert_f64_near(&t, "HEIGHT", 50.0);
}

#[test]
fn test_get_rect_returns_4_values() {
    let (t, _) = load_test_lua(
        "layout-getrect-count",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(100, 50); f:SetPoint("CENTER")
        COUNT = select('#', f:GetRect())
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return COUNT").unwrap(), 4);
}

#[test]
fn test_get_rect_values() {
    let (t, _) = load_test_lua(
        "layout-getrect",
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        root:SetSize(800, 600)
        root:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local f = CreateFrame("Frame", nil, root)
        f:SetSize(100, 50)
        f:SetPoint("TOPLEFT", root, "TOPLEFT", 0, 0)
        local left, bottom, w, h = f:GetRect()
        LEFT = left; W = w; H = h
        -- bottom should equal root's bottom + root's height - frame's height
        -- i.e. same top edge as root (offset 0)
        BOTTOM_DIFF = math.abs(f:GetBottom() - (root:GetTop() - 50))
    "#,
    );
    assert_f64_near(&t, "LEFT", 0.0);
    assert_f64_near(&t, "W", 100.0);
    assert_f64_near(&t, "H", 50.0);
    assert_f64_near(&t, "BOTTOM_DIFF", 0.0);
}

// ---- Invariants ----

#[test]
fn test_invariant_right_minus_left_equals_width() {
    let (t, _) = load_test_lua(
        "layout-inv-rlw",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(137, 89)
        f:SetPoint("CENTER", UIParent, "CENTER", 33, -17)
        DIFF = math.abs(f:GetRight() - f:GetLeft() - f:GetWidth())
    "#,
    );
    let diff = t.env.eval::<f64>("return DIFF").unwrap();
    assert!(diff < 0.01, "right - left - width = {}", diff);
}

#[test]
fn test_invariant_top_minus_bottom_equals_height() {
    let (t, _) = load_test_lua(
        "layout-inv-tbh",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(137, 89)
        f:SetPoint("CENTER", UIParent, "CENTER", 33, -17)
        DIFF = math.abs(f:GetTop() - f:GetBottom() - f:GetHeight())
    "#,
    );
    let diff = t.env.eval::<f64>("return DIFF").unwrap();
    assert!(diff < 0.01, "top - bottom - height = {}", diff);
}

#[test]
fn test_invariant_center_is_midpoint() {
    let (t, _) = load_test_lua(
        "layout-inv-center",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(137, 89)
        f:SetPoint("CENTER", UIParent, "CENTER", 33, -17)
        local cx, cy = f:GetCenter()
        DIFF_X = math.abs(cx - (f:GetLeft() + f:GetRight()) / 2)
        DIFF_Y = math.abs(cy - (f:GetTop() + f:GetBottom()) / 2)
    "#,
    );
    assert!(t.env.eval::<f64>("return DIFF_X").unwrap() < 0.01);
    assert!(t.env.eval::<f64>("return DIFF_Y").unwrap() < 0.01);
}

#[test]
fn test_invariant_rect_matches_edges() {
    let (t, _) = load_test_lua(
        "layout-inv-rect-edges",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(200, 100)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 50, -30)
        local left, bottom, w, h = f:GetRect()
        DL = math.abs(f:GetLeft() - left)
        DB = math.abs(f:GetBottom() - bottom)
        DW = math.abs(f:GetWidth() - w)
        DH = math.abs(f:GetHeight() - h)
    "#,
    );
    for var in &["DL", "DB", "DW", "DH"] {
        let d: f64 = t.env.eval(&format!("return {}", var)).unwrap();
        assert!(d < 0.01, "{} = {}", var, d);
    }
}

// ---- Composite positioning ----

#[test]
fn test_set_all_points_matches_parent_edges() {
    let (t, _) = load_test_lua(
        "layout-sap-edges",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 150); parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, parent)
        child:SetAllPoints()
        DL = math.abs(parent:GetLeft() - child:GetLeft())
        DR = math.abs(parent:GetRight() - child:GetRight())
        DT = math.abs(parent:GetTop() - child:GetTop())
        DB = math.abs(parent:GetBottom() - child:GetBottom())
    "#,
    );
    for var in &["DL", "DR", "DT", "DB"] {
        let d: f64 = t.env.eval(&format!("return {}", var)).unwrap();
        assert!(d < 0.01, "{} = {}", var, d);
    }
}

#[test]
fn test_position_empty_after_clear() {
    let (t, _) = load_test_lua(
        "layout-pos-clear",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(100, 50); f:SetPoint("CENTER")
        f:GetLeft()
        f:ClearAllPoints()
        LEFT_N = select('#', f:GetLeft())
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return LEFT_N").unwrap(), 0);
}

#[test]
fn test_nested_position_offset() {
    let (t, _) = load_test_lua(
        "layout-nested-pos",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 100)
        parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 50, -50)
        local child = CreateFrame("Frame", nil, parent)
        child:SetSize(80, 40)
        child:SetPoint("TOPLEFT", parent, "TOPLEFT", 10, -10)
        CHILD_LEFT = child:GetLeft()
        CHILD_RIGHT = child:GetRight()
    "#,
    );
    assert_f64_near(&t, "CHILD_LEFT", 60.0);
    assert_f64_near(&t, "CHILD_RIGHT", 140.0);
}

#[test]
fn test_sibling_anchor() {
    let (t, _) = load_test_lua(
        "layout-sibling",
        r#"
        local a = CreateFrame("Frame", "SibA", UIParent)
        a:SetSize(100, 50)
        a:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        local b = CreateFrame("Frame", "SibB", UIParent)
        b:SetSize(100, 50)
        b:SetPoint("LEFT", a, "RIGHT", 10, 0)
        A_RIGHT = a:GetRight()
        B_LEFT = b:GetLeft()
    "#,
    );
    let ar = t.env.eval::<f64>("return A_RIGHT").unwrap();
    let bl = t.env.eval::<f64>("return B_LEFT").unwrap();
    assert!(
        (bl - (ar + 10.0)).abs() < 0.01,
        "b left={} vs a right={}+10",
        bl,
        ar
    );
}

#[test]
fn test_opposite_anchors_define_size() {
    let (t, _) = load_test_lua(
        "layout-opp-size",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(400, 300); parent:SetPoint("CENTER")
        local f = CreateFrame("Frame", nil, parent)
        f:SetPoint("TOPLEFT", parent, "TOPLEFT", 20, -20)
        f:SetPoint("BOTTOMRIGHT", parent, "BOTTOMRIGHT", -20, 20)
        W = f:GetWidth(); H = f:GetHeight()
    "#,
    );
    let w = t.env.eval::<f64>("return W").unwrap();
    let h = t.env.eval::<f64>("return H").unwrap();
    assert!((w - 360.0).abs() < 0.01, "w: {}", w);
    assert!((h - 260.0).abs() < 0.01, "h: {}", h);
}

#[test]
fn test_bottom_to_top_anchor() {
    let (t, _) = load_test_lua(
        "layout-bottom-top",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 100); parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, UIParent)
        child:SetSize(100, 30)
        child:SetPoint("BOTTOM", parent, "TOP", 0, 0)
        PARENT_TOP = parent:GetTop()
        CHILD_BOTTOM = child:GetBottom()
    "#,
    );
    let pt = t.env.eval::<f64>("return PARENT_TOP").unwrap();
    let cb = t.env.eval::<f64>("return CHILD_BOTTOM").unwrap();
    assert!(
        (pt - cb).abs() < 0.01,
        "parent top={} vs child bottom={}",
        pt,
        cb
    );
}

#[test]
fn test_right_to_left_anchor() {
    let (t, _) = load_test_lua(
        "layout-right-left",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 100); parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, UIParent)
        child:SetSize(100, 30)
        child:SetPoint("RIGHT", parent, "LEFT", 0, 0)
        PARENT_LEFT = parent:GetLeft()
        CHILD_RIGHT = child:GetRight()
    "#,
    );
    let pl = t.env.eval::<f64>("return PARENT_LEFT").unwrap();
    let cr = t.env.eval::<f64>("return CHILD_RIGHT").unwrap();
    assert!(
        (pl - cr).abs() < 0.01,
        "parent left={} vs child right={}",
        pl,
        cr
    );
}

#[test]
fn test_scaled_frame_preserves_invariants() {
    let (t, _) = load_test_lua(
        "layout-scale-inv",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(100, 50)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        f:SetScale(2.0)
        INV_W = math.abs(f:GetRight() - f:GetLeft() - f:GetWidth())
        INV_H = math.abs(f:GetTop() - f:GetBottom() - f:GetHeight())
    "#,
    );
    assert!(t.env.eval::<f64>("return INV_W").unwrap() < 0.01);
    assert!(t.env.eval::<f64>("return INV_H").unwrap() < 0.01);
}

#[test]
fn test_is_rect_valid_no_anchors() {
    let (t, _) = load_test_lua(
        "layout-isrectvalid-none",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(100, 50)
        local width, height = f:GetSize()
        WIDTH, HEIGHT = width, height
        V2 = f:IsRectValid()
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return WIDTH").unwrap(), 100.0);
    assert_eq!(t.env.eval::<f64>("return HEIGHT").unwrap(), 50.0);
    assert!(!t.env.eval::<bool>("return V2").unwrap());
}

// ---- Helpers ----

fn assert_f64_near(t: &TestCtx, var: &str, expected: f64) {
    let actual: f64 = t.env.eval(&format!("return {}", var)).unwrap();
    assert!(
        (actual - expected).abs() < 0.01,
        "{}: expected {}, got {}",
        var,
        expected,
        actual
    );
}
