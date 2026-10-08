//! Bounded cached-runtime behavior for the identities explicitly named by 7.2.0.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_7_2_0_mask_and_vertex_state(env: &WowLuaEnv) {
    let observed: (String, i32, bool, f64, f64, f64, f64, i32) = env.eval(r#"
        local frame = CreateFrame('Frame')
        local texture = frame:CreateTexture()
        local mask = frame:CreateMaskTexture()
        texture:AddMaskTexture(mask)
        texture:AddMaskTexture(mask)
        local count = texture:GetNumMaskTextures()
        local same = texture:GetMaskTexture(1) == mask
        texture:SetVertexOffset(1, 3.5, -2)
        texture:SetVertexOffset(4, -7, 11)
        local x, y = texture:GetVertexOffset(1)
        local otherX, otherY = texture:GetVertexOffset(4)
        texture:RemoveMaskTexture(mask)
        return mask:GetObjectType(), count, same, x, y, otherX, otherY,
            texture:GetNumMaskTextures()
    "#).unwrap();
    assert_eq!(observed, ("MaskTexture".into(), 1, true, 3.5, -2.0, -7.0, 11.0, 0));
}
}

prefork_full_ui_case! {
fn patch_7_2_0_equipment_set_lifecycle(env: &WowLuaEnv) {
    let observed: (String, bool, bool, bool, bool) = env.eval(r#"
        local initialCount = C_EquipmentSet.GetNumEquipmentSets()
        C_EquipmentSet.CreateEquipmentSet('P720 Tank', '12345')
        local id = C_EquipmentSet.GetEquipmentSetID('P720 Tank')
        local created = C_EquipmentSet.GetNumEquipmentSets() == initialCount + 1
        C_EquipmentSet.ModifyEquipmentSet(id, 'P720 Protection', '67890')
        local renamed = C_EquipmentSet.GetEquipmentSetID('P720 Tank') == nil
            and C_EquipmentSet.GetEquipmentSetID('P720 Protection') == id
        C_EquipmentSet.AssignSpecToEquipmentSet(id, 2)
        local assigned = C_EquipmentSet.GetEquipmentSetAssignedSpec(id) == 2
            and C_EquipmentSet.GetEquipmentSetForSpec(2) == id
        local name = C_EquipmentSet.GetEquipmentSetInfo(id)
        C_EquipmentSet.DeleteEquipmentSet(id)
        local deleted = C_EquipmentSet.GetEquipmentSetID('P720 Protection') == nil
            and C_EquipmentSet.GetNumEquipmentSets() == initialCount
        return name, created, renamed, assigned, deleted
    "#).unwrap();
    assert_eq!(observed, ("P720 Protection".into(), true, true, true, true));
}
}

prefork_full_ui_case! {
fn patch_7_2_0_named_addons_load(env: &WowLuaEnv) {
    let loaded: bool = env.eval(r#"
        for _, name in ipairs({
            'Blizzard_APIDocumentation', 'Blizzard_Contribution', 'Blizzard_Deprecated'
        }) do
            C_AddOns.LoadAddOn(name)
            if not C_AddOns.IsAddOnLoaded(name) then return false end
        end
        return true
    "#).unwrap();
    assert!(loaded, "All three named current-retail addons must load");
}
}
