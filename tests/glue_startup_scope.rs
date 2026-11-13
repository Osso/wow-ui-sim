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

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
#[test]
fn retail_glue_parent_leaves_panel_attributes_with_the_original_layout_owner() {
    test_timeout! {
        use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};

        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(ScreenKind::Login);
        let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
        for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Login) {
            load_addon(&env.loader_env(), &toc)
                .unwrap_or_else(|error| panic!("load {name}: {error}"));
        }
        env.exec(r#"
            assert(C_AddOns.IsAddOnLoaded("Blizzard_GlueParent"))
            assert(UIParent == GlueParent, "the original GlueParent alias must be in effect")
            assert(C_AddOns.IsAddOnLoaded("Blizzard_UIParentPanelManager"))
            assert(GetUIPanelLayoutFrame() == UIPanelLayoutFrame)
            assert(UIPanelLayoutFrame ~= UIParent)
            local expected = {
                DEFAULT_FRAME_WIDTH = 384, TOP_OFFSET = -116, LEFT_OFFSET = 16,
                CENTER_OFFSET = 384, RIGHT_OFFSET = 768, RIGHT_OFFSET_BUFFER = 80,
                PANEl_SPACING_X = 32,
            }
            for name, value in pairs(expected) do
                assert(GetUIPanelLayoutAttribute(name) == value, name .. " must come from original layout")
                assert(UIParent:GetAttribute(name) == nil, name .. " must not be copied to GlueParent")
            end
        "#).expect("the original panel getter should read from the original layout owner");
    }
}
