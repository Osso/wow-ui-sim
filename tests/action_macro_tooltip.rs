//! Simulator assumptions for macro directives, not native macro evaluation.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn explicit_macro_icon_is_returned_by_both_action_texture_queries_after_edit() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local first = "Interface\\Icons\\INV_Misc_QuestionMark"
        local second = "Interface\\Icons\\INV_Misc_Star_01"
        local id = CreateMacro("IconProbe", first, "/say x")
        A_Admin.SetMacroActionSlot(201, id)
        local kind, actual = GetActionInfo(201)
        assert(kind == "macro" and actual == id)
        assert(HasAction(201) and C_ActionBar.HasAction(201))
        assert(GetActionTexture(201) == first, "legacy macro texture must match stored icon")
        assert(C_ActionBar.GetActionTexture(201) == first, "C_ActionBar macro texture must match stored icon")

        EditMacro(id, nil, second, nil)
        assert(GetActionTexture(201) == second, "legacy macro texture must reflect edit")
        assert(C_ActionBar.GetActionTexture(201) == second, "C_ActionBar macro texture must reflect edit")
    "#).unwrap();
}

#[test]
fn macro_icon_queries_follow_move_clear_delete_and_empty_icon() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local icon = "Interface\\Icons\\INV_Misc_QuestionMark"
        local id = CreateMacro("IconProbe", icon, "/say x")
        A_Admin.SetMacroActionSlot(201, id)
        PickupAction(201)
        assert(not HasAction(201) and not C_ActionBar.HasAction(201))
        assert(GetActionTexture(201) == nil and C_ActionBar.GetActionTexture(201) == nil)
        PlaceAction(202)
        assert(HasAction(202) and C_ActionBar.HasAction(202))
        assert(GetActionInfo(202) == "macro")
        assert(GetActionTexture(202) == icon and C_ActionBar.GetActionTexture(202) == icon)

        A_Admin.ClearActionSlot(202)
        assert(not HasAction(202) and not C_ActionBar.HasAction(202))
        assert(GetActionTexture(202) == nil and C_ActionBar.GetActionTexture(202) == nil)

        A_Admin.SetMacroActionSlot(201, id)
        DeleteMacro(id)
        assert(not HasAction(201) and not C_ActionBar.HasAction(201))
        assert(GetActionTexture(201) == nil and C_ActionBar.GetActionTexture(201) == nil)

        local empty = CreateMacro("EmptyIconProbe", "", "/say x")
        A_Admin.SetMacroActionSlot(201, empty)
        assert(HasAction(201) and C_ActionBar.HasAction(201))
        assert(GetActionInfo(201) == "macro")
        assert(GetActionTexture(201) == nil and C_ActionBar.GetActionTexture(201) == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn spell_action_texture_queries_remain_nonempty_and_equal() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetActionSlot(201, 853)
        assert(HasAction(201) and C_ActionBar.HasAction(201))
        assert(GetActionInfo(201) == "spell")
        local texture = GetActionTexture(201)
        assert(type(texture) == "string" and texture ~= "")
        assert(C_ActionBar.GetActionTexture(201) == texture)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn action_macro_tooltip_tracks_edits_and_directive_boundaries() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r##"
        local id = CreateMacro("Probe", "icon", "/say hello")
        A_Admin.SetMacroActionSlot(201, id)
        local kind, actual = GetActionInfo(201)
        assert(kind == "macro" and actual == id)
        assert(HasAction(201) and C_ActionBar.HasAction(201))
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false)
        for _, body in ipairs({"#showtooltip", "  #showtooltip spell", "/say x\r\n\t#showtooltip\t[help] spell"}) do
            EditMacro(id, nil, nil, body)
            assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == true, body)
        end
        for _, body in ipairs({"", "#show", "#showtooltipExtra", "#showtooltip:foo", "#SHOWTOOLTIP", "/say #showtooltip", "#showtooltip[help]"}) do
            EditMacro("Probe", nil, nil, body)
            assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false, body)
        end
        EditMacro(id, nil, nil, "#showtooltip")
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201))
        PickupMacro(id)
        RunMacro(id)
        DeleteMacro(id)
        assert(not HasAction(201))
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false)
        local replacement = CreateMacro("Replacement", "icon", "/say no directive")
        assert(replacement == id)
        assert(not HasAction(201))
        A_Admin.SetMacroActionSlot(201, replacement)
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false)
    "##).unwrap();
    assert!(env.state().borrow().cursor_item.is_none());
    assert!(env.state().borrow().running_macro.is_none());
}

#[cfg(feature = "client-ptr")]
#[test]
fn action_macro_tooltip_slot_mutations_clear_old_associations() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_outfits.insert(203, 17);
    env.exec(
        r##"
        local id = CreateMacro("Move", "icon", "#showtooltip")
        assert(C_ActionBar.IsMacroActionWithShowTooltip(203) == false)
        A_Admin.SetMacroActionSlot(201, id)
        assert(C_ActionBar.PutActionInSlot(203, 201))
        assert(GetActionInfo(201) == "outfit")
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false)
        A_Admin.SetMacroActionSlot(201, id)
        assert(C_ActionBar.PutActionInSlot(201, 203))
        assert(not HasAction(201))
        assert(GetActionInfo(203) == "macro")
        assert(C_ActionBar.IsMacroActionWithShowTooltip(203))
        A_Admin.SetActionSlot(203, 123)
        assert(GetActionInfo(203) == "spell")
        assert(C_ActionBar.IsMacroActionWithShowTooltip(203) == false)
        PickupMacro(id); PlaceAction(201)
        PickupAction(201, true); PlaceAction(202)
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201))
        assert(C_ActionBar.IsMacroActionWithShowTooltip(202))
        PickupAction(201); PlaceAction(203)
        assert(not HasAction(201))
        assert(C_ActionBar.IsMacroActionWithShowTooltip(203))
        PickupSpell(456); PlaceAction(202)
        assert(C_ActionBar.IsMacroActionWithShowTooltip(202) == false)
        A_Admin.ClearActionSlot(203)
        assert(C_ActionBar.IsMacroActionWithShowTooltip(203) == false)
        A_Admin.SetMacroActionSlot(201, id)
        A_Admin.ClearActionBars()
        assert(not HasAction(201))
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false)
        assert(not pcall(A_Admin.SetMacroActionSlot, 201, 9999))
        assert(not pcall(A_Admin.SetMacroActionSlot, 0, id))
        assert(not pcall(C_ActionBar.IsMacroActionWithShowTooltip, 0))
    "##,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn action_macro_tooltip_query_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_ActionBar.IsMacroActionWithShowTooltip == nil)")
        .unwrap();
    wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    env.exec("assert(C_ActionBar.IsMacroActionWithShowTooltip == nil)")
        .unwrap();
}
