use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn load_ui(active_channel_id: Option<i32>) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    {
        let mut state = env.state().borrow_mut();
        state.voice_chat.active_channel_id = active_channel_id;
        state.addon_base_paths = vec![ui.clone()];
    }
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

#[test]
fn channel_button_initialization_keeps_one_icon_for_supplied_voice_states() {
    for (channel, expected) in [
        (None, "chatframe-button-icon-voicechat"),
        (Some(1), "chatframe-button-icon-headset"),
    ] {
        let env = load_ui(channel);
        let (count, atlas): (i32, String) = env
            .eval(
                r#"
            local button = ChatFrameChannelButton
            assert(button and button.Icon, "channel button and icon must exist")
            local count = 0
            for _, region in ipairs({button:GetRegions()}) do
                local atlas = region:GetObjectType() == "Texture" and region:GetAtlas()
                if atlas and atlas:match("^chatframe%-button%-icon%-") then
                    count = count + 1
                end
            end
            return count, button.Icon:GetAtlas()
        "#,
            )
            .expect("inspect channel button");
        assert_eq!(count, 1, "channel {channel:?} must have one state icon");
        assert_eq!(
            atlas, expected,
            "channel {channel:?} must select its state icon"
        );
    }
}
