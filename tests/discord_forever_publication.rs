//! Forever 1.60.1 publishes the shape-matching `C_Discord` members declared by
//! its DiscordDocumentation.lua. The original ChatFrameEditBox `/discord`
//! handler reads `C_Discord.IsUserOAuthed` (ChatFrameEditBox.lua:176).
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn forever_game() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    let game = wow_ui_sim::screen::ScreenKind::Game;
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
    env
}

fn submit_discord_slash(env: &WowLuaEnv) -> (String, String) {
    env.exec(r#"ChatFrameUtil.OpenChat("/discord hello")"#)
        .expect("open chat with the slash text");
    env.fire_on_update(0.05).expect("edit box parses its text");
    env.eval(
        r#"
        local editBox = ChatFrameUtil.ChooseBoxForSend()
        return editBox:GetChatType(), editBox:GetText()
        "#,
    )
    .expect("read the edit box chat type")
}

#[test]
fn forever_discord_slash_follows_oauth_state() {
    crate::common::with_timeout(120, || {
        let env = forever_game();
        let (chat_type, text) = submit_discord_slash(&env);
        assert_ne!(chat_type, "GUILD_DISCORD", "unauthorized slash is cleared");
        assert_eq!(text, "");
        env.exec("C_Discord.Authorize()").expect("authorize");
        assert!(env.eval::<bool>("return C_Discord.IsUserOAuthed()").unwrap());
        let (chat_type, text) = submit_discord_slash(&env);
        assert_eq!(chat_type, "GUILD_DISCORD");
        assert_eq!(text, "hello");
    });
}
