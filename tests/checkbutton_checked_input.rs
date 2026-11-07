use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn checked_input_preserves_booleans_nil_and_strings_and_reads_numeric_flags() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local button = CreateFrame('CheckButton')
        button:SetChecked(true)
        assert(button:GetChecked())
        button:SetChecked(false)
        assert(not button:GetChecked())
        button:SetChecked(true)
        button:SetChecked(nil)
        assert(not button:GetChecked())
        button:SetChecked(true)
        button:SetChecked()
        assert(not button:GetChecked())
        for _, value in ipairs({'true', 'false', '1', '0', ''}) do
            button:SetChecked(true)
            button:SetChecked(value)
            assert(not button:GetChecked(), 'existing string behavior stays unchecked')
        end
        button:SetChecked(true)
        button:SetChecked(0)
        assert(not button:GetChecked(), 'numeric zero must clear checked')
        for _, value in ipairs({1, -2, 0.5}) do
            button:SetChecked(false)
            button:SetChecked(value)
            assert(button:GetChecked(), 'nonzero numeric flag must check')
        end
        button:SetChecked(0)
        assert(not button:GetChecked(), 'checked-to-zero transition must clear')
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wrath")]
#[test]
fn wrath_original_update_state_consumes_numeric_checked_and_unchecked_branches() {
    let env = WowLuaEnv::new().unwrap();
    let path = crate::common::blizzard_ui_cache_profile_dir().join("FrameXML/ActionButton.lua");
    let source = std::fs::read_to_string(&path).expect("original Wrath ActionButton consumer");
    env.exec(&source).unwrap();
    env.exec(
        r#"
        A_Admin.SetActionSlot(201, 853)
        A_Admin.StopCasting()
        local button = CreateFrame('CheckButton')
        button.action = 201
        A_Admin.SetActionRepeatActive(201, true)
        ActionButton_UpdateState(button)
        assert(button:GetChecked(), 'original SetChecked(1) branch must check')
        A_Admin.SetActionRepeatActive(201, false)
        ActionButton_UpdateState(button)
        assert(not button:GetChecked(), 'original SetChecked(0) branch must uncheck')
        "#,
    )
    .unwrap();
}
