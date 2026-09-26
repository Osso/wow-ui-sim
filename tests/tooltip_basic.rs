//! Tests for GameTooltip implementation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_gametooltip_exists_and_has_correct_type() {
    let env = WowLuaEnv::new().unwrap();

    let exists: bool = env.eval("return GameTooltip ~= nil").unwrap();
    assert!(exists);

    let obj_type: String = env.eval("return GameTooltip:GetObjectType()").unwrap();
    assert_eq!(obj_type, "GameTooltip");
}

#[test]
fn test_gametooltip_strata_is_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    let strata: String = env.eval("return GameTooltip:GetFrameStrata()").unwrap();
    assert_eq!(strata, "TOOLTIP");
}

#[test]
fn test_addline_and_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("First line")
        GameTooltip:AddLine("Second line", 1, 0, 0)
        GameTooltip:AddLine("Third line", 0, 1, 0, true)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_set_shapeshift_populates_spell_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    let (name, spell_id, data_id, lines): (String, i32, i32, i32) = env
        .eval(
            r#"
            local data = C_TooltipInfo.GetShapeshift(1)
            GameTooltip:SetShapeshift(1)
            local name, spellID = GameTooltip:GetSpell()
            return name, spellID, data.id, GameTooltip:NumLines()
            "#,
        )
        .unwrap();

    assert_eq!(name, "Devotion Aura");
    assert_eq!(spell_id, 465);
    assert_eq!(data_id, 465);
    assert_eq!(lines, 1);
}

#[test]
fn test_adddoubleline_and_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddDoubleLine("Left", "Right")
        GameTooltip:AddDoubleLine("Name", "Value", 1, 1, 1, 0.5, 0.5, 0.5)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_clearlines_resets_count() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Line 1")
        GameTooltip:AddLine("Line 2")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_settext_clears_and_sets_first_line() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Old line 1")
        GameTooltip:AddLine("Old line 2")
        GameTooltip:SetText("New text")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 1, "SetText should clear existing lines and add one");
}

#[test]
fn test_setowner_and_isowned_and_getowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "TooltipOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let is_owned: bool = env
        .eval("return GameTooltip:IsOwned(TooltipOwner)")
        .unwrap();
    assert!(is_owned, "GameTooltip should be owned by TooltipOwner");

    let owner_name: String = env.eval("return GameTooltip:GetOwner():GetName()").unwrap();
    assert_eq!(owner_name, "TooltipOwner");

    // Check that non-owner returns false
    env.exec(r#"local other = CreateFrame("Frame", "OtherFrame", UIParent)"#)
        .unwrap();
    let not_owned: bool = env.eval("return GameTooltip:IsOwned(OtherFrame)").unwrap();
    assert!(!not_owned, "GameTooltip should not be owned by OtherFrame");
}

#[test]
fn tooltip_owner_hide_releases_owner_without_clearing_lines() {
    let env = WowLuaEnv::new().unwrap();
    let (shown, owner, owned, lines): (bool, bool, bool, i32) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:AddLine("Content")
            GameTooltip:Show()
            GameTooltip:Hide()
            return GameTooltip:IsShown(), GameTooltip:GetOwner() == nil,
                GameTooltip:IsOwned(owner), GameTooltip:NumLines()
            "#,
        )
        .unwrap();
    assert!(!shown);
    assert!(owner, "Hide must release tooltip ownership");
    assert!(!owned);
    assert_eq!(lines, 1, "Hide must not clear tooltip lines");
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name("GameTooltip").unwrap();
    assert_eq!(sim.widgets.get(id).unwrap().tooltip_owner_id, None);
    assert_eq!(sim.tooltips.get(&id).unwrap().owner_id, None);
}

#[test]
fn tooltip_owner_set_shown_false_releases_owner() {
    let env = WowLuaEnv::new().unwrap();
    let (shown, owner, owned): (bool, bool, bool) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:AddLine("Content")
            GameTooltip:Show()
            GameTooltip:SetShown(false)
            return GameTooltip:IsShown(), GameTooltip:GetOwner() == nil, GameTooltip:IsOwned(owner)
            "#,
        )
        .unwrap();
    assert!(!shown);
    assert!(owner);
    assert!(!owned);
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name("GameTooltip").unwrap();
    assert_eq!(sim.tooltips.get(&id).unwrap().owner_id, None);
}

