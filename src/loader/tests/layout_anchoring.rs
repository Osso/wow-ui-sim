//! Tests for anchoring: SetPoint, ClearAllPoints, ClearPoint, GetPoint,
//! GetNumPoints, SetAllPoints, AdjustPointsOffset, cycle detection.

use super::*;

#[test]
fn test_anchor_initial_state() {
    let (t, _) = load_test_lua(
        "layout-anchor-init",
        r#"
        local f = CreateFrame("Frame")
        NUM_POINTS = f:GetNumPoints()
        GET_POINT_COUNT = select('#', f:GetPoint(1))
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return NUM_POINTS").unwrap(), 0);
    assert_eq!(t.env.eval::<i32>("return GET_POINT_COUNT").unwrap(), 0);
}

#[test]
fn test_set_point_center_defaults_to_parent() {
    let (t, _) = load_test_lua(
        "layout-sp-center",
        r#"
        local f = CreateFrame("Frame", "SPCenterFrame", UIParent)
        f:SetSize(50, 30)
        f:SetPoint("CENTER")
        NUM = f:GetNumPoints()
        local point, relativeTo, relPoint, x, y = f:GetPoint(1)
        POINT = point
        REL_POINT = relPoint
        XOFS = x
        YOFS = y
        REL_IS_PARENT = (relativeTo == UIParent)
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return NUM").unwrap(), 1);
    t.assert_lua_str("return POINT", "CENTER");
    t.assert_lua_str("return REL_POINT", "CENTER");
    assert_eq!(t.env.eval::<f64>("return XOFS").unwrap(), 0.0);
    assert_eq!(t.env.eval::<f64>("return YOFS").unwrap(), 0.0);
    t.assert_lua_true("return REL_IS_PARENT", "relativeTo should be UIParent");
}

#[test]
fn test_set_point_with_offset() {
    let (t, _) = load_test_lua(
        "layout-sp-offset",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("TOPLEFT", 10, -20)
        local point, _, relPoint, x, y = f:GetPoint(1)
        RESULT = string.format("%s,%s,%d,%d", point, relPoint, x, y)
    "#,
    );
    t.assert_lua_str("return RESULT", "TOPLEFT,TOPLEFT,10,-20");
}

#[test]
fn test_set_point_full_form() {
    let (t, _) = load_test_lua(
        "layout-sp-full",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("BOTTOMLEFT", UIParent, "TOPLEFT", 5, -10)
        local point, relativeTo, relPoint, x, y = f:GetPoint(1)
        RESULT = string.format("%s,%s,%d,%d", point, relPoint, x, y)
        REL_IS_PARENT = (relativeTo == UIParent)
    "#,
    );
    t.assert_lua_str("return RESULT", "BOTTOMLEFT,TOPLEFT,5,-10");
    t.assert_lua_true("return REL_IS_PARENT", "relativeTo should be UIParent");
}

#[test]
fn test_set_point_with_string_name() {
    let (t, _) = load_test_lua(
        "layout-sp-string",
        r#"
        local target = CreateFrame("Frame", "StringTarget", UIParent)
        target:SetAllPoints(UIParent)
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("CENTER", "StringTarget", "CENTER")
        local _, relativeTo = f:GetPoint(1)
        REL_MATCH = (relativeTo == target)
    "#,
    );
    t.assert_lua_true("return REL_MATCH", "relativeTo should be the named frame");
}

#[test]
fn test_set_point_replaces_same_point() {
    let (t, _) = load_test_lua(
        "layout-sp-replace",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
        f:SetPoint("CENTER", UIParent, "CENTER", 10, 20)
        NUM = f:GetNumPoints()
        local _, _, _, x, y = f:GetPoint(1)
        XOFS = x; YOFS = y
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return NUM").unwrap(), 1);
    assert_eq!(t.env.eval::<f64>("return XOFS").unwrap(), 10.0);
    assert_eq!(t.env.eval::<f64>("return YOFS").unwrap(), 20.0);
}

#[test]
fn test_get_num_points_increments() {
    let (t, _) = load_test_lua(
        "layout-numpoints",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        N0 = f:GetNumPoints()
        f:SetPoint("TOPLEFT")
        N1 = f:GetNumPoints()
        f:SetPoint("BOTTOMRIGHT")
        N2 = f:GetNumPoints()
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N0").unwrap(), 0);
    assert_eq!(t.env.eval::<i32>("return N1").unwrap(), 1);
    assert_eq!(t.env.eval::<i32>("return N2").unwrap(), 2);
}

#[test]
fn test_get_point_sorted_by_sort_key() {
    let (t, _) = load_test_lua(
        "layout-gp-sorted",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", 0, 0)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        f:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
        P1 = select(1, f:GetPoint(1))
        P2 = select(1, f:GetPoint(2))
        P3 = select(1, f:GetPoint(3))
    "#,
    );
    t.assert_lua_str("return P1", "TOPLEFT");
    t.assert_lua_str("return P2", "CENTER");
    t.assert_lua_str("return P3", "BOTTOMRIGHT");
}

#[test]
fn test_get_point_out_of_range() {
    let (t, _) = load_test_lua(
        "layout-gp-oor",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("CENTER")
        COUNT_2 = select('#', f:GetPoint(2))
        COUNT_99 = select('#', f:GetPoint(99))
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return COUNT_2").unwrap(), 0);
    assert_eq!(t.env.eval::<i32>("return COUNT_99").unwrap(), 0);
}

#[test]
fn test_get_point_default_index() {
    let (t, _) = load_test_lua(
        "layout-gp-default",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("CENTER")
        POINT = select(1, f:GetPoint())
    "#,
    );
    t.assert_lua_str("return POINT", "CENTER");
}

#[test]
fn test_clear_all_points() {
    let (t, _) = load_test_lua(
        "layout-clearall",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT")
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT")
        N_BEFORE = f:GetNumPoints()
        f:ClearAllPoints()
        N_AFTER = f:GetNumPoints()
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N_BEFORE").unwrap(), 2);
    assert_eq!(t.env.eval::<i32>("return N_AFTER").unwrap(), 0);
}

#[test]
fn test_clear_all_points_idempotent() {
    let (t, _) = load_test_lua(
        "layout-clearall-idem",
        r#"
        local f = CreateFrame("Frame")
        f:ClearAllPoints()
        f:ClearAllPoints()
        N = f:GetNumPoints()
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N").unwrap(), 0);
}

#[test]
fn test_clear_all_points_invalidates_resolved_rect() {
    let (t, _) = load_test_lua(
        "layout-clearall-dirty",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("CENTER")
        f:GetLeft()
        VALID_BEFORE = f:IsRectValid()
        f:ClearAllPoints()
        VALID_CACHED = f:IsRectValid()
        f:GetLeft()
        VALID_AFTER_QUERY = f:IsRectValid()
    "#,
    );
    t.assert_lua_true("return VALID_BEFORE", "geometry getter caches a valid rect");
    t.assert_lua_true("return VALID_CACHED", "IsRectValid reads the cache");
    assert!(!t.env.eval::<bool>("return VALID_AFTER_QUERY").unwrap());
}

#[test]
fn test_clear_point_specific() {
    let (t, _) = load_test_lua(
        "layout-clearpoint",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT")
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT")
        f:ClearPoint("TOPLEFT")
        N = f:GetNumPoints()
        REMAINING = select(1, f:GetPoint(1))
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N").unwrap(), 1);
    t.assert_lua_str("return REMAINING", "BOTTOMRIGHT");
}

#[test]
fn test_clear_point_nonexistent_is_silent() {
    let (t, _) = load_test_lua(
        "layout-clearpoint-noop",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("CENTER")
        f:ClearPoint("TOPLEFT")
        N = f:GetNumPoints()
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N").unwrap(), 1);
}

#[test]
fn test_set_all_points_creates_tl_br() {
    let (t, _) = load_test_lua(
        "layout-sap-tlbr",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetAllPoints()
        N = f:GetNumPoints()
        P1 = select(1, f:GetPoint(1))
        P2 = select(1, f:GetPoint(2))
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N").unwrap(), 2);
    t.assert_lua_str("return P1", "TOPLEFT");
    t.assert_lua_str("return P2", "BOTTOMRIGHT");
}

#[test]
fn test_set_all_points_explicit_target() {
    let (t, _) = load_test_lua(
        "layout-sap-target",
        r#"
        local target = CreateFrame("Frame", "SAPTarget", UIParent)
        target:SetAllPoints(UIParent)
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetAllPoints(target)
        local _, rel1 = f:GetPoint(1)
        local _, rel2 = f:GetPoint(2)
        REL1_MATCH = (rel1 == target)
        REL2_MATCH = (rel2 == target)
    "#,
    );
    t.assert_lua_true("return REL1_MATCH", "first anchor target");
    t.assert_lua_true("return REL2_MATCH", "second anchor target");
}

#[test]
fn test_set_all_points_clears_previous() {
    let (t, _) = load_test_lua(
        "layout-sap-clears",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("CENTER", UIParent, "CENTER", 50, 50)
        f:SetPoint("TOP", UIParent, "TOP")
        f:SetAllPoints()
        N = f:GetNumPoints()
        P1 = select(1, f:GetPoint(1))
        P2 = select(1, f:GetPoint(2))
    "#,
    );
    assert_eq!(t.env.eval::<i32>("return N").unwrap(), 2);
    t.assert_lua_str("return P1", "TOPLEFT");
    t.assert_lua_str("return P2", "BOTTOMRIGHT");
}

#[test]
fn test_set_all_points_matches_parent_size() {
    let (t, _) = load_test_lua(
        "layout-sap-size",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 150)
        parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, parent)
        child:SetAllPoints()
        local pw, ph = parent:GetSize()
        local cw, ch = child:GetSize()
        W_OK = math.abs(pw - cw) < 0.01
        H_OK = math.abs(ph - ch) < 0.01
    "#,
    );
    t.assert_lua_true("return W_OK", "child width should match parent");
    t.assert_lua_true("return H_OK", "child height should match parent");
}

#[test]
fn test_adjust_points_offset_single() {
    let (t, _) = load_test_lua(
        "layout-adjoffset",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)
        f:AdjustPointsOffset(5, -3)
        local _, _, _, x, y = f:GetPoint(1)
        XOFS = x; YOFS = y
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return XOFS").unwrap(), 15.0);
    assert_eq!(t.env.eval::<f64>("return YOFS").unwrap(), -13.0);
}

#[test]
fn test_adjust_points_offset_all_anchors() {
    let (t, _) = load_test_lua(
        "layout-adjoffset-all",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", -10, 10)
        f:AdjustPointsOffset(5, 5)
        local _, _, _, x1, y1 = f:GetPoint(1)
        local _, _, _, x2, y2 = f:GetPoint(2)
        X1 = x1; Y1 = y1; X2 = x2; Y2 = y2
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return X1").unwrap(), 15.0);
    assert_eq!(t.env.eval::<f64>("return Y1").unwrap(), -5.0);
    assert_eq!(t.env.eval::<f64>("return X2").unwrap(), -5.0);
    assert_eq!(t.env.eval::<f64>("return Y2").unwrap(), 15.0);
}

#[test]
fn test_anchor_cycle_detection() {
    let (t, _) = load_test_lua(
        "layout-cycle",
        r#"
        local a = CreateFrame("Frame", "CycleA", UIParent)
        local b = CreateFrame("Frame", "CycleB", UIParent)
        a:SetSize(50, 30); b:SetSize(50, 30)
        a:SetPoint("CENTER", b, "CENTER")
        CYCLE_ERR = not pcall(function() b:SetPoint("CENTER", a, "CENTER") end)
    "#,
    );
    t.assert_lua_true("return CYCLE_ERR", "cycle should error");
}

#[test]
fn test_anchor_cycle_detection_chain() {
    let (t, _) = load_test_lua(
        "layout-cycle-chain",
        r#"
        local a = CreateFrame("Frame", "ChainA2", UIParent)
        local b = CreateFrame("Frame", "ChainB2", UIParent)
        local c = CreateFrame("Frame", "ChainC2", UIParent)
        a:SetSize(50, 30); b:SetSize(50, 30); c:SetSize(50, 30)
        a:SetPoint("CENTER", b, "CENTER")
        b:SetPoint("CENTER", c, "CENTER")
        CYCLE_ERR = not pcall(function() c:SetPoint("CENTER", a, "CENTER") end)
    "#,
    );
    t.assert_lua_true("return CYCLE_ERR", "transitive cycle should error");
}

#[test]
fn test_set_point_preserves_cached_validity_until_position_query() {
    let (t, _) = load_test_lua(
        "layout-dirty-sp",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30)
        f:SetPoint("CENTER")
        f:GetLeft()
        VALID_BEFORE = f:IsRectValid()
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 5, -5)
        VALID_CACHED = f:IsRectValid()
        LEFT_AFTER = f:GetLeft()
        VALID_AFTER_QUERY = f:IsRectValid()
    "#,
    );
    t.assert_lua_true("return VALID_BEFORE", "valid after geometry resolution");
    t.assert_lua_true("return VALID_CACHED", "IsRectValid does not resolve layout");
    assert_eq!(t.env.eval::<f64>("return LEFT_AFTER").unwrap(), 5.0);
    t.assert_lua_true("return VALID_AFTER_QUERY", "position query caches validity");
}
