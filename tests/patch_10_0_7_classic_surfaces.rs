//! Retail retirement must leave the existing classic lookup surface unchanged.
#![cfg(feature = "client-mists")]

#[test]
fn patch_10_0_7_mists_preserves_legacy_namespace_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_QuestOffer.GetHideRequiredItemsOnTurnIn) == 'function')
        local names = {
            'GetLastAchievement', 'GetLastItem', 'GetLastScreenshotIndex',
            'GetMaxTweetLength', 'GetScreenshotInfoByIndex', 'GetTweetLength',
            'IsSocialEnabled', 'RegisterSocialBrowser', 'SetTextureToScreenshot',
            'TwitterCheckStatus', 'TwitterConnect', 'TwitterDisconnect',
            'TwitterGetMSTillCanPost', 'TwitterPostAchievement', 'TwitterPostItem',
            'TwitterPostMessage', 'TwitterPostScreenshot',
        }
        for _, name in ipairs(names) do
            assert(type(C_Social[name]) == 'function', name)
        end
        "#,
    )
    .unwrap();
}