#[test]
fn tooltip_owner_clear_lines_retains_owner() {
    let env = WowLuaEnv::new().unwrap();
    let (lines, owned, owner): (i32, bool, bool) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:AddLine("Content")
            GameTooltip:ClearLines()
            return GameTooltip:NumLines(), GameTooltip:IsOwned(owner), GameTooltip:GetOwner() == owner
            "#,
        )
        .unwrap();
    assert_eq!(lines, 0);
    assert!(owned && owner);
}

#[test]
fn tooltip_owner_normal_frame_hide_does_not_change_tooltip_owner() {
    let env = WowLuaEnv::new().unwrap();
    let (shown, owned): (bool, bool) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:AddLine("Content")
            owner:Show()
            owner:Hide()
            return owner:IsShown(), GameTooltip:IsOwned(owner)
            "#,
        )
        .unwrap();
    assert!(!shown);
    assert!(owned);
}

#[test]
fn test_getanchortype_after_setowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_BOTTOMRIGHT")
    "#,
    )
    .unwrap();

    let anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(anchor, "ANCHOR_BOTTOMRIGHT");
}

#[test]
fn test_on_tooltip_cleared_fires_on_clearlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.tooltip_cleared_count = 0
        GameTooltip:SetScript("OnTooltipCleared", function()
            _G.tooltip_cleared_count = _G.tooltip_cleared_count + 1
        end)
        GameTooltip:AddLine("Some line")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.tooltip_cleared_count").unwrap();
    assert_eq!(count, 1, "OnTooltipCleared should fire once on ClearLines");
}

#[test]
fn test_on_tooltip_cleared_fires_on_setowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.cleared_count = 0
        GameTooltip:SetScript("OnTooltipCleared", function()
            _G.cleared_count = _G.cleared_count + 1
        end)
        local owner = CreateFrame("Frame", "ClearedTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.cleared_count").unwrap();
    assert_eq!(count, 1, "OnTooltipCleared should fire on SetOwner");
}

#[test]
fn test_set_frame_stack_populates_lines_returns_frame_and_fires_script() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "FrameStackParent", UIParent)
        local child = CreateFrame("Frame", "FrameStackChild", parent)
        _G.frame_stack_highlight = nil
        GameTooltip:SetScript("OnTooltipSetFramestack", function(self, highlightFrame)
            _G.frame_stack_highlight = highlightFrame and highlightFrame:GetDebugName() or nil
        end)
    "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let child_id = state.widgets.get_id_by_name("FrameStackChild").unwrap();
        state.hovered_frame = Some(child_id);
    }

    let returned_name: String = env
        .eval(
            r#"
            local highlight = GameTooltip:SetFrameStack(false, false, 0)
            return highlight and highlight:GetDebugName() or ""
        "#,
        )
        .unwrap();
    let script_name: String = env.eval("return _G.frame_stack_highlight or ''").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert_eq!(returned_name, "FrameStackChild");
    assert_eq!(script_name, "FrameStackChild");
    assert!(
        num_lines >= 2,
        "SetFrameStack should populate tooltip lines for the highlighted frame stack"
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines[0].left_text, "FrameStackChild");
}

#[test]
fn test_isobjecttype_frame_returns_true_for_gametooltip() {
    let env = WowLuaEnv::new().unwrap();

    let is_frame: bool = env
        .eval("return GameTooltip:IsObjectType('Frame')")
        .unwrap();
    assert!(
        is_frame,
        "GameTooltip:IsObjectType('Frame') should return true"
    );

    let is_region: bool = env
        .eval("return GameTooltip:IsObjectType('Region')")
        .unwrap();
    assert!(
        is_region,
        "GameTooltip:IsObjectType('Region') should return true"
    );

    let is_tooltip: bool = env
        .eval("return GameTooltip:IsObjectType('GameTooltip')")
        .unwrap();
    assert!(
        is_tooltip,
        "GameTooltip:IsObjectType('GameTooltip') should return true"
    );

    let is_button: bool = env
        .eval("return GameTooltip:IsObjectType('Button')")
        .unwrap();
    assert!(
        !is_button,
        "GameTooltip:IsObjectType('Button') should return false"
    );
}

