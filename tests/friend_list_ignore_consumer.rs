#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_friends_frame() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("friend-list fixture environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env
}

#[test]
fn real_friends_frame_ignore_scrollbox_uses_supplied_names() {
    let env = load_friends_frame();
    env.exec(
        r#"
        A_Admin.SetSocialIgnoreName(1, "R88eh-IgnoreAlpha")
        A_Admin.SetSocialIgnoreName(2, "R88eh-IgnoreBeta")
        FriendsFrame:Show()
        FriendsFrame.IgnoreListWindow:Show()
        IgnoreList_Update()

        local scrollBox = FriendsFrame.IgnoreListWindow.ScrollBox
        local provider = scrollBox:GetDataProvider()
        assert(provider:GetSize() == 3, "ignore provider must contain its header and two rows")
        local function check(index, expected)
            local row = assert(scrollBox:FindFrameByPredicate(function(_, elementData)
                return elementData.squelchType ~= nil and elementData.index == index
            end), "original ignore row not materialized: " .. index)
            assert(row.name:GetText() == expected, "original ignore row must show supplied name")
        end
        check(1, "R88eh-IgnoreAlpha")
        check(2, "R88eh-IgnoreBeta")
        "#,
    )
    .expect("original FriendsFrame ignore ScrollBox should display supplied names");
}
