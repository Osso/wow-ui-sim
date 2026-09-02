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
fn social_panel_uses_current_provider_without_legacy_wow_friend_rows() {
    common::with_perf_lock(|| {
        common::with_timeout(120, || {
            let env = full_game_env_after_startup();

            let (
                legacy_friend_system_enabled,
                social_panel_shown,
                provider_row_count,
                has_legacy_wow_row,
                online_names,
                offline_name,
                offline_connected,
            ): (bool, bool, i64, bool, String, String, bool) = env
                .eval(
                    r#"
                    ToggleSocialPanel()
                    FriendsList_Update(true)

                    local providerRowCount = 0
                    local hasLegacyWowRow = false
                    for _, elementData in FriendsListFrame.ScrollBox:EnumerateDataProviderEntireRange() do
                        providerRowCount = providerRowCount + 1
                        if elementData.buttonType == FRIENDS_BUTTON_TYPE_WOW then
                            hasLegacyWowRow = true
                        end
                    end

                    local onlineNames = {}
                    local offlineName = ""
                    local offlineConnected = true
                    for index = 1, C_FriendList.GetNumFriends() do
                        local info = C_FriendList.GetFriendInfoByIndex(index)
                        if info.connected then
                            table.insert(onlineNames, info.name)
                        else
                            offlineName = info.name
                            offlineConnected = info.connected
                        end
                    end
                    table.sort(onlineNames)

                    return C_FriendList.IsLegacyFriendSystemEnabled(),
                        FriendsListFrame:IsShown(),
                        providerRowCount,
                        hasLegacyWowRow,
                        table.concat(onlineNames, "\n"),
                        offlineName,
                        offlineConnected
                    "#,
                )
                .unwrap();

            assert!(
                !legacy_friend_system_enabled,
                "retail fixture must retain unsupported legacy friend-system semantics"
            );
            assert!(
                social_panel_shown,
                "ToggleSocialPanel should show the social panel"
            );
            assert!(
                provider_row_count > 0,
                "current retail FriendsList provider should populate when the panel opens"
            );
            assert!(
                !has_legacy_wow_row,
                "current retail provider must not add legacy WoW friend rows"
            );
            assert_eq!(online_names, "Arthax\nSylvara");
            assert_eq!(offline_name, "Durotan");
            assert!(!offline_connected, "Durotan should remain offline");
        });
    });
}
