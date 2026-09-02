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
                provider_diagnostic,
                legacy_friend_system_enabled,
                social_panel_shown,
                provider_row_count,
                has_legacy_wow_row,
                online_names,
                offline_name,
                offline_connected,
            ): (String, bool, bool, i64, bool, String, String, bool) = env
                .eval(
                    r#"
                    ToggleSocialPanel()
                    FriendsList_Update(true)

                    local scrollBox = FriendsListFrame and FriendsListFrame.ScrollBox
                    local providerExists = scrollBox ~= nil
                        and scrollBox.HasDataProvider ~= nil
                        and scrollBox:HasDataProvider()
                    local providerRowCount = 0
                    local hasLegacyWowRow = false
                    local elementSummaries = {}
                    if providerExists then
                        for _, elementData in scrollBox:EnumerateDataProviderEntireRange() do
                            providerRowCount = providerRowCount + 1
                            if elementData.buttonType == FRIENDS_BUTTON_TYPE_WOW then
                                hasLegacyWowRow = true
                            end

                            local keyTypes = {}
                            if type(elementData) == "table" then
                                for key, value in pairs(elementData) do
                                    table.insert(keyTypes, tostring(key) .. ":" .. type(value))
                                    if #keyTypes == 8 then
                                        break
                                    end
                                end
                            end
                            table.sort(keyTypes)
                            table.insert(
                                elementSummaries,
                                type(elementData) .. "{" .. table.concat(keyTypes, ",") .. "}"
                            )
                        end
                    end

                    local friendCount = C_FriendList.GetNumFriends()
                    local onlineFriendCount = C_FriendList.GetNumOnlineFriends()
                    local providerDiagnostic = string.format(
                        "legacy_enabled=%s provider_exists=%s provider_rows=%d elements=[%s] friend_count=%d online_friend_count=%d",
                        tostring(C_FriendList.IsLegacyFriendSystemEnabled()),
                        tostring(providerExists),
                        providerRowCount,
                        table.concat(elementSummaries, ";"),
                        friendCount,
                        onlineFriendCount
                    )

                    local onlineNames = {}
                    local offlineName = ""
                    local offlineConnected = true
                    for index = 1, friendCount do
                        local info = C_FriendList.GetFriendInfoByIndex(index)
                        if info.connected then
                            table.insert(onlineNames, info.name)
                        else
                            offlineName = info.name
                            offlineConnected = info.connected
                        end
                    end
                    table.sort(onlineNames)

                    return providerDiagnostic,
                        C_FriendList.IsLegacyFriendSystemEnabled(),
                        FriendsListFrame:IsShown(),
                        providerRowCount,
                        hasLegacyWowRow,
                        table.concat(onlineNames, "\n"),
                        offlineName,
                        offlineConnected
                    "#,
                )
                .unwrap();

            eprintln!("[friends-provider-diagnostic] {provider_diagnostic}");

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
