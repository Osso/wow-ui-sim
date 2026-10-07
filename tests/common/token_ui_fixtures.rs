use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) fn load_token_ui(env: &WowLuaEnv) {
    env.exec(
        r#"
        local loaded, reason = C_AddOns.LoadAddOn("Blizzard_TokenUI")
        assert(loaded, "LoadAddOn(Blizzard_TokenUI) failed: " .. tostring(reason))
        if ContainerFrameSettingsManager and not ContainerFrameSettingsManager.TokenTracker then
            ContainerFrameSettingsManager:OnAddonLoaded("Blizzard_TokenUI")
        end
        assert(BackpackTokenFrame, "BackpackTokenFrame should exist after loading Blizzard_TokenUI")
        assert(
            ContainerFrameSettingsManager and ContainerFrameSettingsManager.TokenTracker == BackpackTokenFrame,
            "ContainerFrameSettingsManager should own BackpackTokenFrame after loading Blizzard_TokenUI"
        )
        "#,
    )
    .expect("Failed to runtime-load Blizzard_TokenUI");
}
