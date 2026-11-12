//! Tests for size: SetSize, SetWidth, SetHeight, GetSize, GetWidth, GetHeight,
//! explicit vs computed, dirty tracking.

use super::*;

#[test]
fn test_size_initial_zero() {
    let (t, _) = load_test_lua(
        "layout-size-init",
        r#"
        local f = CreateFrame("Frame")
        W = f:GetWidth()
        H = f:GetHeight()
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 0.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 0.0);
}

#[test]
fn test_set_size_and_get() {
    let (t, _) = load_test_lua(
        "layout-setsize",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(100, 50)
        local w, h = f:GetSize()
        W = w; H = h
        COUNT = select('#', f:GetSize())
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 100.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 50.0);
    assert_eq!(t.env.eval::<i32>("return COUNT").unwrap(), 2);
}

#[test]
fn test_set_width_only() {
    let (t, _) = load_test_lua(
        "layout-setwidth",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(100, 50)
        f:SetWidth(200)
        W = f:GetWidth(); H = f:GetHeight()
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 200.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 50.0);
}

#[test]
fn test_set_height_only() {
    let (t, _) = load_test_lua(
        "layout-setheight",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(100, 50)
        f:SetHeight(75)
        W = f:GetWidth(); H = f:GetHeight()
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 100.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 75.0);
}

#[test]
fn test_get_width_true_returns_explicit() {
    let (t, _) = load_test_lua(
        "layout-width-true",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(100, 50)
        W = f:GetWidth(); W_TRUE = f:GetWidth(true)
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 100.0);
    assert_eq!(t.env.eval::<f64>("return W_TRUE").unwrap(), 100.0);
}

#[test]
fn test_get_height_true_returns_explicit() {
    let (t, _) = load_test_lua(
        "layout-height-true",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(100, 50)
        H = f:GetHeight(); H_TRUE = f:GetHeight(true)
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 50.0);
    assert_eq!(t.env.eval::<f64>("return H_TRUE").unwrap(), 50.0);
}

#[test]
fn test_set_size_preserves_cached_validity_until_position_query() {
    let (t, _) = load_test_lua(
        "layout-size-dirty",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30); f:SetPoint("CENTER")
        f:GetLeft()
        VALID_BEFORE = f:IsRectValid()
        f:SetSize(100, 60)
        VALID_CACHED = f:IsRectValid()
        LEFT_AFTER = f:GetLeft()
        VALID_AFTER_QUERY = f:IsRectValid()
    "#,
    );
    t.assert_lua_true("return VALID_BEFORE", "valid after geometry resolution");
    t.assert_lua_true("return VALID_CACHED", "IsRectValid reads cached validity");
    assert!(t.env.eval::<f64>("return LEFT_AFTER").unwrap().is_finite());
    t.assert_lua_true(
        "return VALID_AFTER_QUERY",
        "position query refreshes validity",
    );
}

#[test]
fn test_set_size_same_values_preserves_cached_validity() {
    let (t, _) = load_test_lua(
        "layout-size-nodirty",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        f:SetSize(50, 30); f:SetPoint("CENTER")
        f:GetLeft()
        f:SetSize(50, 30)
        STILL_VALID = f:IsRectValid()
    "#,
    );
    t.assert_lua_true("return STILL_VALID", "IsRectValid reads cached validity");
}

#[test]
fn test_set_size_same_values_no_render_dirty() {
    let (t, _) = load_test_lua(
        "layout-size-no-render-dirty",
        r#"
        local f = CreateFrame("Frame", "SameSizeNoRenderDirtyFrame", UIParent)
        f:SetSize(50, 30)
    "#,
    );

    {
        let state = t.env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
    }

    t.env
        .exec("SameSizeNoRenderDirtyFrame:SetSize(50, 30)")
        .unwrap();

    let (dirty_mask, dirty_ids) = {
        let state = t.env.state().borrow();
        state.widgets.take_render_dirty_with_ids()
    };
    assert_eq!(dirty_mask, 0, "same SetSize should not dirty render state");
    assert!(
        dirty_ids.is_some_and(|ids| ids.is_empty()),
        "same SetSize should not enqueue dirty frame IDs"
    );
}

