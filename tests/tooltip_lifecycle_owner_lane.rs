#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tooltip_lifecycle_original_simple_tooltip_uses_bottomright_owner_anchor() {
    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let source =
        std::fs::read_to_string(addons.join("Blizzard_SharedXML/SharedTooltipTemplates.lua"))
            .unwrap();
    env.exec(&source).unwrap();
    env.exec(r#"
        local owner = CreateFrame("Button", "LifecycleTotem", UIParent)
        GameTooltip_ShowSimpleTooltip(GameTooltip, "test", CreateColor(1, 1, 1), false, owner, "ANCHOR_BOTTOMRIGHT")
        local point, relative, relativePoint = GameTooltip:GetPoint(1)
        assert(relative == owner and point == "TOPLEFT" and relativePoint == "BOTTOMRIGHT", "simple tooltip bottomright points")
    "#).unwrap();
}

#[test]
fn tooltip_lifecycle_cursor_modes_follow_cursor_with_native_points() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1000.0, 800.0);
    env.state()
        .borrow_mut()
        .set_mouse_position(Some((100.0, 200.0)));
    env.exec(r#"
        CursorOwner = CreateFrame("Frame", nil, UIParent)
        CursorTip = CreateFrame("GameTooltip", "LifecycleCursorTip", UIParent)
        CursorTip:SetOwner(CursorOwner, "ANCHOR_CURSOR", 7, 9)
        local point, _, relative, x, y = CursorTip:GetPoint(1)
        assert(point == "BOTTOM" and relative == "BOTTOMLEFT" and x == 100 and y == 600, "cursor ignores offsets")
        CursorTip:SetOwner(CursorOwner, "ANCHOR_CURSOR_RIGHT", 7, 9)
        point, _, relative, x, y = CursorTip:GetPoint(1)
        assert(point == "BOTTOMLEFT" and relative == "BOTTOMLEFT" and x == 107 and y == 609, "cursor right uses offsets")
        CursorTip:AddLine("visible")
        CursorTip:Show()
    "#).unwrap();
    env.state()
        .borrow_mut()
        .set_mouse_position(Some((150.0, 250.0)));
    env.exec(r#"
        local point, _, relative, x, y = CursorTip:GetPoint(1)
        assert(point == "BOTTOMLEFT" and relative == "BOTTOMLEFT" and x == 157 and y == 559, "cursor movement updates anchor")
        CursorTip:SetOwner(CursorOwner, "ANCHOR_CURSOR_LEFT", -3, 4)
        point, _, relative, x, y = CursorTip:GetPoint(1)
        assert(point == "BOTTOMRIGHT" and relative == "BOTTOMLEFT" and x == 147 and y == 554, "cursor left uses offsets")
    "#).unwrap();
}

#[test]
fn tooltip_lifecycle_reownership_clears_before_assigning_new_owner() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local a = CreateFrame("Frame", nil, UIParent)
        local b = CreateFrame("Frame", nil, UIParent)
        local t = CreateFrame("GameTooltip", "LifecycleOwnerTip", UIParent)
        t:SetOwner(a, "ANCHOR_NONE")
        t:AddLine("old")
        local line = t:GetLeftLine(1)
        t:Show()
        local events = {}
        t:SetScript("OnTooltipCleared", function(self)
            events[#events + 1] = self:GetOwner() == a and "clear-old" or "clear-other"
        end)
        t:SetOwner(a, "ANCHOR_NONE")
        assert(t:IsShown(), "same owner retains visibility")
        assert(t:NumLines() == 0 and not line:IsShown(), "cached line cleared and hidden")
        assert(events[1] == "clear-old", "clear occurs with old owner")
        t:ClearLines()
        assert(#events == 1, "empty clear has no notification")
        t:AddLine("another")
        t:SetOwner(b, "ANCHOR_NONE")
        assert(not t:IsShown(), "different owner hides tooltip")
        assert(events[2] == "clear-old", "reassignment clears before changing owner")
        assert(t:GetOwner() == b, "new owner assigned")
    "#,
    )
    .unwrap();
}
