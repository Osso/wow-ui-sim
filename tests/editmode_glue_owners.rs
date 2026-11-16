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
fn load_startup_addons(env: &WowLuaEnv, ui_dir: &Path, screen: ScreenKind) -> Vec<String> {
    if screen == ScreenKind::Game {
        crate::common::load_profile_framexml(env);
    }
    let addons = discover_blizzard_startup_addons_for_screen(ui_dir, screen);
    let mut loaded = Vec::with_capacity(addons.len());
    for addon in addons {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
        loaded.push(addon.name);
    }
    env.apply_post_load_workarounds();
    env.sync_string_metatable_to_global_string();
    loaded
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn function_source(env: &WowLuaEnv, global_name: &str) -> String {
    let code = format!(
        "local f = rawget(_G, {global_name:?}); if type(f) ~= 'function' then return '<missing>' end; return debug.getinfo(f, 'S').source or '<unknown>'"
    );
    env.eval(&code).expect("read global function source")
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn addon_precedes(addons: &[String], owner: &str, caller: &str) -> bool {
    let owner_index = addons.iter().position(|name| name == owner);
    let caller_index = addons.iter().position(|name| name == caller);
    matches!((owner_index, caller_index), (Some(owner), Some(caller)) if owner < caller)
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

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn invoke_group_finder_consumers(env: &WowLuaEnv) -> String {
    env.eval(
        r#"
        local calls = {}
        local namespace = C_AddOns
        local original = namespace and namespace.LoadAddOn
        if type(original) == "function" then
            namespace.LoadAddOn = function(name, ...)
                calls[#calls + 1] = name
                return original(name, ...)
            end
        end
        PVEFrame_ToggleFrame()
        if type(LFDMicroButtonMixin) == "table" and type(LFDMicroButtonMixin.OnClick) == "function" then
            LFDMicroButtonMixin:OnClick()
        end
        return table.concat(calls, ",")
        "#,
    )
    .expect("run original Group Finder consumers and capture addon loads")
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn probe_group_finder_consumer(env: &WowLuaEnv, addons: &[String]) {
    let source_before = function_source(env, "PVEFrame_ToggleFrame");
    let loaded_before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GroupFinder')")
        .expect("check Group Finder load state");
    let owner_before = source_before.contains("Blizzard_GroupFinder/");
    let owner_order = addon_precedes(addons, "Blizzard_GroupFinder", "Blizzard_MicroMenu");
    eprintln!(
        "BOUNDARY|PVEFrame_ToggleFrame|source_before={source_before}|loaded_before={loaded_before}|owner_before={owner_before}|owner_before_micro_menu={owner_order}"
    );

    let load_attempts = invoke_group_finder_consumers(env);
    eprintln!(
        "BOUNDARY|PVEFrame_ToggleFrame|source_after={}|load_addon_calls={load_attempts}",
        function_source(env, "PVEFrame_ToggleFrame")
    );
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn probe_print_handler_consumer(env: &WowLuaEnv, addons: &[String]) {
    let setter_source = function_source(env, "setprinthandler");
    let caller_addon_loaded = addons.iter().any(|name| name == "Blizzard_GlueParent");
    let owner_order = addon_precedes(addons, "Blizzard_PrintHandler", "Blizzard_GlueParent");
    let handler_source: String = env
        .eval(
            "local f = type(getprinthandler) == 'function' and getprinthandler(); if type(f) ~= 'function' then return '<missing>' end; return debug.getinfo(f, 'S').source or '<unknown>'",
        )
        .expect("read the Glue print handler installed by its caller");
    let caller_registered = handler_source.contains("Blizzard_GlueParent/");
    eprintln!(
        "BOUNDARY|setprinthandler|setter_source={setter_source}|caller_addon_loaded={caller_addon_loaded}|owner_before_caller={owner_order}|caller_registered={caller_registered}|handler_source={handler_source}"
    );

    if setter_source.contains("Blizzard_PrintHandler/") {
        let stores_handler: bool = env
            .eval("local f = function() end; setprinthandler(f); return getprinthandler() == f")
            .expect("exercise the original print-handler owner");
        assert!(stores_handler, "the original print owner must store its handler");
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn probe_store_consumer(env: &WowLuaEnv, addons: &[String]) {
    let owner_source = function_source(env, "StoreFrame_WaitingForCharacterListUpdate");
    let consumer_source = env
        .eval::<String>(
            "local f = CharacterSelectFrameMixin and CharacterSelectFrameMixin.OnUpdate; if type(f) ~= 'function' then f = CharacterSelect_OnUpdate end; if type(f) ~= 'function' then return '<missing>' end; return debug.getinfo(f, 'S').source or '<unknown>'",
        )
        .expect("read the original character-select consumer source");
    let consumer_loaded = addons.iter().any(|name| name == "Blizzard_GlueXML");
    let owner_loaded = addons.iter().any(|name| name == "Blizzard_StoreUI");
    let owner_is_original = owner_source.contains("Blizzard_StoreUI/");
    eprintln!(
        "BOUNDARY|StoreFrame_WaitingForCharacterListUpdate|owner_source={owner_source}|owner_loaded={owner_loaded}|consumer_source={consumer_source}|consumer_loaded={consumer_loaded}"
    );

    if consumer_source.contains("CharacterSelect.lua") {
        let callback_called: bool = env
            .eval(
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
            .expect("run the original character-select OnUpdate consumer");
        assert_eq!(callback_called, owner_is_original);
    }
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
    let game_addons = load_startup_addons(&game_env, &ui_dir, ScreenKind::Game);
    probe_group_finder_consumer(&game_env, &game_addons);

    let glue_env = prepare_env(&ui_dir, ScreenKind::CharacterSelect);
    let glue_addons = load_startup_addons(&glue_env, &ui_dir, ScreenKind::CharacterSelect);
    probe_print_handler_consumer(&glue_env, &glue_addons);
    probe_store_consumer(&glue_env, &glue_addons);
}
