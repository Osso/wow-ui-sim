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
fn invalid_rect_size_getters_return_stored_dimensions() {
    let env = env();
    let (
        width,
        height,
        size_width,
        size_height,
        individual_width,
        individual_height,
        rect_valid,
    ): (f64, f64, f64, f64, f64, f64, bool) = env
        .eval(
            r##"
            local f = CreateFrame("Frame", "UnanchoredProbe", UIParent)
            f:SetSize(100, 50)
            local width, height = f:GetWidth(), f:GetHeight()
            local size_width, size_height = f:GetSize()

            local individual = CreateFrame("Frame", "IndividualSizeProbe", UIParent)
            individual:SetWidth(100)
            individual:SetHeight(50)
            local individual_width, individual_height = individual:GetSize()
            return width, height, size_width, size_height,
                   individual_width, individual_height, f:IsRectValid()
            "##,
        )
        .expect("invalid geometry returns stored size");

    assert_eq!((width, height), (100.0, 50.0));
    assert_eq!((size_width, size_height), (100.0, 50.0));
    assert_eq!((individual_width, individual_height), (100.0, 50.0));
    assert!(!rect_valid);
}

#[test]
fn valid_anchored_size_queries_use_resolved_dimensions() {
    let env = env();
    let (
        width,
        height,
        size_width,
        size_height,
        left,
        bottom,
        rect_width,
        rect_height,
        valid_after,
    ): (f64, f64, f64, f64, f64, f64, f64, f64, bool) = env
        .eval(
            r#"
            local leftTarget = CreateFrame("Frame", "ResolvedSizeLeftTarget", UIParent)
            leftTarget:SetSize(10, 10)
            leftTarget:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 100, 100)
            local rightTarget = CreateFrame("Frame", "ResolvedSizeRightTarget", UIParent)
            rightTarget:SetSize(10, 10)
            rightTarget:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 150, 100)
            local frame = CreateFrame("Frame", "ResolvedSizeProbe", UIParent)
            frame:SetSize(5, 20)
            frame:SetPoint("BOTTOMLEFT", leftTarget, "BOTTOMRIGHT")
            frame:SetPoint("BOTTOMRIGHT", rightTarget, "BOTTOMLEFT")
            local width = frame:GetWidth()
            local height = frame:GetHeight()
            local size_width, size_height = frame:GetSize()
            local left, bottom, rect_width, rect_height = frame:GetRect()
            local valid_after = frame:IsRectValid()
            return width, height, size_width, size_height, left, bottom,
                   rect_width, rect_height, valid_after
            "#,
        )
        .expect("eval should succeed");

    assert!(valid_after);
    assert_eq!(width, 40.0);
    assert_eq!(height, 20.0);
    assert_eq!(size_width, 40.0);
    assert_eq!(size_height, 20.0);
    assert_eq!(left, 110.0);
    assert_eq!(bottom, 100.0);
    assert_eq!(rect_width, 40.0);
    assert_eq!(rect_height, 20.0);
}

#[test]
fn geometry_getter_resolves_dirty_anchor_target_before_dependent() {
    let env = env();
    let (initial_left, moved_left): (f64, f64) = env
        .eval(
            r##"
            local target = CreateFrame("Frame", "DirtyGeometryTarget", UIParent)
            target:SetSize(30, 20)
            target:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 50, 60)
            local child = CreateFrame("Frame", "DirtyGeometryChild", UIParent)
            child:SetSize(10, 12)
            child:SetPoint("BOTTOMLEFT", target, "TOPRIGHT", 5, 7)

            local initial_left = child:GetLeft()
            target:SetSize(50, 20)
            local moved_left = child:GetLeft()
            return initial_left, moved_left
            "##,
        )
        .expect("position query resolves its dirty anchor target first");

    assert_eq!(initial_left, 85.0);
    assert_eq!(moved_left, 105.0);
}

#[test]
fn geometry_getter_resolves_dirty_anchor_chain_before_dependent() {
    let env = env();
    let (initial_left, moved_left): (f64, f64) = env
        .eval(
            r##"
            local source = CreateFrame("Frame", "DirtyGeometryChainSource", UIParent)
            source:SetSize(20, 20)
            source:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 50, 60)
            local middle = CreateFrame("Frame", "DirtyGeometryChainMiddle", UIParent)
            middle:SetSize(10, 10)
            middle:SetPoint("BOTTOMLEFT", source, "TOPRIGHT")
            local dependent = CreateFrame("Frame", "DirtyGeometryChainDependent", UIParent)
            dependent:SetSize(5, 5)
            dependent:SetPoint("BOTTOMLEFT", middle, "TOPRIGHT")

            local initial_left = dependent:GetLeft()
            source:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 100, 60)
            local moved_left = dependent:GetLeft()
            return initial_left, moved_left
            "##,
        )
        .expect("position query resolves a dirty anchor chain before its dependent");

    assert_eq!(initial_left, 80.0);
    assert_eq!(moved_left, 130.0);
}

#[test]
fn geometry_getter_uses_clean_cached_anchor_target() {
    let env = env();
    let (target_left, child_left): (f64, f64) = env
        .eval(
            r##"
            local target = CreateFrame("Frame", "CleanGeometryTarget", UIParent)
            target:SetSize(30, 20)
            target:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 50, 60)
            local target_left = target:GetLeft()

            local child = CreateFrame("Frame", "CleanGeometryChild", UIParent)
            child:SetSize(10, 12)
            child:SetPoint("BOTTOMLEFT", target, "TOPRIGHT", 5, 7)
            local child_left = child:GetLeft()
            return target_left, child_left
            "##,
        )
        .expect("position query uses the cached valid anchor target");

    assert_eq!(target_left, 50.0);
    assert_eq!(child_left, 85.0);
}

