//! Tests for frame hierarchy methods: GetChildren, GetNumChildren, GetRegions.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn set_parent_rejects_self_without_changing_hierarchy() {
    let env = env();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", nil, UIParent)
        local child = CreateFrame("Frame", nil, owner)
        local ok, err = pcall(owner.SetParent, owner, owner)
        assert(not ok and type(err) == "string" and err:find("cycle"), tostring(err))
        assert(owner:GetParent() == UIParent)
        assert(child:GetParent() == owner)
        assert(owner:GetNumChildren() == 1 and owner:GetChildren() == child)
        assert(owner:IsVisible() and child:IsVisible())
        "#,
    ).unwrap();
}

#[test]
fn set_parent_rejects_descendant_without_changing_hierarchy() {
    let env = env();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", nil, UIParent)
        local child = CreateFrame("Frame", nil, owner)
        local grandchild = CreateFrame("Frame", nil, child)
        local ok, err = pcall(owner.SetParent, owner, grandchild)
        assert(not ok and type(err) == "string" and err:find("cycle"), tostring(err))
        assert(owner:GetParent() == UIParent and child:GetParent() == owner)
        assert(grandchild:GetParent() == child)
        assert(owner:GetNumChildren() == 1 and owner:GetChildren() == child)
        assert(child:GetNumChildren() == 1 and child:GetChildren() == grandchild)
        assert(grandchild:GetNumChildren() == 0)
        assert(owner:IsVisible() and grandchild:IsVisible())
        "#,
    ).unwrap();
}

#[test]
fn set_parent_valid_reparent_nil_and_same_parent_preserve_child_counts() {
    let env = env();
    env.exec(
        r#"
        local left = CreateFrame("Frame", nil, UIParent)
        local right = CreateFrame("Frame", nil, UIParent)
        local child = CreateFrame("Frame", nil, left)
        child:SetParent(left)
        assert(left:GetNumChildren() == 1 and left:GetChildren() == child)
        child:SetParent(right)
        assert(child:GetParent() == right and left:GetNumChildren() == 0)
        assert(right:GetNumChildren() == 1 and right:GetChildren() == child)
        child:SetParent(nil)
        assert(child:GetParent() == nil and right:GetNumChildren() == 0)
        child:SetParent(left)
        assert(child:GetParent() == left and left:GetNumChildren() == 1)
        "#,
    ).unwrap();
}

#[test]
fn set_parent_dispatches_effective_visibility_children_first_without_changing_shown() {
    let env = env();
    env.exec(
        r#"
        local visible = CreateFrame("Frame", nil, UIParent)
        local hidden = CreateFrame("Frame", nil, UIParent)
        hidden:Hide()
        local child = CreateFrame("Frame", nil, visible)
        local grandchild = CreateFrame("Frame", nil, child)
        local locallyHidden = CreateFrame("Frame", nil, child)
        locallyHidden:Hide()
        local events = {}
        local function record(frame, name)
            frame:SetScript("OnHide", function(self)
                assert(self:GetParent() == (name == "child" and hidden or child))
                assert(self:IsShown() and not self:IsVisible())
                table.insert(events, name .. ":hide")
            end)
            frame:SetScript("OnShow", function(self)
                assert(self:GetParent() == (name == "child" and visible or child))
                assert(self:IsShown() and self:IsVisible())
                table.insert(events, name .. ":show")
            end)
        end
        record(child, "child")
        record(grandchild, "grandchild")
        locallyHidden:SetScript("OnShow", function() error("hidden child shown") end)
        locallyHidden:SetScript("OnHide", function() error("hidden child hidden") end)
        child:SetParent(hidden)
        assert(child:IsShown() and grandchild:IsShown() and locallyHidden:IsShown() == false)
        assert(not child:IsVisible() and not grandchild:IsVisible())
        child:SetParent(visible)
        assert(child:IsVisible() and grandchild:IsVisible())
        assert(table.concat(events, ",") == "grandchild:hide,child:hide,grandchild:show,child:show", table.concat(events, ","))
        "#,
    ).unwrap();
}

#[test]
fn set_parent_visibility_noops_and_reentry_observe_final_hierarchy() {
    let env = env();
    env.exec(
        r#"
        local left = CreateFrame("Frame", nil, UIParent)
        local right = CreateFrame("Frame", nil, UIParent)
        local hidden = CreateFrame("Frame", nil, UIParent)
        hidden:Hide()
        local alphaZero = CreateFrame("Frame", nil, UIParent)
        alphaZero:SetAlpha(0)
        local child = CreateFrame("Frame", nil, left)
        local count = 0
        local staleHides = 0
        local grandchild = CreateFrame("Frame", nil, child)
        grandchild:SetScript("OnHide", function()
            if count == 0 then child:SetParent(right) end
        end)
        child:SetScript("OnHide", function() staleHides = staleHides + 1 end)
        child:SetScript("OnShow", function() count = count + 1 end)
        child:SetParent(left)
        child:SetParent(right)
        child:SetParent(alphaZero)
        child:SetParent(hidden)
        assert(count == 1 and staleHides == 0, "redirect must suppress stale parent hide")
        assert(child:GetParent() == right and child:IsVisible() and child:IsShown())
        child:SetParent(hidden)
        assert(count == 1 and staleHides == 1 and not child:IsVisible())
        child:SetParent(hidden)
        assert(count == 1 and staleHides == 1)
        child:Hide()
        child:SetParent(left)
        assert(count == 1 and staleHides == 1 and not child:IsVisible() and not child:IsShown())
        "#,
    ).unwrap();
}

#[test]
fn set_parent_handler_error_continues_to_parent_binding() {
    let env = env();
    env.exec(
        r#"
        local visible = CreateFrame("Frame", nil, UIParent)
        local hidden = CreateFrame("Frame", nil, UIParent)
        hidden:Hide()
        local child = CreateFrame("Frame", nil, visible)
        local grandchild = CreateFrame("Frame", nil, child)
        local calls = {}
        grandchild:SetScript("OnHide", function()
            table.insert(calls, "grandchild")
            error("reparent hide failure")
        end)
        child:SetScript("OnHide", function() table.insert(calls, "child") end)
        child:SetParent(hidden)
        assert(table.concat(calls, ",") == "grandchild,child", table.concat(calls, ","))
        "#,
    ).unwrap();
    assert!(env.state().borrow().lua_errors.iter().any(|error| error.contains("reparent hide failure")));
}

#[test]
fn get_children_excludes_regions() {
    let env = env();
    env.exec(
        r#"
        local parent = CreateFrame("Frame", "HierarchyParent", UIParent)
        local child = CreateFrame("Frame", "HierarchyChild", parent)
        local tex = parent:CreateTexture("HierarchyTexture", "ARTWORK")
        local text = parent:CreateFontString("HierarchyFontString", "ARTWORK")

        assert(parent:GetNumChildren() == 1, "regions should not count as children")
        assert(parent:GetNumRegions() == 2, "texture and font string should count as regions")

        local onlyChild = parent:GetChildren()
        assert(onlyChild == child, "GetChildren should return only child frames")

        local firstRegion, secondRegion = parent:GetRegions()
        assert(firstRegion == tex, "GetRegions should include texture regions")
        assert(secondRegion == text, "GetRegions should include font string regions")
    "#,
    )
    .unwrap();
}
