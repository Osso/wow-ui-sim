//! Inferred macro-label semantics; native qualifying action types remain unprobed.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn action_text_reads_macro_names_and_returns_plain_boolean() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        local id = CreateMacro("DrinkBot Water", "icon", "/use water")
        PickupMacro(id); PlaceAction(201)
        assert(GetActionInfo(201) == "macro")
        assert(C_ActionBar.UsesActionText(201) == true, "placed macro must use text")
        assert(type(C_ActionBar.UsesActionText(201)) == "boolean")
        assert(not issecretvalue(C_ActionBar.UsesActionText(201)))
        assert(select("#", C_ActionBar.UsesActionText(201)) == 1)
        assert(C_ActionBar.GetActionText(201) == "DrinkBot Water", "text must be exact stored name")
        assert(select("#", C_ActionBar.GetActionText(201)) == 1)
        EditMacro(id, "  Renamed Water  ", nil, "#showtooltip water")
        assert(C_ActionBar.UsesActionText(201) == true)
        assert(C_ActionBar.GetActionText(201) == "  Renamed Water  ", "rename must be visible without reassignment")
        EditMacro(id, nil, nil, "")
        assert(C_ActionBar.UsesActionText(201) == true, "body does not select label usage")
        assert(C_ActionBar.GetActionText(201) == "  Renamed Water  ")
        "##,
    )
    .unwrap();
}

#[test]
fn action_text_tracks_cursor_move_slot_transfer_and_spell_replacement() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local id = CreateMacro("MoveLabel", "icon", "/say hello")
        PickupMacro(id); PlaceAction(201)
        PickupAction(201); PlaceAction(202)
        assert(C_ActionBar.UsesActionText(201) == false)
        assert(C_ActionBar.GetActionText(201) == nil)
        assert(C_ActionBar.UsesActionText(202) == true, "cursor move must preserve label usage")
        assert(C_ActionBar.GetActionText(202) == "MoveLabel")
        assert(C_ActionBar.PutActionInSlot(202, 203))
        assert(C_ActionBar.UsesActionText(202) == false)
        assert(C_ActionBar.GetActionText(202) == nil)
        assert(C_ActionBar.UsesActionText(203) == true)
        assert(C_ActionBar.GetActionText(203) == "MoveLabel")
        PickupSpell(853); PlaceAction(203)
        assert(GetActionInfo(203) == "spell")
        assert(C_ActionBar.UsesActionText(203) == false)
        assert(C_ActionBar.GetActionText(203) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn action_text_deletion_and_slot_reuse_do_not_restore_labels() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local id = CreateMacro("DeleteLabel", "icon", "/say hello")
        PickupMacro(id); PlaceAction(201)
        PickupMacro(id); PlaceAction(202)
        assert(C_ActionBar.UsesActionText(201) == true, "macro must qualify before deletion")
        DeleteMacro("DeleteLabel")
        for _, slot in ipairs({201, 202}) do
            assert(C_ActionBar.UsesActionText(slot) == false)
            assert(C_ActionBar.GetActionText(slot) == nil)
        end
        assert(CreateMacro("Replacement", "icon", "/say bye") == id)
        assert(C_ActionBar.UsesActionText(201) == false)
        assert(C_ActionBar.GetActionText(201) == nil)
        PickupMacro(id); PlaceAction(201)
        assert(C_ActionBar.GetActionText(201) == "Replacement")
        A_Admin.ClearActionSlot(201)
        assert(C_ActionBar.UsesActionText(201) == false)
        assert(C_ActionBar.GetActionText(201) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn action_text_requires_a_populated_macro_name() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local id = CreateMacro("Occupied", "icon", "/say hello")
        PickupMacro(id); PlaceAction(201)
        assert(C_ActionBar.GetActionText(201) == "Occupied", "occupied name must be returned")
        EditMacro(id, "", nil, nil)
        assert(C_ActionBar.UsesActionText(201) == false)
        assert(C_ActionBar.GetActionText(201) == nil)
        EditMacro(id, "Restored", nil, nil)
        assert(C_ActionBar.UsesActionText(201) == true)
        assert(C_ActionBar.GetActionText(201) == "Restored")
        "#,
    )
    .unwrap();
}

#[test]
fn action_text_empty_spell_and_unplaceable_item_controls() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        assert(C_ActionBar.UsesActionText(201) == false)
        assert(type(C_ActionBar.UsesActionText(201)) == "boolean")
        assert(not issecretvalue(C_ActionBar.UsesActionText(201)))
        assert(C_ActionBar.GetActionText(201) == nil)
        assert(select("#", C_ActionBar.GetActionText(201)) == 1)
        PickupSpell(853); PlaceAction(201)
        assert(GetActionInfo(201) == "spell")
        assert(C_ActionBar.UsesActionText(201) == false)
        assert(C_ActionBar.GetActionText(201) == nil)
        -- The current model rejects item action placement: this is not a real item-slot fixture.
        PickupMerchantItem(1)
        assert(GetCursorInfo() == "item")
        PlaceAction(202)
        assert(GetActionInfo(202) == nil)
        assert(C_ActionBar.UsesActionText(202) == false)
        assert(C_ActionBar.GetActionText(202) == nil)
        "##,
    )
    .unwrap();
}