#[test]
fn is_rect_valid_reads_cached_flag_without_resolving_dirty_target() {
    let env = env();
    let (cached_validity, fallback_width, fallback_height, invalid_validity): (
        bool,
        f64,
        f64,
        bool,
    ) = env
        .eval(
            r##"
                local target = CreateFrame("Frame", "CachedValidityTarget", UIParent)
                target:SetSize(30, 20)
                target:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 50, 60)
                local child = CreateFrame("Frame", "CachedValidityChild", UIParent)
                child:SetSize(10, 12)
                child:SetPoint("BOTTOMLEFT", target, "TOPRIGHT", 5, 7)
                child:GetLeft()

                target:ClearAllPoints()
                local cached_validity = child:IsRectValid()
                local fallback_width, fallback_height = child:GetSize()
                local invalid_validity = child:IsRectValid()
                return cached_validity, fallback_width, fallback_height, invalid_validity
                "##,
        )
        .expect("validity query reads cached state before geometry resolution");

    assert!(cached_validity);
    assert_eq!((fallback_width, fallback_height), (10.0, 12.0));
    assert!(!invalid_validity);
}

#[test]
fn child_anchored_to_unanchored_parent_uses_its_stored_size() {
    let env = env();
    let (width, height, rect_valid): (f64, f64, bool) = env
        .eval(
            r##"
            local parent = CreateFrame("Frame", "UnanchoredParentProbe", UIParent)
            parent:SetSize(100, 100)

            local child = CreateFrame("Frame", "ChildOfUnanchoredParentProbe", parent)
            child:SetSize(20, 20)
            child:SetPoint("CENTER", parent, "CENTER", 0, 0)

            local width, height = child:GetSize()
            return width, height, child:IsRectValid()
            "##,
        )
        .expect("invalid child geometry returns its own stored size");

    assert_eq!((width, height), (20.0, 20.0));
    assert!(!rect_valid);
}

#[test]
fn sibling_anchor_geometry_does_not_require_a_containment_parent_rect() {
    // INFERRED by R59; retain as simulator coverage, not native proof.

    let env = env();
    let (
        parent_width,
        parent_height,
        left,
        bottom,
        width,
        height,
        target_valid_after,
        child_valid_after,
    ): (f64, f64, f64, f64, f64, f64, bool, bool) = env
        .eval(
            r##"
            local parent = CreateFrame("Frame", "AnchorGroupParent", UIParent)
            parent:SetSize(100, 80)
            local target = CreateFrame("Frame", "AnchorGroupTarget", UIParent)
            target:SetSize(30, 20)
            target:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 50, 60)
            local child = CreateFrame("Frame", "AnchorGroupChild", parent)
            child:SetSize(10, 12)
            child:SetPoint("BOTTOMLEFT", target, "TOPRIGHT", 5, 7)
            local parent_width, parent_height = parent:GetSize()
            local left = child:GetLeft()
            local bottom = child:GetBottom()
            local width = child:GetWidth()
            local height = child:GetHeight()
            local target_valid_after = target:IsRectValid()
            local child_valid_after = child:IsRectValid()
            return parent_width, parent_height, left, bottom, width, height,
                   target_valid_after, child_valid_after
            "##,
        )
        .expect("eval should succeed");

    assert_eq!(parent_width, 100.0);
    assert_eq!(parent_height, 80.0);
    assert_eq!(left, 85.0);
    assert_eq!(bottom, 87.0);
    assert_eq!(width, 10.0);
    assert_eq!(height, 12.0);
    assert!(target_valid_after);
    assert!(child_valid_after);
}

#[test]
fn set_size_and_individual_size_setters_store_the_same_dimensions() {
    let env = env();
    let (
        set_size_raw_width,
        set_size_raw_height,
        individual_raw_width,
        individual_raw_height,
        set_size_width,
        set_size_height,
        individual_width,
        individual_height,
        set_size_size_width,
        set_size_size_height,
        individual_size_width,
        individual_size_height,
    ): (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r##"
            local set_size = CreateFrame("Frame", "SetSizeEquivalenceProbe", UIParent)
            set_size:SetSize(40, 30)
            set_size:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 10, 20)

            local individual = CreateFrame("Frame", "IndividualSetSizeEquivalenceProbe", UIParent)
            individual:SetWidth(40)
            individual:SetHeight(30)
            individual:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 10, 20)

            local set_size_width, set_size_height = set_size:GetSize()
            local individual_width, individual_height = individual:GetSize()
            return set_size:GetWidth(true), set_size:GetHeight(true),
                   individual:GetWidth(true), individual:GetHeight(true),
                   set_size:GetWidth(), set_size:GetHeight(),
                   individual:GetWidth(), individual:GetHeight(),
                   set_size_width, set_size_height, individual_width, individual_height
            "##,
        )
        .expect("size setters produce queryable geometry");

    assert_eq!(
        (
            set_size_raw_width,
            set_size_raw_height,
            individual_raw_width,
            individual_raw_height,
            set_size_width,
            set_size_height,
            individual_width,
            individual_height,
            set_size_size_width,
            set_size_size_height,
            individual_size_width,
            individual_size_height,
        ),
        (
            40.0, 30.0, 40.0, 30.0, 40.0, 30.0, 40.0, 30.0, 40.0, 30.0, 40.0, 30.0
        )
    );
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
