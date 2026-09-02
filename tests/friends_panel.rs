use crate::common;

use std::path::{Path, PathBuf};

use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn full_game_env_after_startup() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1600.0, 1200.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    load_all_blizzard_addons(&env, &blizzard_ui_dir());
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(&env);
    fire_one_on_update_tick(&env);

    env
}

fn load_all_blizzard_addons(env: &WowLuaEnv, ui: &Path) {
    for (name, toc_path) in &discover_blizzard_addons(ui) {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[load {name}] FAILED: {err}");
        }
    }
}

#[test]
fn social_panel_toggle_populates_online_friend_provider_and_provides_offline_friend_data() {
    common::with_perf_lock(|| {
        common::with_timeout(120, || {
            let env = full_game_env_after_startup();

            let (
                provider_online_names,
                expected_online_names,
                has_offline_element,
                offline_name,
                offline_connected,
            ): (String, String, bool, String, bool) = env
                .eval(
                    r#"
                    ToggleSocialPanel()
                    FriendsList_Update(true)

                    local providerOnlineNames = {}
                    for elementData in FriendsListFrame.ScrollBox:EnumerateDataProviderEntireRange() do
                        if elementData.buttonType == FRIENDS_BUTTON_TYPE_WOW then
                            local info = C_FriendList.GetFriendInfoByIndex(elementData.id)
                            if info and info.connected then
                                table.insert(providerOnlineNames, info.name)
                            end
                        end
                    end
                    table.sort(providerOnlineNames)

                    local expectedOnlineNames = {}
                    for index = 1, C_FriendList.GetNumFriends() do
                        local info = C_FriendList.GetFriendInfoByIndex(index)
                        if info and info.connected then
                            table.insert(expectedOnlineNames, info.name)
                        end
                    end
                    table.sort(expectedOnlineNames)

                    local offlineElement = FriendsListFrame.ScrollBox:GetDataProvider():FindElementDataByPredicate(function(elementData)
                        if elementData.buttonType ~= FRIENDS_BUTTON_TYPE_WOW then
                            return false
                        end
                        local info = C_FriendList.GetFriendInfoByIndex(elementData.id)
                        return info and not info.connected
                    end)
                    local offlineInfo = offlineElement and C_FriendList.GetFriendInfoByIndex(offlineElement.id)

                    return table.concat(providerOnlineNames, "\n"),
                        table.concat(expectedOnlineNames, "\n"),
                        offlineElement ~= nil,
                        offlineInfo and offlineInfo.name or "",
                        offlineInfo and offlineInfo.connected or false
                    "#,
                )
                .unwrap();

            assert!(
                !expected_online_names.is_empty(),
                "state-backed friend fixture should include an online friend"
            );
            assert_eq!(
                provider_online_names, expected_online_names,
                "WoW friend data provider should match the state-backed online friends"
            );
            assert!(
                has_offline_element,
                "friends data provider should contain a state-backed offline WoW friend"
            );
            assert!(
                !offline_name.is_empty(),
                "offline data-provider row should resolve to a named friend"
            );
            assert!(
                !offline_connected,
                "offline data-provider row should resolve with connected=false"
            );

            let (wow_alpha, offline_alpha): (f64, f64) = env
                .eval(
                    r#"
                    return FRIENDS_WOW_BACKGROUND_COLOR.a, FRIENDS_OFFLINE_BACKGROUND_COLOR.a
                    "#,
                )
                .unwrap();

            assert!(
                (wow_alpha - 0.05).abs() < 0.001,
                "online friend row background should be translucent, got alpha {wow_alpha}"
            );
            assert!(
                (offline_alpha - 0.05).abs() < 0.001,
                "offline friend row background should be translucent, got alpha {offline_alpha}"
            );
        });
    });
}
