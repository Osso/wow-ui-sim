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
