//! Tests for methods_anchor.rs: SetPoint, ClearAllPoints, GetPoint, GetNumPoints,
//! SetAllPoints, AdjustPointsOffset, GetPointByName.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SetPoint / GetPoint / GetNumPoints
// ============================================================================

#[test]
fn test_set_point_basic() {
    let env = env();
    env.exec(r#"
        local f = CreateFrame("Frame", "AnchorFrame1", UIParent)
        f:SetPoint("CENTER", UIParent, "CENTER", 10, 20)
    "#).unwrap();

    let num: i32 = env.eval("return AnchorFrame1:GetNumPoints()").unwrap();
    assert_eq!(num, 1);
}

#[test]
fn test_get_point_returns_values() {
    let env = env();
    // GetPoint returns (point, relativeTo, relativePoint, x, y) where relativeTo is a frame/nil
    // Do assertions in Lua and return offsets to Rust
    let (x, y): (f64, f64) = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorFrame2", UIParent)
        f:SetPoint("TOPLEFT", UIParent, "BOTTOMRIGHT", 5, -10)
        local point, relTo, relPoint, x, y = f:GetPoint(1)
        assert(point == "TOPLEFT", "point should be TOPLEFT, got " .. tostring(point))
        assert(relPoint == "BOTTOMRIGHT", "relPoint should be BOTTOMRIGHT, got " .. tostring(relPoint))
        return x, y
    "#).unwrap();
    assert!((x - 5.0).abs() < 0.01);
    assert!((y - (-10.0)).abs() < 0.01);
}

#[test]
fn test_set_point_default_relative_point() {
    let env = env();
    env.exec(r#"
        local f = CreateFrame("Frame", "AnchorFrame3", UIParent)
        f:SetPoint("CENTER")
        local point, relTo, relPoint = f:GetPoint(1)
        assert(point == "CENTER", "point should be CENTER, got " .. tostring(point))
        assert(relPoint == "CENTER", "relativePoint should default to CENTER, got " .. tostring(relPoint))
    "#).unwrap();
}

#[test]
fn test_set_point_multiple() {
    let env = env();
    let num: i32 = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorFrame4", UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", 0, 0)
        return f:GetNumPoints()
    "#).unwrap();
    assert_eq!(num, 2);
}

#[test]
fn test_set_point_replaces_same_point() {
    let env = env();
    let x: f64 = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorFrame5", UIParent)
        f:SetPoint("CENTER", UIParent, "CENTER", 10, 0)
        f:SetPoint("CENTER", UIParent, "CENTER", 99, 0)
        local _, _, _, xOfs = f:GetPoint(1)
        return xOfs
    "#).unwrap();
    assert!((x - 99.0).abs() < 0.01, "Second SetPoint should replace the first");
}

// ============================================================================
// ClearAllPoints
// ============================================================================

#[test]
fn test_clear_all_points() {
    let env = env();
    let num: i32 = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorClear", UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", 0, 0)
        f:ClearAllPoints()
        return f:GetNumPoints()
    "#).unwrap();
    assert_eq!(num, 0);
}

// ============================================================================
// SetAllPoints
// ============================================================================

#[test]
fn test_set_all_points() {
    let env = env();
    let num: i32 = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorAll", UIParent)
        f:SetAllPoints(UIParent)
        return f:GetNumPoints()
    "#).unwrap();
    assert_eq!(num, 2, "SetAllPoints should add TOPLEFT and BOTTOMRIGHT");
}

#[test]
fn test_set_all_points_offsets_zero() {
    let env = env();
    let (x1, y1): (f64, f64) = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorAllOff", UIParent)
        f:SetAllPoints(UIParent)
        local _, _, _, x, y = f:GetPoint(1)
        return x, y
    "#).unwrap();
    assert_eq!(x1, 0.0);
    assert_eq!(y1, 0.0);
}

// ============================================================================
// AdjustPointsOffset
// ============================================================================

#[test]
fn test_adjust_points_offset() {
    let env = env();
    let (x, y): (f64, f64) = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorAdj", UIParent)
        f:SetPoint("CENTER", UIParent, "CENTER", 10, 20)
        f:AdjustPointsOffset(5, -3)
        local _, _, _, xOfs, yOfs = f:GetPoint(1)
        return xOfs, yOfs
    "#).unwrap();
    assert!((x - 15.0).abs() < 0.01, "x should be 10+5=15, got {}", x);
    assert!((y - 17.0).abs() < 0.01, "y should be 20+(-3)=17, got {}", y);
}

#[test]
fn test_adjust_points_offset_multiple_anchors() {
    let env = env();
    let (x2, y2): (f64, f64) = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorAdjMulti", UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", 0, 0)
        f:AdjustPointsOffset(10, 10)
        local _, _, _, x, y = f:GetPoint(2)
        return x, y
    "#).unwrap();
    assert!((x2 - 10.0).abs() < 0.01);
    assert!((y2 - 10.0).abs() < 0.01);
}

// ============================================================================
// GetPointByName
// ============================================================================

#[test]
fn test_get_point_by_name() {
    let env = env();
    let (x, y): (f64, f64) = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorByName", UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 5, 10)
        f:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", -5, -10)
        local point, relTo, relPoint, xOfs, yOfs = f:GetPointByName("BOTTOMRIGHT")
        assert(point == "BOTTOMRIGHT", "point should be BOTTOMRIGHT")
        assert(relPoint == "BOTTOMRIGHT", "relPoint should be BOTTOMRIGHT")
        return xOfs, yOfs
    "#).unwrap();
    assert!((x - (-5.0)).abs() < 0.01);
    assert!((y - (-10.0)).abs() < 0.01);
}

#[test]
fn test_get_point_by_name_not_found() {
    let env = env();
    let is_nil: bool = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorByNameNil", UIParent)
        f:SetPoint("CENTER")
        return f:GetPointByName("TOPLEFT") == nil
    "#).unwrap();
    assert!(is_nil, "GetPointByName should return nil for non-existent anchor");
}

// ============================================================================
// Cycle detection
// ============================================================================

#[test]
fn test_set_point_self_reference_no_crash() {
    let env = env();
    // SetPoint to self should not cause infinite recursion
    env.exec(r#"
        local f = CreateFrame("Frame", "AnchorSelf", UIParent)
        f:SetPoint("CENTER", f, "CENTER", 0, 0)
    "#).unwrap();
}

// ============================================================================
// GetNumPoints default
// ============================================================================

#[test]
fn test_get_num_points_default_zero() {
    let env = env();
    let num: i32 = env.eval(r#"
        local f = CreateFrame("Frame", "AnchorNum", UIParent)
        return f:GetNumPoints()
    "#).unwrap();
    assert_eq!(num, 0);
}