#[test]
fn test_set_width_same_value_no_render_dirty() {
    let (t, _) = load_test_lua(
        "layout-width-no-render-dirty",
        r#"
        local f = CreateFrame("Frame", "SameWidthNoRenderDirtyFrame", UIParent)
        f:SetSize(50, 30)
    "#,
    );

    {
        let state = t.env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
    }

    t.env
        .exec("SameWidthNoRenderDirtyFrame:SetWidth(50)")
        .unwrap();

    let (dirty_mask, dirty_ids) = {
        let state = t.env.state().borrow();
        state.widgets.take_render_dirty_with_ids()
    };
    assert_eq!(dirty_mask, 0, "same SetWidth should not dirty render state");
    assert!(
        dirty_ids.is_some_and(|ids| ids.is_empty()),
        "same SetWidth should not enqueue dirty frame IDs"
    );
}

#[test]
fn test_set_height_same_value_no_render_dirty() {
    let (t, _) = load_test_lua(
        "layout-height-no-render-dirty",
        r#"
        local f = CreateFrame("Frame", "SameHeightNoRenderDirtyFrame", UIParent)
        f:SetSize(50, 30)
    "#,
    );

    {
        let state = t.env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
    }

    t.env
        .exec("SameHeightNoRenderDirtyFrame:SetHeight(30)")
        .unwrap();

    let (dirty_mask, dirty_ids) = {
        let state = t.env.state().borrow();
        state.widgets.take_render_dirty_with_ids()
    };
    assert_eq!(
        dirty_mask, 0,
        "same SetHeight should not dirty render state"
    );
    assert!(
        dirty_ids.is_some_and(|ids| ids.is_empty()),
        "same SetHeight should not enqueue dirty frame IDs"
    );
}

#[test]
fn test_child_hide_does_not_replace_layout_parent_onupdate() {
    let (t, _) = load_test_lua(
        "test-child-hide-keeps-layout-parent-onupdate",
        r#"
        local parent = CreateFrame("Frame", "HideLayoutParentFrame", UIParent)
        local child = CreateFrame("Frame", "HideLayoutChildFrame", parent)
        local customUpdate = function() end
        local layoutUpdate = function() end

        function parent:IsLayoutFrame()
            return true
        end

        function parent:MarkDirty()
            self:SetScript("OnUpdate", self.OnUpdate)
        end

        parent.OnUpdate = layoutUpdate
        parent:SetScript("OnUpdate", customUpdate)
        parent:Hide()

        _G.HideLayoutParentKeptCustomUpdate = parent:GetScript("OnUpdate") == customUpdate
    "#,
    );

    t.assert_lua_true(
        "return HideLayoutParentKeptCustomUpdate",
        "hiding a layout parent should not let child visibility dispatch replace its custom OnUpdate script",
    );
}

#[test]
fn test_show_does_not_replace_layout_frame_onupdate() {
    let (t, _) = load_test_lua(
        "test-show-keeps-layout-frame-onupdate",
        r#"
        local frame = CreateFrame("Frame", "ShowLayoutFrame", UIParent)
        local customUpdate = function() end
        local layoutUpdate = function() end

        function frame:IsLayoutFrame()
            return true
        end

        function frame:MarkDirty()
            self:SetScript("OnUpdate", self.OnUpdate)
        end

        frame:Hide()
        frame.OnUpdate = layoutUpdate
        frame:SetScript("OnUpdate", customUpdate)
        frame:Show()

        _G.ShowLayoutFrameKeptCustomUpdate = frame:GetScript("OnUpdate") == customUpdate
    "#,
    );

    t.assert_lua_true(
        "return ShowLayoutFrameKeptCustomUpdate",
        "showing a layout frame should not replace its custom OnUpdate script",
    );
}

#[test]
fn test_layout_dirty_helper_preserves_custom_onupdate() {
    let (t, _) = load_test_lua(
        "test-layout-dirty-helper-keeps-custom-onupdate",
        r#"
        local frame = CreateFrame("Frame", "DirtyHelperLayoutFrame", UIParent)
        local customUpdate = function() end
        local layoutUpdate = function() end

        function frame:IsLayoutFrame()
            return true
        end

        function frame:MarkDirty()
            self:SetScript("OnUpdate", self.OnUpdate)
        end

        frame.OnUpdate = layoutUpdate
        frame:SetScript("OnUpdate", customUpdate)
        __wow_mark_layout_frame_dirty(frame)

        _G.DirtyHelperKeptCustomUpdate = frame:GetScript("OnUpdate") == customUpdate
    "#,
    );

    t.assert_lua_true(
        "return DirtyHelperKeptCustomUpdate",
        "simulator layout dirty helper should preserve an already installed custom OnUpdate script",
    );
}