#[test]
fn test_isobjecttype_for_other_types() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TypeTestButton", UIParent)
        local cb = CreateFrame("CheckButton", "TypeTestCheckButton", UIParent)
        local frame = CreateFrame("Frame", "TypeTestFrame", UIParent)
    "#,
    )
    .unwrap();

    // Button is a Frame
    let btn_is_frame: bool = env
        .eval("return TypeTestButton:IsObjectType('Frame')")
        .unwrap();
    assert!(btn_is_frame);

    // CheckButton is a Button
    let cb_is_button: bool = env
        .eval("return TypeTestCheckButton:IsObjectType('Button')")
        .unwrap();
    assert!(cb_is_button);

    // CheckButton is a Frame
    let cb_is_frame: bool = env
        .eval("return TypeTestCheckButton:IsObjectType('Frame')")
        .unwrap();
    assert!(cb_is_frame);

    // Frame is NOT a Button
    let frame_is_button: bool = env
        .eval("return TypeTestFrame:IsObjectType('Button')")
        .unwrap();
    assert!(!frame_is_button);
}

#[test]
fn test_setminimumwidth_and_getminimumwidth() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetMinimumWidth(150)").unwrap();

    let width: f32 = env.eval("return GameTooltip:GetMinimumWidth()").unwrap();
    assert_eq!(width, 150.0);
}

#[test]
fn test_setpadding_and_getpadding() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetPadding(8)").unwrap();

    let padding: f32 = env.eval("return GameTooltip:GetPadding()").unwrap();
    assert_eq!(padding, 8.0);
}

#[test]
fn test_fadeout_hides_and_clears_owner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "FadeOutOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:FadeOut()
    "#,
    )
    .unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!visible, "FadeOut should hide the tooltip");

    let has_owner: bool = env.eval("return GameTooltip:GetOwner() ~= nil").unwrap();
    assert!(!has_owner, "FadeOut should clear the owner");
}

#[test]
fn test_appendtext() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Hello")
        GameTooltip:AppendText(" World")
    "#,
    )
    .unwrap();

    // Verify through tooltip data
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines.len(), 1);
    assert_eq!(td.lines[0].left_text, "Hello World");
}

#[test]
fn tooltip_content_lifecycle_setowner_hides_clears_and_retains_new_owner() {
    let env = WowLuaEnv::new().unwrap();
    let (shown, lines, owned): (bool, i32, bool) = env
        .eval(
            r#"
            local first = CreateFrame("Frame", nil, UIParent)
            local second = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(first, "ANCHOR_RIGHT")
            GameTooltip:SetSpellByID(19750)
            assert(GameTooltip:IsShown())
            GameTooltip:SetOwner(second, "ANCHOR_LEFT")
            return GameTooltip:IsShown(), GameTooltip:NumLines(),
                GameTooltip:GetOwner() == second and GameTooltip:IsOwned(second)
            "#,
        )
        .unwrap();
    assert!(!shown);
    assert_eq!(lines, 0);
    assert!(owned);
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name("GameTooltip").unwrap();
    assert_eq!(sim.widgets.get(id).unwrap().tooltip_owner_id, sim.tooltips.get(&id).unwrap().owner_id);
    assert!(sim.tooltips.get(&id).unwrap().owner_id.is_some());
}

#[test]
fn tooltip_content_lifecycle_appends_remain_hidden_until_show() {
    let env = WowLuaEnv::new().unwrap();
    let (hidden, count, shown): (bool, i32, bool) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:AddLine("First")
            GameTooltip:AddDoubleLine("Second", "Right")
            local hidden, count = not GameTooltip:IsShown(), GameTooltip:NumLines()
            GameTooltip:Show()
            return hidden, count, GameTooltip:IsShown()
            "#,
        )
        .unwrap();
    assert!(hidden);
    assert_eq!(count, 2);
    assert!(shown);
}

#[test]
fn tooltip_content_lifecycle_settext_shows_owned_populated_tooltip() {
    let env = WowLuaEnv::new().unwrap();
    let (shown, owned, count): (bool, bool, i32) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:SetText("New content")
            return GameTooltip:IsShown(), GameTooltip:IsOwned(owner), GameTooltip:NumLines()
            "#,
        )
        .unwrap();
    assert!(shown);
    assert!(owned);
    assert_eq!(count, 1);
}

#[test]
fn tooltip_content_lifecycle_spell_payload_still_shows_after_setowner() {
    let env = WowLuaEnv::new().unwrap();
    let (shown, owned, count): (bool, bool, i32) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            GameTooltip:SetSpellByID(19750)
            return GameTooltip:IsShown(), GameTooltip:IsOwned(owner), GameTooltip:NumLines()
            "#,
        )
        .unwrap();
    assert!(shown);
    assert!(owned);
    assert!(count > 0);
}

