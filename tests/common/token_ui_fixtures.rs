use wow_ui_sim::lua_api::WowLuaEnv;

// Mainline token ownership is attached to ContainerFrameSettingsManager. The
// Classic consumer keeps the legacy BackpackTokenFrame without that manager.
pub(crate) fn load_token_ui(env: &WowLuaEnv) {
    env.exec(
        r#"
        local loaded, reason = C_AddOns.LoadAddOn("Blizzard_TokenUI")
        assert(loaded, "LoadAddOn(Blizzard_TokenUI) failed: " .. tostring(reason))
        assert(BackpackTokenFrame, "BackpackTokenFrame should exist after loading Blizzard_TokenUI")
        if ContainerFrameSettingsManager then
            if not ContainerFrameSettingsManager.TokenTracker then
                ContainerFrameSettingsManager:OnAddonLoaded("Blizzard_TokenUI")
            end
            assert(
                ContainerFrameSettingsManager.TokenTracker == BackpackTokenFrame,
                "ContainerFrameSettingsManager should own BackpackTokenFrame after loading Blizzard_TokenUI"
            )
        end
        "#,
    )
    .expect("Failed to runtime-load Blizzard_TokenUI");
}