#[test]
fn test_layout_dirty_preserves_custom_parent_onupdate_through_recursive_markdirty() {
    let (t, _) = load_test_lua(
        "test-layout-dirty-keeps-recursive-parent-onupdate",
        r#"
        local parent = CreateFrame("Frame", "DirtyRecursiveParentFrame", UIParent)
        local child = CreateFrame("Frame", "DirtyRecursiveChildFrame", parent)
        local grandchild = CreateFrame("Frame", "DirtyRecursiveGrandchildFrame", child)
        local customUpdate = function() end
        local parentLayoutUpdate = function() end
        local childLayoutUpdate = function() end

        function parent:IsLayoutFrame()
            return true
        end

        function parent:MarkDirty()
            self:SetScript("OnUpdate", self.OnUpdate)
        end

        function child:IsLayoutFrame()
            return true
        end

        function child:MarkDirty()
            self:SetScript("OnUpdate", self.OnUpdate)
            local frameParent = self:GetParent()
            while frameParent do
                if frameParent.IsLayoutFrame and frameParent:IsLayoutFrame() then
                    frameParent:MarkDirty()
                    return
                end
                frameParent = frameParent:GetParent()
            end
        end

        parent.OnUpdate = parentLayoutUpdate
        child.OnUpdate = childLayoutUpdate
        parent:SetScript("OnUpdate", customUpdate)
        grandchild:SetWidth(10)

        _G.DirtyRecursiveParentKeptCustomUpdate = parent:GetScript("OnUpdate") == customUpdate
    "#,
    );

    t.assert_lua_true(
        "return DirtyRecursiveParentKeptCustomUpdate",
        "simulator layout dirtying should preserve custom OnUpdate handlers clobbered by recursive parent MarkDirty calls",
    );
}

#[test]
fn test_zero_size() {
    let (t, _) = load_test_lua(
        "layout-zero-size",
        r#"
        local f = CreateFrame("Frame")
        f:SetSize(0, 0)
        W = f:GetWidth(); H = f:GetHeight()
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 0.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 0.0);
}

#[test]
fn test_width_from_opposite_anchors() {
    let (t, _) = load_test_lua(
        "layout-width-opp",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 100); parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, parent)
        child:SetPoint("LEFT", parent, "LEFT", 10, 0)
        child:SetPoint("RIGHT", parent, "RIGHT", -10, 0)
        W = child:GetWidth()
    "#,
    );
    let w = t.env.eval::<f64>("return W").unwrap();
    assert!((w - 180.0).abs() < 0.01, "expected 180, got {}", w);
}

#[test]
fn test_height_from_opposite_anchors() {
    let (t, _) = load_test_lua(
        "layout-height-opp",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 100); parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, parent)
        child:SetPoint("TOP", parent, "TOP", 0, -5)
        child:SetPoint("BOTTOM", parent, "BOTTOM", 0, 5)
        H = child:GetHeight()
    "#,
    );
    let h = t.env.eval::<f64>("return H").unwrap();
    assert!((h - 90.0).abs() < 0.01, "expected 90, got {}", h);
}

#[test]
fn test_explicit_vs_computed_width() {
    let (t, _) = load_test_lua(
        "layout-explicit-vs-computed",
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(200, 100); parent:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, parent)
        child:SetSize(50, 30)
        child:SetPoint("LEFT", parent, "LEFT", 10, 0)
        child:SetPoint("RIGHT", parent, "RIGHT", -10, 0)
        W_COMPUTED = child:GetWidth()
        W_EXPLICIT = child:GetWidth(true)
    "#,
    );
    let computed = t.env.eval::<f64>("return W_COMPUTED").unwrap();
    let explicit = t.env.eval::<f64>("return W_EXPLICIT").unwrap();
    assert!((computed - 180.0).abs() < 0.01, "computed: {}", computed);
    assert_eq!(explicit, 50.0);
}
