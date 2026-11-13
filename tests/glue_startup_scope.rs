//! Preserve legacy startup fixtures outside the Retail/PTR/Forever login model.
//! Bare-env sentinels observe simulator setup, not native addon behavior.

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn modeled_login_profile() -> bool {
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

#[test]
fn login_chat_fixture_skip_does_not_change_legacy_startup() {
    for screen in [ScreenKind::Login, ScreenKind::CharacterSelect] {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(screen);
        env.exec(
            r#"
            ChatFrame1 = CreateFrame("Frame", "ChatFrame1", UIParent)
            function ChatFrame1:_AddMessageSilent() end
            _FakeChat = nil
            "#,
        )
        .unwrap();
        env.apply_post_event_workarounds();
        assert_eq!(
            env.eval::<bool>("return _FakeChat ~= nil and #_FakeChat.msgs.general > 0")
                .unwrap(),
            screen == ScreenKind::Login && !modeled_login_profile(),
            "post-event chat fixture scope on {ACTIVE:?}/{screen:?}"
        );
    }
}

#[test]
fn login_character_refresh_skip_does_not_change_legacy_startup() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::Login);
    env.exec(
        r#"
        CharacterRefreshCount = 0
        CharacterSelectListUtil = {
            GetCharacterListUpdate = function()
                CharacterRefreshCount = CharacterRefreshCount + 1
            end,
        }
        "#,
    )
    .unwrap();
    wow_ui_sim::startup::settle_headless_startup(&env);
    assert_eq!(
        env.eval::<i64>("return CharacterRefreshCount").unwrap(),
        if modeled_login_profile() { 0 } else { 1 },
        "Login compatibility refresh scope on {ACTIVE:?}"
    );
}
