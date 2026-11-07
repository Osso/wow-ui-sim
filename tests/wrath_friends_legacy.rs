//! Original Wrath FriendsFrame consumes modeled legacy friends globals.
#![cfg(feature = "client-wrath")]

use crate::common;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

fn load_friends_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Wrath environment");
    env.set_screen_size(1600.0, 1200.0);
    let ui = common::panel_fixtures::blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    common::load_profile_framexml(&env);
    for (name, toc) in discover_blizzard_addons(&ui) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    process_pending_timers(&env);
    fire_one_on_update_tick(&env);
    env
}

#[test]
fn wrath_friends_legacy_original_panel_displays_seeded_rows_and_selection() {
    common::with_timeout(90, || {
        let env = load_friends_env();
        // Storage order differs from the online-first display projection.
        env.state().borrow_mut().social_friends.swap(0, 1);
        env.exec(
            r#"
            local errors = {}
            seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
            ToggleFriendsFrame(1)
            assert(#errors == 0, table.concat(errors, '\n'))
            FriendsList_Update()
            assert(FriendsFrame:IsShown() and FriendsListFrame:IsShown(), 'original panel opens')
            assert(GetNumFriends() == 3 and select(2, GetNumFriends()) == 2)
            assert(FriendsFrameOfflineHeader:IsShown(), 'native offline header')
            local buttons = FriendsFrameFriendsScrollFrame.buttons
            assert(buttons[1].name:GetText() == 'Arthax, '..format(FRIENDS_LEVEL_TEMPLATE, 70, 'Paladin'))
            assert(buttons[1].info:GetText() == 'Stormwind City')
            assert(buttons[2].name:GetText() == 'Sylvara, '..format(FRIENDS_LEVEL_TEMPLATE, 60, 'Mage'))
            assert(buttons[4].name:GetText() == 'Durotan')
            assert(GetSelectedFriend() == 1 and FriendsFrame.selectedFriend == 1)
            assert(FriendsFrameSendMessageButton:IsEnabled() == true or FriendsFrameSendMessageButton:IsEnabled() == 1)
            FriendsFrame_SelectFriend(FRIENDS_BUTTON_TYPE_WOW, 3)
            FriendsList_Update()
            assert(GetSelectedFriend() == 3 and FriendsFrame.selectedFriend == 3)
            assert(FriendsFrameSendMessageButton:IsEnabled() == false or FriendsFrameSendMessageButton:IsEnabled() == 0, 'offline selection disables tell')
            FriendsFrame_SelectFriend(FRIENDS_BUTTON_TYPE_WOW, 2)
            FriendsList_Update()
            assert(GetSelectedFriend() == 2 and FriendsFrame.selectedFriend == 2)
            assert(FriendsFrameSendMessageButton:IsEnabled() == true or FriendsFrameSendMessageButton:IsEnabled() == 1, 'online selection enables tell')
            "#,
        )
        .expect("original ToggleFriendsFrame/FriendsList_Update values");
    });
}
