use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn dropdown_cached_templates_preserve_original_mixin_behavior(env: &WowLuaEnv) {
    env.exec(r#"
        local selection = CreateFrame("DropdownButton", nil, UIParent, "WowStyle1DropdownTemplate")
        selection:SetSelectionText(function() return "selected" end)
        selection:UpdateToMenuSelections(nil, {})
        assert(selection.Text:GetText() == "selected")

        local filter = CreateFrame("DropdownButton", nil, UIParent, "WowStyle1FilterDropdownTemplate")
        filter:SetIsDefaultCallback(function() return false end)
        filter:ValidateResetState()
        assert(filter.ResetButton:IsShown())
        filter:SetIsDefaultCallback(function() return true end)
        filter:ValidateResetState()
        assert(not filter.ResetButton:IsShown())

        local arrow = CreateFrame("DropdownButton", nil, UIParent, "WowStyle1ArrowDropdownTemplate")
        arrow:OnButtonStateChanged()
        assert(arrow.Arrow:GetAtlas() == "common-dropdown-a-button")
    "#).expect("cached dropdown templates should preserve observable mixin behavior");
}
}
