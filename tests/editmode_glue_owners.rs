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
    let loaded = addons
        .iter()
        .map(|addon| addon.name.clone())
        .collect::<Vec<_>>();
    for addon in addons {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
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

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
#[derive(Clone, Copy, Debug)]
struct BoundaryCounters {
    pve_install: i32,
    pve_calls: i32,
    print_install: i32,
    print_calls: i32,
    store_install: i32,
    store_calls: i32,
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
struct GroupFinderObservation {
    source_before: String,
    loaded_before: bool,
    owner_before: bool,
    load_attempts: String,
    source_after: String,
    counters_at_startup: BoundaryCounters,
    counters_after_consumer: BoundaryCounters,
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
struct PrintHandlerObservation {
    setter_source: String,
    owner_order: bool,
    caller_registered: bool,
    counters_at_startup: BoundaryCounters,
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
struct StoreObservation {
    consumer_source: String,
    consumer_loaded: bool,
    owner_loaded: bool,
    owner_is_original: bool,
    counters_at_startup: BoundaryCounters,
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn read_boundary_counter(env: &WowLuaEnv, name: &str) -> i32 {
    let code = format!(
        "local registry = debug.getregistry(); local counters = rawget(registry, '__wow_editmode_glue_probe_counters'); if type(counters) ~= 'table' then return 0 end; return tonumber(rawget(counters, {name:?})) or 0"
    );
    env.eval(&code)
        .unwrap_or_else(|error| panic!("read boundary counter {name}: {error}"))
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn read_boundary_counters(env: &WowLuaEnv) -> BoundaryCounters {
    BoundaryCounters {
        pve_install: read_boundary_counter(env, "pve_install"),
        pve_calls: read_boundary_counter(env, "pve_calls"),
        print_install: read_boundary_counter(env, "print_install"),
        print_calls: read_boundary_counter(env, "print_calls"),
        store_install: read_boundary_counter(env, "store_install"),
        store_calls: read_boundary_counter(env, "store_calls"),
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn assert_counter_install_counts(counters: BoundaryCounters, baseline: bool) {
    let expected_pve_install = if baseline || cfg!(feature = "client-wowforever") {
        1
    } else {
        0
    };
    let expected_glue_install = if baseline { 1 } else { 0 };
    assert_eq!(counters.pve_install, expected_pve_install, "PVE bridge install count");
    assert_eq!(counters.print_install, expected_glue_install, "print fallback install count");
    assert_eq!(counters.store_install, expected_glue_install, "Store fallback install count");
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn log_boundary_counters(label: &str, phase: &str, counters: BoundaryCounters) {
    eprintln!(
        "COUNTER|{label}|phase={phase}|pve_install={}|pve_calls={}|print_install={}|print_calls={}|store_install={}|store_calls={}",
        counters.pve_install,
        counters.pve_calls,
        counters.print_install,
        counters.print_calls,
        counters.store_install,
        counters.store_calls,
    );
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn read_baseline_adapter_mode() -> bool {
    match std::env::var("WOW_SIM_EDITMODE_GLUE_ADAPTER").as_deref() {
        Ok("base") => true,
        Ok("candidate") => false,
        Ok(value) => panic!("unknown Edit Mode Glue adapter variant: {value}"),
        Err(error) => panic!("missing Edit Mode Glue adapter variant: {error}"),
    }
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
        C_LFGInfo.CanPlayerUseGroupFinder = function() return false end
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
fn log_group_finder_startup(
    source: &str,
    loaded: bool,
    owner_before: bool,
    owner_order: bool,
    counters: BoundaryCounters,
) {
    eprintln!(
        "OWNER|PVEFrame_ToggleFrame|source_before={source}|loaded_before={loaded}|owner_before={owner_before}|owner_before_micro_menu={owner_order}"
    );
    log_boundary_counters("PVEFrame_ToggleFrame", "startup", counters);
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn log_group_finder_result(source_after: &str, load_attempts: &str, counters: BoundaryCounters) {
    eprintln!(
        "OWNER|PVEFrame_ToggleFrame|source_after={source_after}|load_addon_calls={load_attempts}"
    );
    log_boundary_counters("PVEFrame_ToggleFrame", "after_consumer", counters);
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn run_group_finder_probe(env: &WowLuaEnv, addons: &[String]) -> GroupFinderObservation {
    let counters_at_startup = read_boundary_counters(env);
    let source_before = function_source(env, "PVEFrame_ToggleFrame");
    let loaded_before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GroupFinder')")
        .expect("check Group Finder load state");
    let owner_before = source_before.contains("Blizzard_GroupFinder/");
    let owner_order = addon_precedes(addons, "Blizzard_GroupFinder", "Blizzard_MicroMenu");
    log_group_finder_startup(
        &source_before,
        loaded_before,
        owner_before,
        owner_order,
        counters_at_startup,
    );

    let load_attempts = invoke_group_finder_consumers(env);
    let source_after = function_source(env, "PVEFrame_ToggleFrame");
    let counters_after_consumer = read_boundary_counters(env);
    log_group_finder_result(&source_after, &load_attempts, counters_after_consumer);
    GroupFinderObservation {
        source_before,
        loaded_before,
        owner_before,
        load_attempts,
        source_after,
        counters_at_startup,
        counters_after_consumer,
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn assert_group_finder_probe(observation: &GroupFinderObservation, baseline: bool) {
    assert_counter_install_counts(observation.counters_at_startup, baseline);
    assert_counter_install_counts(observation.counters_after_consumer, baseline);
    let fallback_calls = observation.counters_after_consumer.pve_calls
        - observation.counters_at_startup.pve_calls;
    if observation.owner_before {
        assert!(observation.loaded_before, "the Group Finder owner must be loaded before use");
        assert!(observation.load_attempts.is_empty(), "an active owner must not need LoadAddOn");
        assert!(observation.source_after.contains("Blizzard_GroupFinder/"));
        assert_eq!(fallback_calls, 0, "the fallback must not run when the owner is active");
    } else {
        assert_eq!(observation.load_attempts, "Blizzard_GroupFinder");
        assert!(fallback_calls > 0, "the retained bridge must be detected by its counter");
    }

    if baseline {
        eprintln!(
            "EXPECTED_BASE_DIFFERENCE|PVEFrame_ToggleFrame|fallback_installed={}|owner_before={}|fallback_calls={fallback_calls}",
            observation.counters_after_consumer.pve_install,
            observation.owner_before,
        );
    } else if cfg!(feature = "client-wowforever") {
        assert_eq!(observation.counters_after_consumer.pve_install, 1);
        assert!(fallback_calls > 0, "Forever is the retained-bridge positive control");
    } else {
        assert_eq!(observation.counters_after_consumer.pve_install, 0);
        assert_eq!(observation.counters_after_consumer.pve_calls, 0);
        assert_eq!(fallback_calls, 0, "retired profiles must not reach the PVE bridge");
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn probe_group_finder_consumer(env: &WowLuaEnv, addons: &[String], baseline: bool) {
    let observation = run_group_finder_probe(env, addons);
    assert_group_finder_probe(&observation, baseline);
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn read_print_handler_observation(
    env: &WowLuaEnv,
    addons: &[String],
) -> PrintHandlerObservation {
    let counters_at_startup = read_boundary_counters(env);
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
        "OWNER|setprinthandler|setter_source={setter_source}|caller_addon_loaded={caller_addon_loaded}|owner_before_caller={owner_order}|caller_registered={caller_registered}|handler_source={handler_source}"
    );
    log_boundary_counters("setprinthandler", "startup", counters_at_startup);
    PrintHandlerObservation {
        setter_source,
        owner_order,
        caller_registered,
        counters_at_startup,
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn assert_print_handler_observation(observation: &PrintHandlerObservation, baseline: bool) {
    assert_counter_install_counts(observation.counters_at_startup, baseline);
    let original_setter = observation.setter_source.contains("Blizzard_PrintHandler/");
    if baseline {
        eprintln!(
            "EXPECTED_BASE_DIFFERENCE|setprinthandler|fallback_installed={}|owner_before_caller={}|original_setter={}|caller_registered={}|fallback_calls={}",
            observation.counters_at_startup.print_install,
            observation.owner_order,
            original_setter,
            observation.caller_registered,
            observation.counters_at_startup.print_calls,
        );
        if observation.caller_registered && (!original_setter || !observation.owner_order) {
            assert!(observation.counters_at_startup.print_calls > 0);
        }
    } else {
        assert_eq!(observation.counters_at_startup.print_install, 0);
        assert_eq!(observation.counters_at_startup.print_calls, 0);
        assert!(original_setter, "candidate must select the original print owner");
        if observation.caller_registered {
            assert!(observation.owner_order, "the print owner must load before its caller");
        }
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn probe_print_handler_consumer(env: &WowLuaEnv, addons: &[String], baseline: bool) {
    let observation = read_print_handler_observation(env, addons);
    assert_print_handler_observation(&observation, baseline);
    if observation.setter_source.contains("Blizzard_PrintHandler/") {
        let stores_handler: bool = env
            .eval("local f = function() end; setprinthandler(f); return getprinthandler() == f")
            .expect("exercise the original print-handler owner");
        assert!(stores_handler, "the original print owner must store its handler");
    }

    let counters_after_consumer = read_boundary_counters(env);
    assert_counter_install_counts(counters_after_consumer, baseline);
    if !baseline {
        assert_eq!(counters_after_consumer.print_calls, 0);
    }
    log_boundary_counters(
        "setprinthandler",
        "after_consumer",
        counters_after_consumer,
    );
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn store_consumer_source(env: &WowLuaEnv) -> String {
    env.eval::<String>(
        "local f = CharacterSelectFrameMixin and CharacterSelectFrameMixin.OnUpdate; if type(f) ~= 'function' then f = CharacterSelect_OnUpdate end; if type(f) ~= 'function' then return '<missing>' end; return debug.getinfo(f, 'S').source or '<unknown>'",
    )
    .expect("read the original character-select consumer source")
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn invoke_store_consumer(env: &WowLuaEnv) -> bool {
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
fn read_store_observation(env: &WowLuaEnv, addons: &[String]) -> StoreObservation {
    let counters_at_startup = read_boundary_counters(env);
    let owner_source = function_source(env, "StoreFrame_WaitingForCharacterListUpdate");
    let consumer_source = store_consumer_source(env);
    let consumer_loaded = addons.iter().any(|name| name == "Blizzard_GlueXML");
    let owner_loaded = addons.iter().any(|name| name == "Blizzard_StoreUI");
    let owner_is_original = owner_source.contains("Blizzard_StoreUI/");
    eprintln!(
        "OWNER|StoreFrame_WaitingForCharacterListUpdate|owner_source={owner_source}|owner_loaded={owner_loaded}|consumer_source={consumer_source}|consumer_loaded={consumer_loaded}"
    );
    log_boundary_counters(
        "StoreFrame_WaitingForCharacterListUpdate",
        "startup",
        counters_at_startup,
    );
    StoreObservation {
        consumer_source,
        consumer_loaded,
        owner_loaded,
        owner_is_original,
        counters_at_startup,
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn assert_store_startup(observation: &StoreObservation, baseline: bool) {
    assert_counter_install_counts(observation.counters_at_startup, baseline);
    assert!(
        observation.consumer_source.contains("CharacterSelect.lua"),
        "the selected CharacterSelect consumer must be present"
    );
    assert!(observation.consumer_loaded, "the selected CharacterSelect consumer must load");
    if baseline {
        eprintln!(
            "EXPECTED_BASE_DIFFERENCE|StoreFrame_WaitingForCharacterListUpdate|fallback_installed={}|owner_is_original={}|owner_loaded={}",
            observation.counters_at_startup.store_install,
            observation.owner_is_original,
            observation.owner_loaded,
        );
    } else {
        assert!(
            observation.owner_loaded && observation.owner_is_original,
            "the original StoreUI owner must be active"
        );
        assert_eq!(observation.counters_at_startup.store_calls, 0);
    }
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn assert_store_consumer_result(
    observation: &StoreObservation,
    counters_after: BoundaryCounters,
    callback_called: bool,
    baseline: bool,
) {
    assert_counter_install_counts(counters_after, baseline);
    let fallback_call_delta = counters_after.store_calls - observation.counters_at_startup.store_calls;
    if observation.owner_is_original {
        assert_eq!(fallback_call_delta, 0);
        if !baseline {
            assert!(callback_called);
            assert_eq!(counters_after.store_calls, 0);
        }
    } else {
        assert!(baseline, "candidate must not call the Store fallback");
        assert_eq!(fallback_call_delta, 1);
        assert!(!callback_called);
    }
    if baseline {
        eprintln!(
            "EXPECTED_BASE_DIFFERENCE|StoreFrame_WaitingForCharacterListUpdate|owner_is_original={}|fallback_call_delta={fallback_call_delta}|consumer_callback_called={callback_called}",
            observation.owner_is_original,
        );
    }
    log_boundary_counters(
        "StoreFrame_WaitingForCharacterListUpdate",
        "after_consumer",
        counters_after,
    );
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn run_store_consumer_probe(
    env: &WowLuaEnv,
    observation: &StoreObservation,
    baseline: bool,
) {
    let callback_called = invoke_store_consumer(env);
    let counters_after = read_boundary_counters(env);
    assert_store_consumer_result(observation, counters_after, callback_called, baseline);
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
fn probe_store_consumer(env: &WowLuaEnv, addons: &[String], baseline: bool) {
    let observation = read_store_observation(env, addons);
    assert_store_startup(&observation, baseline);
    run_store_consumer_probe(env, &observation, baseline);
}

#[cfg(any(
    feature = "client-ptr",
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-mists"
))]
#[test]
fn editmode_glue_owner_boundaries_preserve_original_consumers() {
    let baseline = read_baseline_adapter_mode();
    let ui_dir = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let game_env = prepare_env(&ui_dir, ScreenKind::Game);
    let game_addons = load_startup_addons(&game_env, &ui_dir, ScreenKind::Game);
    probe_group_finder_consumer(&game_env, &game_addons, baseline);

    let glue_env = prepare_env(&ui_dir, ScreenKind::CharacterSelect);
    let glue_addons = load_startup_addons(&glue_env, &ui_dir, ScreenKind::CharacterSelect);
    probe_print_handler_consumer(&glue_env, &glue_addons, baseline);
    probe_store_consumer(&glue_env, &glue_addons, baseline);
}
