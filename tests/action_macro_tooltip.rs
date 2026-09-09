//! Simulator assumptions for macro directives, not native macro evaluation.
use wow_ui_sim::lua_api::WowLuaEnv;

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
        DeleteMacro(id)
        assert(not HasAction(201))
        assert(C_ActionBar.IsMacroActionWithShowTooltip(201) == false)
    "##).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn action_macro_tooltip_slot_mutations_clear_old_associations() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_outfits.insert(203, 17);
    env.exec(r##"
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
    "##).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn action_macro_tooltip_query_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_ActionBar.IsMacroActionWithShowTooltip == nil)")
        .unwrap();
}