#[test]
fn tooltip_content_lifecycle_explicit_hide_releases_owner_when_already_hidden() {
    for method in ["Hide()", "SetShown(false)", "FadeOut()"] {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            local owner = CreateFrame("Frame", "HiddenTooltipOwner", UIParent)
            GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
            "#,
        )
        .unwrap();
        assert!(!env.eval::<bool>("return GameTooltip:IsShown()").unwrap());
        env.exec(&format!("GameTooltip:{method}")).unwrap();
        let (shown, owned, owner_nil): (bool, bool, bool) = env
            .eval("return GameTooltip:IsShown(), GameTooltip:IsOwned(HiddenTooltipOwner), GameTooltip:GetOwner() == nil")
            .unwrap();
        assert!(!shown, "{method}");
        assert!(!owned, "{method}");
        assert!(owner_nil, "{method}");
        let sim = env.state().borrow();
        let id = sim.widgets.get_id_by_name("GameTooltip").unwrap();
        assert_eq!(sim.tooltips.get(&id).unwrap().owner_id, None, "{method}");
    }
}

#[test]
fn test_repeated_identical_tooltip_refresh_keeps_cached_strata_buckets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "TooltipRefreshOwner", UIParent)
        owner:SetSize(100, 50)
        owner:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:SetSpellByID(19750)
    "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        assert!(
            state.strata_buckets.is_some(),
            "initial tooltip show should populate cached strata buckets"
        );
    }

    env.exec(
        r#"
        GameTooltip:SetOwner(TooltipRefreshOwner, "ANCHOR_RIGHT")
        GameTooltip:SetSpellByID(19750)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    assert!(
        state.strata_buckets.is_some(),
        "repeating the same tooltip refresh should not invalidate cached strata buckets"
    );
}

#[test]
fn test_createframe_gametooltip_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local tt = CreateFrame("GameTooltip", "CustomTooltip", UIParent)
        tt:AddLine("Test")
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return CustomTooltip:GetObjectType()").unwrap();
    assert_eq!(obj_type, "GameTooltip");

    let count: i32 = env.eval("return CustomTooltip:NumLines()").unwrap();
    assert_eq!(count, 1);

    let strata: String = env.eval("return CustomTooltip:GetFrameStrata()").unwrap();
    assert_eq!(strata, "TOOLTIP");
}

#[test]
fn test_other_tooltip_frames_exist() {
    let env = WowLuaEnv::new().unwrap();

    let item_ref: bool = env.eval("return ItemRefTooltip ~= nil").unwrap();
    let shopping1: bool = env.eval("return ShoppingTooltip1 ~= nil").unwrap();
    let shopping2: bool = env.eval("return ShoppingTooltip2 ~= nil").unwrap();
    let friends: bool = env.eval("return FriendsTooltip ~= nil").unwrap();

    assert!(item_ref, "ItemRefTooltip should exist");
    assert!(shopping1, "ShoppingTooltip1 should exist");
    assert!(shopping2, "ShoppingTooltip2 should exist");
    assert!(friends, "FriendsTooltip should exist");

    // All should be GameTooltip type
    let item_type: String = env.eval("return ItemRefTooltip:GetObjectType()").unwrap();
    assert_eq!(item_type, "GameTooltip");
}

#[test]
fn test_copy_tooltip_copies_lines_and_spell_data_without_reowning() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "CopyTooltipOwner", UIParent)
        ShoppingTooltip1:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:SetSpellByID(19750)
        ShoppingTooltip1:CopyTooltip(GameTooltip)
    "#,
    )
    .unwrap();

    let copied_lines: i32 = env.eval("return ShoppingTooltip1:NumLines()").unwrap();
    assert!(
        copied_lines >= 3,
        "CopyTooltip should copy the source tooltip lines into ShoppingTooltip1"
    );

    let copied_spell_name: String = env
        .eval("local name = ShoppingTooltip1:GetSpell(); return name or ''")
        .unwrap();
    let copied_spell_id: i32 = env
        .eval("local _, id = ShoppingTooltip1:GetSpell(); return id or 0")
        .unwrap();
    let owner_name: String = env
        .eval("return ShoppingTooltip1:GetOwner():GetName()")
        .unwrap();

    assert_eq!(copied_spell_name, "Flash of Light");
    assert_eq!(copied_spell_id, 19750);
    assert_eq!(owner_name, "CopyTooltipOwner");

    let state = env.state().borrow();
    let shopping_id = state.widgets.get_id_by_name("ShoppingTooltip1").unwrap();
    let td = state.tooltips.get(&shopping_id).unwrap();
    assert_eq!(td.lines[0].left_text, "Flash of Light");
    assert_eq!(td.spell_id, Some(19750));
}
