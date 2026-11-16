#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
use std::path::Path;

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
use wow_ui_sim::screen::ScreenKind;

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn prepare_env(ui_dir: &Path, screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    env.state().borrow_mut().addon_base_paths = vec![ui_dir.to_path_buf()];
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn load_startup_addons(env: &WowLuaEnv, ui_dir: &Path, screen: ScreenKind) {
    if screen == ScreenKind::Game {
        crate::common::load_profile_framexml(env);
    }
    for addon in discover_blizzard_startup_addons_for_screen(ui_dir, screen) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.apply_post_load_workarounds();
    env.sync_string_metatable_to_global_string();
}

#[cfg(feature = "retail-12-1-0")]
fn clear_raid_warning_enums(env: &WowLuaEnv) {
    env.exec(
        "Enum.EditModeAccountSetting.ShowRaidWarning = nil; Enum.EditModeSystem.RaidWarning = nil",
    )
    .expect("clear Edit Mode enum fields");
    wow_ui_sim::ptr::compat_bootstrap::apply_post_load(env);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn edit_mode_raid_warning_defaults_select_original_consumers() {
    let ui_dir = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = prepare_env(&ui_dir, ScreenKind::Game);
    clear_raid_warning_enums(&env);
    load_startup_addons(&env, &ui_dir, ScreenKind::Game);
    clear_raid_warning_enums(&env);

    let preset_selected_exact_system: bool = env
        .eval("return EDIT_MODE_MODERN_SYSTEM_MAP[24] ~= nil")
        .expect("inspect the original preset-layout consumer");
    assert!(
        preset_selected_exact_system,
        "EditModePresetLayouts.lua must select RaidWarning at Enum.EditModeSystem.RaidWarning = 24"
    );

    let selected_account_setting: i32 = env
        .eval(
            r#"
            local selected
            EditModeManagerFrame = {
                OnAccountSettingChanged = function(_, setting) selected = setting end,
            }
            local settings = { RefreshRaidWarning = function() end }
            EditModeAccountSettingsMixin.SetRaidWarningShown(settings, true, true)
            return selected
            "#,
        )
        .expect("invoke the original Edit Mode account-setting consumer");
    assert_eq!(selected_account_setting, 33);
}

#[cfg(all(
    any(
        feature = "client-ptr",
        feature = "client-retail",
        feature = "client-mists"
    ),
    not(feature = "client-wowforever")
))]
fn group_finder_toggle_opens_and_closes(env: &WowLuaEnv) {
    let toggled: bool = env
        .eval(
            r#"
            if type(PVEFrame_ToggleFrame) ~= "function" then return false end
            C_LFGInfo = C_LFGInfo or {}
            C_LFGInfo.CanPlayerUseGroupFinder = function() return true end
            if PVEFrame then PVEFrame:Hide() end
            PVEFrame_ToggleFrame()
            local frame = PVEFrame
            if not frame or not frame:IsShown() then return false end
            PVEFrame_ToggleFrame()
            return not frame:IsShown()
            "#,
        )
        .expect("run the Group Finder toggle consumer");
    assert!(toggled, "the Group Finder consumer must open and close PVEFrame");
}

#[cfg(feature = "client-wowforever")]
fn group_finder_toggle_opens_and_closes(env: &WowLuaEnv) {
    let invoked_without_error: bool = env
        .eval(
            r#"
            local toggle = PVEFrame_ToggleFrame
            if type(toggle) ~= "function" then return false end
            local ok = pcall(toggle)
            return ok
            "#,
        )
        .expect("invoke the Forever Group Finder bridge");
    assert!(
        invoked_without_error,
        "the Forever Group Finder bridge must return without an error"
    );
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn print_owner_stores_handler(env: &WowLuaEnv) {
    let stored: bool = env
        .eval("local handler = function() end; setprinthandler(handler); return getprinthandler() == handler")
        .expect("exercise the print-handler owner");
    assert!(stored, "the print owner must return the handler that was stored");
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn invoke_character_select_consumer(env: &WowLuaEnv) -> bool {
    env.eval(
        r#"
        local called = false
        C_CharacterServices = C_CharacterServices or {}
        C_CharacterServices.HasQueuedUpgrade = function() return false end
        C_StoreGlue = C_StoreGlue or {}
        C_StoreGlue.GetVASProductReady = function() return true end
        CharacterServicesMaster_OnCharacterListUpdate = function() end
        StoreFrame_OnCharacterListUpdate = function() called = true end
        StaticPopup_CheckQueuedDialogs = function() end
        local mixin = CharacterSelectFrameMixin
        if mixin and type(mixin.OnUpdate) == 'function' then
            mixin:OnUpdate(0)
        elseif type(CharacterSelect_OnUpdate) == 'function' then
            CharacterSelect_OnUpdate({}, 0)
        end
        return called
        "#,
    )
    .expect("run the original character-select OnUpdate consumer")
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
#[test]
fn editmode_glue_owner_boundaries_preserve_original_consumers() {
    let ui_dir = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let game_env = prepare_env(&ui_dir, ScreenKind::Game);
    load_startup_addons(&game_env, &ui_dir, ScreenKind::Game);
    group_finder_toggle_opens_and_closes(&game_env);

    let glue_env = prepare_env(&ui_dir, ScreenKind::CharacterSelect);
    load_startup_addons(&glue_env, &ui_dir, ScreenKind::CharacterSelect);
    print_owner_stores_handler(&glue_env);
    assert!(
        invoke_character_select_consumer(&glue_env),
        "the CharacterSelect consumer must call the Store update callback"
    );
}
