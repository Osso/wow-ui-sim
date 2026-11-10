use wow_ui_sim::{client_profile::ClientProfile, lua_api::WowLuaEnv};

#[test]
fn issue52_gamepad_button_scripts_register_on_evidenced_frame_and_button_types() {
    if wow_ui_sim::client_profile::ACTIVE == ClientProfile::Wrath {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, kind in ipairs({'Frame','Button'}) do
            local frame = CreateFrame(kind)
            for _, script in ipairs({'OnGamePadButtonDown','OnGamePadButtonUp'}) do
                assert(frame:HasScript(script), kind .. ':' .. script)
                local handler = function() error('registration must not dispatch') end
                frame:SetScript(script, handler)
                assert(frame:GetScript(script) == handler)
                frame:SetScript(script, nil)
                assert(frame:GetScript(script) == nil)
            end
        end
        local frame = CreateFrame('Frame')
        for _, object in ipairs({CreateFrame('CheckButton'), frame:CreateTexture(), frame:CreateFontString(), frame:CreateLine(), frame:CreateAnimationGroup()}) do
            assert(not object:HasScript('OnGamePadButtonDown'))
            assert(not object:HasScript('OnGamePadButtonUp'))
        end
    "#).unwrap();
}
