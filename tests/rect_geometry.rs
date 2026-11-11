//! Pins the rect / geometry query family ported from master-era
//! `methods_rect.rs` onto the live rilua registrar. Each test creates
//! a frame, anchors it into UIParent at a known position, then asserts
//! the returned WoW-coord values match the expected layout.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

/// Anchor a 100×50 frame at TOPLEFT + (10, -20) and read back every
/// rect query in one eval. The TOPLEFT anchor puts the frame's top
/// edge 20 below the top of UIParent in screen space — which, in the
/// WoW UI coord system (origin bottom-left), means the top edge sits
/// at `screen_height - 20` (before effective-scale division).
#[test]
fn rect_family_returns_consistent_coords_for_topleft_anchored_frame() {
    let env = env();
    let (left, bottom, width, height, get_left, get_right, get_top, get_bottom, cx, cy): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "RectProbe", UIParent)
            f:SetSize(100, 50)
            f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -20)
            local left, bottom, width, height = f:GetRect()
            local get_left = f:GetLeft()
            local get_right = f:GetRight()
            local get_top = f:GetTop()
            local get_bottom = f:GetBottom()
            local cx, cy = f:GetCenter()
            return left, bottom, width, height,
                   get_left, get_right, get_top, get_bottom, cx, cy
            "#,
        )
        .expect("eval should succeed");

    // Position/size come back intact after coord-system conversion.
    assert_eq!(width, 100.0);
    assert_eq!(height, 50.0);
    assert_eq!(left, 10.0, "GetRect.left should match TOPLEFT.x offset");
    assert_eq!(get_left, left);
    assert_eq!(get_right, left + width);
    // GetRect.bottom + height == GetTop.
    assert_eq!(get_top, bottom + height);
    assert_eq!(get_bottom, bottom);
    // Center is mid-rect.
    assert_eq!(cx, left + width / 2.0);
    assert_eq!(cy, bottom + height / 2.0);
}

#[test]
fn get_size_returns_width_height_pair() {
    let env = env();
    let (w, h): (f64, f64) = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "SizeProbe", UIParent)
            f:SetSize(123, 77)
            f:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
            return f:GetSize()
            "#,
        )
        .expect("eval should succeed");
    assert_eq!(w, 123.0);
    assert_eq!(h, 77.0);
}

#[test]
fn scaled_rect_is_unscaled_layout_rect() {
    // When the frame inherits UIParent's effective scale (the default
    // 1.0), GetScaledRect width/height match GetRect's. The bottom
    // edge in GetScaledRect uses screen-space coords (no scale
    // division) so it differs from GetRect.bottom only when scale ≠ 1.
    let env = env();
    let (rect_left, rect_bottom, rect_w, rect_h, scaled_left, scaled_bottom, scaled_w, scaled_h): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "ScaledRectProbe", UIParent)
            f:SetSize(40, 30)
            f:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 5, 5)
            local rl, rb, rw, rh = f:GetRect()
            local sl, sb, sw, sh = f:GetScaledRect()
            return rl, rb, rw, rh, sl, sb, sw, sh
            "#,
        )
        .expect("eval should succeed");
    assert_eq!(rect_w, scaled_w);
    assert_eq!(rect_h, scaled_h);
    assert_eq!(rect_left, scaled_left);
    assert_eq!(rect_bottom, scaled_bottom);
}

#[test]
fn rect_queries_return_nothing_for_unanchored_frame() {
    let env = env();
    let (rect_values, width, height, size_width, size_height, rect_valid): (
        f64, f64, f64, f64, f64, bool,
    ) = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "UnanchoredProbe", UIParent)
            f:SetSize(100, 50)
            local rect_values = select("#", f:GetRect())
            local size_width, size_height = f:GetSize()
            return rect_values, f:GetWidth(), f:GetHeight(), size_width, size_height, f:IsRectValid()
            "#,
        )
        .expect("eval should succeed");

    assert_eq!(rect_values, 0.0);
    assert_eq!(width, 100.0);
    assert_eq!(height, 50.0);
    assert_eq!(size_width, 100.0);
    assert_eq!(size_height, 50.0);
    assert!(!rect_valid);
}

#[test]
fn child_anchored_to_unanchored_parent_has_no_queryable_rect() {
    let env = env();
    let result: f64 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "UnanchoredParentProbe", UIParent)
            parent:SetSize(100, 100)

            local child = CreateFrame("Frame", "ChildOfUnanchoredParentProbe", parent)
            child:SetSize(20, 20)
            child:SetPoint("CENTER", parent, "CENTER", 0, 0)

            return select("#", child:GetRect())
            "#,
        )
        .expect("eval should succeed");
    assert_eq!(result, 0.0);
}

#[test]
fn child_rect_queries_follow_anchor_targets_not_unanchored_parent() {
    let env = env();
    let (
        parent_rect_values,
        parent_width,
        parent_height,
        child_rect_values,
        left,
        bottom,
        width,
        height,
        scaled_rect_values,
        center_values,
        center_x,
        center_y,
        child_width,
        child_height,
    ): (
        f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64,
    ) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "AnchorGroupParent", UIParent)
            parent:SetSize(100, 80)
            local target = CreateFrame("Frame", "AnchorGroupTarget", UIParent)
            target:SetSize(30, 20)
            target:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 50, 60)
            local child = CreateFrame("Frame", "AnchorGroupChild", parent)
            child:SetSize(10, 12)
            child:SetPoint("BOTTOMLEFT", target, "TOPRIGHT", 5, 7)

            local parent_rect_values = select("#", parent:GetRect())
            local parent_width, parent_height = parent:GetSize()
            local child_rect_values = select("#", child:GetRect())
            local left, bottom, width, height = child:GetRect()
            local scaled_rect_values = select("#", child:GetScaledRect())
            local center_values = select("#", child:GetCenter())
            local center_x, center_y = child:GetCenter()
            local child_width, child_height = child:GetSize()
            return parent_rect_values, parent_width, parent_height, child_rect_values,
                   left, bottom, width, height, scaled_rect_values, center_values,
                   center_x, center_y, child_width, child_height
            "#,
        )
        .expect("eval should succeed");

    assert_eq!(parent_rect_values, 0.0);
    assert_eq!(parent_width, 100.0);
    assert_eq!(parent_height, 80.0);
    assert_eq!(child_rect_values, 4.0);
    assert_eq!(left, 85.0);
    assert_eq!(bottom, 87.0);
    assert_eq!(width, 10.0);
    assert_eq!(height, 12.0);
    assert_eq!(scaled_rect_values, 4.0);
    assert_eq!(center_values, 2.0);
    assert_eq!(center_x, 90.0);
    assert_eq!(center_y, 93.0);
    assert_eq!(child_width, 10.0);
    assert_eq!(child_height, 12.0);
}

#[test]
fn waypoint_style_fit_content_skips_child_without_queryable_ancestor_chain() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "WaypointFitParentProbe", UIParent)
            parent:SetSize(100, 100)

            local child = CreateFrame("Frame", "WaypointFitChildProbe", parent)
            child:SetSize(20, 20)
            child:SetPoint("CENTER", parent, "CENTER", 0, 0)

            local parentLeft = parent:GetLeft()
            local childLeft = child:GetRect()
            if childLeft then
                return tostring(childLeft - parentLeft)
            end
            return "skipped"
            "#,
        )
        .expect("eval should succeed");
    assert_eq!(result, "skipped");
}
