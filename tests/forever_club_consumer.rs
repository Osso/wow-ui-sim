#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_game_addons(env: &WowLuaEnv) {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Forever Blizzard UI cache should be available");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let addons = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc) in addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("[load {name}] {error}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(env, ScreenKind::Game);
}

#[test]
fn unit_popup_title_friend_request_records_guild_member() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    load_game_addons(&env);
    let member_name: String = env
        .eval("return C_Club.GetMemberInfo('guild-0', C_Club.GetClubMembers('guild-0')[2]).name")
        .unwrap();
    env.exec(
        r#"
        local memberIDs = C_Club.GetClubMembers('guild-0')
        BNGetInfo = function() return nil, 'Fixture#1234' end
        UnitPopupAddTitleFriendButtonMixin:OnClick({
            clubInfo = { clubId = 'guild-0' },
            clubMemberInfo = { memberId = memberIDs[2] },
        })
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().title_friend_requests,
        vec![member_name]
    );
}
