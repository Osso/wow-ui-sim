use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn private_auras_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PrivateAurasUI")
}

fn private_auras_toc() -> PathBuf {
    private_auras_dir().join("Blizzard_PrivateAurasUI.toc")
}

/// The pinned TOC selects its flavor body through `[Family]`
/// (`AddOns/Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.toc:8-9` in every
/// profile cache), which resolves to `Classic` on Wrath/Mists/Era/Anniversary
/// and to `Mainline` on Retail/PTR/WowForever (`src/toc/mod.rs:88-97`).
fn family_dir() -> &'static str {
    match wow_ui_sim::client_profile::ACTIVE {
        wow_ui_sim::client_profile::ClientProfile::Retail
        | wow_ui_sim::client_profile::ClientProfile::Ptr
        | wow_ui_sim::client_profile::ClientProfile::WowForever => "Mainline",
        wow_ui_sim::client_profile::ClientProfile::Wrath
        | wow_ui_sim::client_profile::ClientProfile::Mists
        | wow_ui_sim::client_profile::ClientProfile::Era
        | wow_ui_sim::client_profile::ClientProfile::Anniversary => "Classic",
    }
}

fn private_auras_toc_files() -> Vec<String> {
    let family = family_dir();
    vec![
        "Shared/PrivateAurasTooltip.lua".to_string(),
        format!("{family}/PrivateAurasTooltip.lua"),
        format!("{family}/PrivateAurasTooltip.xml"),
        "Blizzard_PrivateAurasUI.lua".to_string(),
        "Blizzard_PrivateAurasUI.xml".to_string(),
        "PrivateAuraInit.lua".to_string(),
    ]
}

/// The classic body declares three hard dependencies
/// (`mists|era|anniversary/AddOns/Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.toc:3-5`);
/// the mainline body adds `Blizzard_RaidWarning` and `Blizzard_VisualAlerts`
/// (`retail/AddOns/Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.toc:3-7`).
fn private_auras_dependencies() -> Vec<&'static str> {
    let mut deps = vec![
        "Blizzard_SharedXMLGame",
        "Blizzard_FrameXMLUtil",
        "Blizzard_GameTooltip",
    ];
    if family_dir() == "Mainline" {
        deps.push("Blizzard_RaidWarning");
        deps.push("Blizzard_VisualAlerts");
    }
    deps
}

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "CompactDispelDebuffMixin",
    "PrivateAuraMixin",
    "PrivateAuraAnchorContainerMixin",
    "PrivateAuraAnchorSingleMixin",
    "CompactUnitFrameDispelOverlayMixin",
];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBAL_ENV: &[&str] = &[
    "PrivateAuraUnitWatcherTemplate",
    "PrivateAuraTemplate",
    "CompactDispelDebuffTemplate",
    "CompactUnitFrameDispelOverlayTemplate",
];

const SCOPED_NAMED_FRAMES_HIDDEN_FROM_GLOBAL_ENV: &[&str] =
    &["RaidBossEmoteFramePrivate", "PrivateAurasTooltip"];

#[cfg(feature = "client-retail")]
#[derive(Debug)]
struct SecureColorCatalogSnapshot {
    get_colors_type: String,
    tradeskill_color_type: String,
    secure_tooltip_background_type: String,
    ordinary_tooltip_background_type: String,
    has_tradeskill_record: bool,
    has_tooltip_background_record: bool,
}

#[cfg(feature = "client-retail")]
#[derive(Debug)]
struct SecureColorRouteSnapshot {
    create_color_source: String,
    create_color_type: String,
    create_color_uses_secure_environment: bool,
    current_environment_is_secure: bool,
    catalog: SecureColorCatalogSnapshot,
}

#[cfg(feature = "client-retail")]
fn capture_secure_color_route_snapshot(env: &WowLuaEnv) -> SecureColorRouteSnapshot {
    let (create_color_source, create_color_type, create_color_uses_secure_environment) =
        read_secure_create_color(env);
    let current_environment_is_secure = probe_secure_current_environment(env);
    let catalog = read_secure_color_catalog(env);
    SecureColorRouteSnapshot {
        create_color_source,
        create_color_type,
        create_color_uses_secure_environment,
        current_environment_is_secure,
        catalog,
    }
}

#[cfg(feature = "client-retail")]
fn assert_secure_color_route(snapshot: &SecureColorRouteSnapshot) {
    assert_eq!(
        snapshot.create_color_type, "function",
        "SharedXMLBase Color.lua must define CreateColor in the secure environment"
    );
    assert!(
        snapshot.create_color_uses_secure_environment,
        "CreateColor must use the shared secure environment"
    );
    assert!(
        snapshot.current_environment_is_secure,
        "GetCurrentEnvironment must return the secure environment from secure code"
    );
    assert_eq!(
        snapshot.catalog.get_colors_type, "function",
        "secure C_UIColor must expose the GetColors catalog"
    );
    assert_eq!(
        snapshot.catalog.tradeskill_color_type, "table",
        "Color.lua must publish an existing C_UIColor record to secureenv"
    );
    assert!(
        snapshot.catalog.has_tradeskill_record,
        "the existing C_UIColor color record must be published by Color.lua"
    );
}

#[cfg(feature = "client-retail")]
fn read_secure_create_color(env: &WowLuaEnv) -> (String, String, bool) {
    env.eval(
        r#"
            local createColor = rawget(__secureenv, 'CreateColor')
            local source = type(createColor) == 'function'
                and debug.getinfo(createColor, 'S').source or ''
            return source,
                type(createColor),
                type(createColor) == 'function' and debug.getfenv(createColor) == __secureenv
        "#,
    )
    .expect("secure CreateColor environment query should evaluate")
}

#[cfg(feature = "client-retail")]
fn probe_secure_current_environment(env: &WowLuaEnv) -> bool {
    env.exec_rilua_secure(
        r#"
            rawset(
                _G,
                '__private_auras_current_environment_is_self',
                GetCurrentEnvironment() == _G
            )
        "#,
    )
    .expect("secure current-environment probe should execute");
    env.eval("return rawget(__secureenv, '__private_auras_current_environment_is_self') == true")
        .expect("secure current-environment result should evaluate")
}

#[cfg(feature = "client-retail")]
fn read_secure_color_catalog(env: &WowLuaEnv) -> SecureColorCatalogSnapshot {
    let (
        get_colors_type,
        tradeskill_color_type,
        secure_tooltip_background_type,
        ordinary_tooltip_background_type,
        has_tradeskill_record,
        has_tooltip_background_record,
    ): (String, String, String, String, bool, bool) = env
        .eval(
            r#"
                local secureColors = rawget(__secureenv, 'C_UIColor')
                local getColors = secureColors and rawget(secureColors, 'GetColors')
                local hasTradeskillRecord = false
                local hasTooltipBackgroundRecord = false
                if type(getColors) == 'function' then
                    for _, dbColor in ipairs(getColors()) do
                        hasTradeskillRecord = hasTradeskillRecord
                            or dbColor.baseTag == 'TRADESKILL_EXPERIENCE_COLOR'
                        hasTooltipBackgroundRecord = hasTooltipBackgroundRecord
                            or dbColor.baseTag == 'TOOLTIP_DEFAULT_BACKGROUND_COLOR'
                    end
                end
                return type(getColors),
                    type(rawget(__secureenv, 'TRADESKILL_EXPERIENCE_COLOR')),
                    type(rawget(__secureenv, 'TOOLTIP_DEFAULT_BACKGROUND_COLOR')),
                    type(rawget(_G, 'TOOLTIP_DEFAULT_BACKGROUND_COLOR')),
                    hasTradeskillRecord,
                    hasTooltipBackgroundRecord
            "#,
        )
        .expect("secure C_UIColor color catalog query should evaluate");
    SecureColorCatalogSnapshot {
        get_colors_type,
        tradeskill_color_type,
        secure_tooltip_background_type,
        ordinary_tooltip_background_type,
        has_tradeskill_record,
        has_tooltip_background_record,
    }
}

#[cfg(feature = "client-retail")]
fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.exec(
        "rawset(_G, 'TooltipDataProcessor', nil); \
         rawset(_G, 'AddTooltipDataAccessor', nil); \
         rawset(__secureenv, 'TooltipDataProcessor', nil); \
         rawset(__secureenv, 'AddTooltipDataAccessor', nil)",
    )
    .expect("disable the TooltipDataProcessor fallback before loading addons");

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[cfg(feature = "client-retail")]
#[test]
fn secure_private_auras_tooltip_consumer_runs_without_errors() {
    let env = load_full_game_ui();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PrivateAurasUI')")
        .expect("PrivateAurasUI loaded-state query should succeed");
    assert!(
        loaded,
        "normal Retail startup must load Blizzard_PrivateAurasUI"
    );
    let secure_environment_has_no_snapshot_only_globals: bool = env
        .eval(
            r#"
                return type(rawget(_G, 'debug')) == 'table'
                    and rawget(_G, 'coroutine') ~= nil
                    and rawget(__secureenv, 'debug') == nil
                    and rawget(__secureenv, 'coroutine') == nil
            "#,
        )
        .expect("secure environment visibility query should succeed");
    assert!(
        secure_environment_has_no_snapshot_only_globals,
        "debug and coroutine must stay out of the fixed secure environment"
    );

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PrivateAurasUI")
                || message.contains("PrivateAurasTooltip")
                || message.contains("TooltipDataProcessor")
                || message.contains("TextureUtil.lua")
                || message.contains("TooltipDataHandler.lua")
        })
        .cloned()
        .collect();
    let registration_snapshot: (String, String, String) = env
        .eval(
            r#"
                return type(rawget(__secureenv, 'CopyTable')),
                    type(rawget(__secureenv, 'GetFrameMetatable')),
                    type(rawget(__secureenv, 'TextureKitConstants'))
            "#,
        )
        .expect("secure registration snapshot should evaluate");
    let secure_mixin_snapshot: (String, String, String, String) = env
        .eval(
            r#"
                local privateAura = rawget(__secureenv, 'PrivateAurasTooltipMixin')
                local gameTooltip = rawget(__secureenv, 'GameTooltipDataMixin')
                return type(privateAura),
                    type(privateAura and rawget(privateAura, 'OnLoad_PrivateAuraTooltip')),
                    type(gameTooltip),
                    type(gameTooltip and rawget(gameTooltip, 'OnLoad'))
            "#,
        )
        .expect("secure tooltip mixin snapshot should evaluate");
    let template_method_binders: (String, String) = env
        .eval(
            "return type(rawget(_G, '__wow_bind_xml_method')), \
             type(rawget(__secureenv, '__wow_bind_xml_method'))",
        )
        .expect("template method binder visibility should be observable");
    let tooltip_callback_globals: (String, String, String, String) = env
        .eval(
            r#"
                return type(rawget(__secureenv, 'GameTooltip_OnLoad')),
                    type(rawget(__secureenv, 'SharedTooltip_OnLoad')),
                    type(rawget(_G, 'GameTooltip_OnLoad')),
                    type(rawget(_G, 'SharedTooltip_OnLoad'))
            "#,
        )
        .expect("secure tooltip callback global snapshot should evaluate");
    let tooltip_method_probe: (bool, String) = env
        .eval(
            r#"
                local mixin = rawget(__secureenv, 'PrivateAurasTooltipMixin')
                local frame = CreateFrame('GameTooltip')
                local ok, result = pcall(rawget(mixin, 'OnLoad_PrivateAuraTooltip'), frame)
                return ok, tostring(result)
            "#,
        )
        .expect("secure tooltip method probe should evaluate");
    let color_route_snapshot = capture_secure_color_route_snapshot(&env);
    let secure_dependency_states: Vec<(String, bool)> = {
        let state = env.state().borrow();
        private_auras_dependencies()
            .into_iter()
            .map(|name| {
                let loaded = state
                    .addons
                    .iter()
                    .find(|addon| addon.folder_name == name)
                    .is_some_and(|addon| addon.is_loaded_in_environment(true));
                (name.to_string(), loaded)
            })
            .collect()
    };
    let producer_source: (String, String, String, bool, bool) = env
        .eval(
            r#"
                local processor = rawget(__secureenv, 'TooltipDataProcessor')
                local accessor = rawget(__secureenv, 'AddTooltipDataAccessor')
                local source = type(accessor) == 'function'
                    and debug.getinfo(accessor, 'S').source or ''
                return type(processor),
                    type(processor and rawget(processor, 'AllTypes')),
                    source,
                    processor ~= rawget(_G, 'TooltipDataProcessor'),
                    type(accessor) == 'function' and debug.getfenv(accessor) == __secureenv
            "#,
        )
        .expect("secure TooltipDataProcessor producer provenance should evaluate");
    assert!(
        secure_dependency_states.iter().all(|(_, loaded)| *loaded),
        "every PrivateAurasUI dependency must complete a secure load: {secure_dependency_states:?}"
    );
    assert!(
        load_errors.is_empty(),
        "PrivateAuras dependencies and the PrivateAurasTooltip consumer must load without errors:\n  {}\nsecure dependencies: {secure_dependency_states:?}\nsecure mixins: {secure_mixin_snapshot:?}\ntooltip callbacks: {tooltip_callback_globals:?}\ntooltip method: {tooltip_method_probe:?}\ncolor route: {color_route_snapshot:?}\nproducer: {producer_source:?}\nregistrations: {registration_snapshot:?}\ntemplate method binders: {template_method_binders:?}",
        load_errors.join("\n  ")
    );
    assert_secure_color_route(&color_route_snapshot);
    assert_eq!(
        producer_source.0, "table",
        "secure dependency must define TooltipDataProcessor"
    );
    assert_eq!(
        producer_source.1, "string",
        "the original producer must define AllTypes"
    );
    assert!(
        producer_source.2.contains("TooltipDataHandler.lua"),
        "secure AddTooltipDataAccessor must come from TooltipDataHandler.lua, got {}",
        producer_source.2
    );
    assert!(
        producer_source.3,
        "secure execution must create its own TooltipDataProcessor table"
    );
    assert!(
        producer_source.4,
        "the dependency accessor function must use the shared secure table as its environment"
    );
    assert_eq!(
        registration_snapshot.0, "function",
        "SharedXMLBase TableUtil.lua must provide CopyTable in secureenv"
    );
    assert_eq!(
        registration_snapshot.1, "function",
        "GetFrameMetatable must be published in secureenv"
    );
    assert_eq!(
        registration_snapshot.2, "table",
        "TextureKitConstants defined during secure execution must be exported"
    );

    let secure_exports_are_isolated: bool = env
        .eval(
            "local cTimer = rawget(__secureenv, 'C_Timer'); \
             local store = rawget(__secureenv, 'C_StoreSecure'); \
             local color = rawget(__secureenv, 'C_UIColor'); \
             local secureEnum = rawget(__secureenv, 'Enum'); \
             rawset(_G, '__secure_env_ordinary_only_probe', true); \
             return type(rawget(__secureenv, 'Mixin')) == 'function' \
                 and type(rawget(__secureenv, 'CreateFromMixins')) == 'function' \
                 and type(cTimer) == 'table' \
                 and cTimer ~= rawget(_G, 'C_Timer') \
                 and type(cTimer.After) == 'function' \
                 and type(store) == 'table' \
                 and store ~= rawget(_G, 'C_StoreSecure') \
                 and type(store.GetProductGroups) == 'function' \
                 and type(rawget(__secureenv, 'CreateFrame')) == 'function' \
                 and type(rawget(__secureenv, 'CreateSecureDelegate')) == 'function' \
                 and type(rawget(__secureenv, 'GetFrameMetatable')) == 'function' \
                 and type(GetFrameMetatable().__index) == 'table' \
                 and type(color) == 'table' \
                 and color ~= rawget(_G, 'C_UIColor') \
                 and type(color.GetColors) == 'function' \
                 and type(secureEnum) == 'table' \
                 and secureEnum ~= rawget(_G, 'Enum') \
                 and type(rawget(rawget(secureEnum, 'TooltipDataType'), 'Unit')) == 'number' \
                 and rawget(__secureenv, '__secure_env_ordinary_only_probe') == nil",
        )
        .expect("secure export isolation query should succeed");
    assert!(
        secure_exports_are_isolated,
        "secure registrations must stay isolated from ordinary-only globals"
    );
}

#[cfg(feature = "client-retail")]
fn retail_full_ui_errors_from(env: &WowLuaEnv, markers: &[&str]) -> Vec<String> {
    env.state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| markers.iter().any(|marker| message.contains(marker)))
        .cloned()
        .collect()
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_secure_shared_xmlbase_publishes_add_source_location_exclude() {
    let env = load_full_game_ui();
    let consumer_errors = retail_full_ui_errors_from(&env, &["FrameFactory.lua", "Pools.lua"]);
    assert!(
        consumer_errors.is_empty(),
        "the secure SharedXMLBase consumers at FrameFactory.lua:64 and Pools.lua:866 must run: {consumer_errors:?}"
    );

    let published: (String, String, bool) = env
        .eval(
            r#"
            local ordinary = rawget(_G, "AddSourceLocationExclude")
            local secure = rawget(__secureenv, "AddSourceLocationExclude")
            return type(ordinary), type(secure), not rawequal(ordinary, secure)
            "#,
        )
        .expect("AddSourceLocationExclude publication probe");
    assert_eq!(published, ("function".into(), "function".into(), true));
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_secure_shared_xml_publishes_table_count_for_account_util() {
    let env = load_full_game_ui();
    let consumer_errors = retail_full_ui_errors_from(&env, &["AccountUtil.lua"]);
    assert!(
        consumer_errors.is_empty(),
        "the secure AccountUtil.lua:129 table.count consumer must run: {consumer_errors:?}"
    );

    let published: (bool, bool, bool, bool, bool, bool, i32) = env
        .eval(
            r#"
            local ordinary = rawget(_G, "table")
            local secure = rawget(__secureenv, "table")
            local ordinaryCount = type(ordinary) == "table" and rawget(ordinary, "count")
            local secureCount = type(secure) == "table" and rawget(secure, "count")
            local count = type(secureCount) == "function" and secureCount({ first = 1, second = 2 }) or -1
            return type(ordinary) == "table",
                type(secure) == "table",
                ordinary ~= secure,
                type(ordinaryCount) == "function",
                type(secureCount) == "function",
                not rawequal(ordinaryCount, secureCount),
                count
            "#,
        )
        .expect("table.count publication probe");
    assert_eq!(published, (true, true, true, true, true, true, 2));
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_secure_colors_publishes_existing_color_override_consumers() {
    let env = load_full_game_ui();
    let consumer_errors = retail_full_ui_errors_from(&env, &["ColorManager.lua"]);
    assert!(
        consumer_errors.is_empty(),
        "the secure Blizzard_Colors ColorManager.lua consumer must run: {consumer_errors:?}"
    );

    let published: bool = env
        .eval(
            r#"
            local ordinary = rawget(_G, "C_ColorOverrides")
            local secure = rawget(__secureenv, "C_ColorOverrides")
            if type(ordinary) ~= "table" or type(secure) ~= "table" or ordinary == secure then
                return false
            end
            for _, name in ipairs({ "GetColorForQuality", "GetColorOverrideInfo" }) do
                local ordinaryFunction = rawget(ordinary, name)
                local secureFunction = rawget(secure, name)
                if type(ordinaryFunction) ~= "function"
                    or type(secureFunction) ~= "function"
                    or rawequal(ordinaryFunction, secureFunction) then
                    return false
                end
            end
            return true
            "#,
        )
        .expect("C_ColorOverrides publication probe");
    assert!(
        published,
        "publish the existing C_ColorOverrides handlers separately"
    );
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_secure_shared_xml_publishes_account_store_constants() {
    let env = load_full_game_ui();
    let consumer_errors = retail_full_ui_errors_from(&env, &["GameRulesUtil.lua"]);
    assert!(
        consumer_errors.is_empty(),
        "the secure GameRulesUtil.lua AccountStoreConsts consumer must run: {consumer_errors:?}"
    );

    let constants: (bool, bool, bool, bool, bool, f64, f64, f64, f64) = env
        .eval(
            r#"
            local ordinaryConstants = rawget(_G, "Constants")
            local secureConstants = rawget(__secureenv, "Constants")
            local ordinary = type(ordinaryConstants) == "table"
                and rawget(ordinaryConstants, "AccountStoreConsts")
            local secure = type(secureConstants) == "table"
                and rawget(secureConstants, "AccountStoreConsts")
            return type(ordinaryConstants) == "table",
                type(secureConstants) == "table",
                ordinaryConstants ~= secureConstants,
                type(ordinary) == "table",
                type(secure) == "table",
                not rawequal(ordinary, secure),
                secure and secure.PlunderstormStoreFrontID or -1,
                secure and secure.WowhackStoreFrontID or -1,
                secure and secure.PlunderstormPlunderCurrencyID or -1,
                secure and secure.KegLegRenownCurrencyID or -1
            "#,
        )
        .expect("AccountStoreConsts publication probe");
    assert_eq!(
        constants,
        (true, true, true, true, true, true, 1.0, 3.0, 3139.0, 2814.0)
    );
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_private_auras_xml_uses_the_shared_secure_uiparent_frame() {
    let env = load_full_game_ui();
    let consumer_errors = retail_full_ui_errors_from(&env, &["PrivateAurasTooltip"]);
    assert!(
        consumer_errors.is_empty(),
        "the secure PrivateAurasTooltip.xml parent consumer must load: {consumer_errors:?}"
    );

    let published: (bool, bool, bool) = env
        .eval(
            r#"
            local ordinary = rawget(_G, "UIParent")
            local secure = rawget(__secureenv, "UIParent")
            return type(ordinary) == "table",
                type(secure) == "table",
                rawequal(ordinary, secure)
            "#,
        )
        .expect("UIParent named-frame publication probe");
    assert_eq!(published, (true, true, true));

    let state = env.state();
    let state = state.borrow();
    let ui_parent = state
        .widgets
        .get_id_by_name("UIParent")
        .expect("the built-in UIParent frame");
    let tooltip = state
        .widgets
        .get_id_by_name("PrivateAurasTooltip")
        .expect("the original PrivateAurasTooltip XML frame");
    assert_eq!(
        state.widgets.get(tooltip).and_then(|frame| frame.parent_id),
        Some(ui_parent),
        "PrivateAurasTooltip.xml must parent its frame to the UIParent frame"
    );
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&private_auras_dir()).expect("Blizzard_PrivateAurasUI TOC resolves");
    assert_eq!(
        resolved,
        private_auras_toc(),
        "Blizzard_PrivateAurasUI ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The addon hosts the in-world private-aura UI (raid-frame dispel overlays + \
         per-unit private aura buttons), all driven from C_UnitAurasPrivate callbacks, \
         a single bare TOC drives every flavor"
    );

    let mainline = private_auras_dir().join("Blizzard_PrivateAurasUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_toc_declares_secure_game_only_addon() {
    let toc = TocFile::from_file(&private_auras_toc()).expect("Blizzard_PrivateAurasUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on the Game screen so the \
         C_UnitAurasPrivate anchor callbacks are wired up before any unit-aura packet arrives"
    );
    assert!(!toc.is_load_first());
    assert!(
        toc.is_secure_env(),
        "TOC must declare `## UseSecureEnvironment: 1` — the addon manipulates secure \
         frames (raid-frame dispel overlays gated on protected unit-frame state) so its \
         compiled chunks must run in the secure environment. `is_secure_env` at \
         src/toc.rs:237-242 returns true when this flag is present"
    );

    assert!(
        !toc.is_game_type_restricted(),
        "TOC must NOT declare `## AllowLoadGameType:` — private auras are universal across \
         retail / classic / etc."
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) must enable Game screen — `allows_screen` at \
         src/toc.rs:306-313 uses case-insensitive comparison so the lowercase value matches"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: game` must EXCLUDE {screen:?} — private auras only matter \
             once unit frames exist, glue screens have no UnitID surface to anchor to"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_toc_uses_singular_dep_form() {
    let toc = TocFile::from_file(&private_auras_toc()).expect("Blizzard_PrivateAurasUI TOC parses");

    let dependencies = toc.dependencies();
    assert_eq!(
        dependencies,
        private_auras_dependencies(),
        "the pinned TOC declares its hard dependencies through singular `## Dep:` lines"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — the addon's only soft surface is C_UnitAurasPrivate \
         (a C namespace, not an addon dep)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — anchor state is owned by the C_UnitAurasPrivate engine \
         side, the Lua side is a stateless view"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(private_auras_toc())
        .expect("Blizzard_PrivateAurasUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PrivateAurasUI"),
        "TOC must declare `## Title: Blizzard_PrivateAurasUI` exactly"
    );
    assert!(
        raw.contains("## UseSecureEnvironment: 1"),
        "TOC must declare `## UseSecureEnvironment: 1` — secures the addon environment so \
         protected raid-frame manipulation does not taint the secure dispatch path"
    );
    assert!(
        raw.contains("## AllowLoad: game"),
        "TOC must declare `## AllowLoad: game` (lowercase exactly) — case-insensitive \
         matching means the lowercase form still routes through the Game-only screen gate"
    );

    for dep_line in private_auras_dependencies() {
        let dep_line = format!("## Dep: {dep_line}");
        assert!(
            raw.contains(&dep_line),
            "TOC must contain `{dep_line}` in raw bytes; the parser preserves singular \
             Dep entries as hard dependencies"
        );
    }

    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — it uses the non-canonical singular \
         `## Dep:` form instead (likely a Blizzard internal build-tool artifact)"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare `## RequiredDep:` or `## RequiredDeps:`"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on Game screen"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless C-API view"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — minimal header on this internal addon"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_toc_lists_files_in_order() {
    let toc = TocFile::from_file(&private_auras_toc()).expect("Blizzard_PrivateAurasUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed,
        private_auras_toc_files(),
        "TOC body must list tooltip setup first, then Blizzard_PrivateAurasUI.lua \
         (publishes 5 mixin globals + addon-private PrivateAuras.unitWatchers / \
         PrivateAuras.PrivateAuraUnitWatcher bindings), Blizzard_PrivateAurasUI.xml \
         (defines aura templates inside a `<ScopedModifier forbidden hideFromGlobalEnv>` \
         wrapper), and finally PrivateAuraInit.lua (wires C_UnitAurasPrivate callbacks). \
         Order matters: PrivateAuraInit.lua must run last because it asserts \
         `PrivateAuras.unitWatchers ~= nil` which the .lua chunk creates"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_appears_only_in_game_screen_eager_discovery() {
    let ui = blizzard_ui_dir();

    let game = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game
        .iter()
        .any(|(name, _)| name == "Blizzard_PrivateAurasUI");
    assert!(
        game_found,
        "Blizzard_PrivateAurasUI must appear in eager discovery for ScreenKind::Game — \
         `## AllowLoad: game` opens the addon to Game screen only and there is no \
         LoadOnDemand filter to exclude it"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PrivateAurasUI");
        assert!(
            !found,
            "Blizzard_PrivateAurasUI must NOT appear in eager discovery for {screen:?} — \
             `## AllowLoad: game` excludes glue screens"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PrivateAurasUI");
    assert!(
        found,
        "Blizzard_PrivateAurasUI must appear in `discover_all_blizzard_addons` — the \
         unfiltered inventory at src/loader/mod.rs lists every parseable Blizzard_* TOC \
         regardless of AllowLoad gating"
    );
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_is_addon_loaded_after_game_screen_boot(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PrivateAurasUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PrivateAurasUI') must return true after the \
         eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_reads_raid_warning_util_and_publishes_mixins_into_secure_env(env: &WowLuaEnv) {
    let raid_warning_util_is_available: bool = env
        .eval(
            "local util = rawget(__secureenv, 'RaidWarningUtil'); \
             return type(util) == 'table' \
                and type(util.MessageType) == 'table' \
                and type(util.MessageType.BossEmote) == 'number' \
                and type(util.UpdateCenterScreenAnchors) == 'function'",
        )
        .expect("secure RaidWarningUtil probe should succeed");

    assert!(
        raid_warning_util_is_available,
        "Blizzard_PrivateAurasUI must read Blizzard_RaidWarning's RaidWarningUtil through \
         __secureenv; its secure XML references RaidWarningUtil.MessageTypes while its TOC \
         declares Blizzard_RaidWarning as a hard dependency"
    );

    for name in PUBLIC_MIXIN_GLOBALS {
        let in_secure_env: String = env
            .eval(&format!("return type(rawget(__secureenv, '{name}'))"))
            .unwrap_or_else(|err| panic!("rawget(__secureenv, '{name}') probe failed: {err}"));
        assert_eq!(
            in_secure_env, "table",
             "__secureenv.{name} must publish as a table — Blizzard_PrivateAurasUI.lua \
             runs under `## UseSecureEnvironment: 1`, so mark_secure_state at \
             src/lua_api/globals/security/secure_env.rs:82 swaps the chunk's fenv to \
             __secureenv (a shallow _G copy without a live `_G` fallback). Plain \
             top-level assignments like `CompactDispelDebuffMixin = {{}}` therefore \
             write into __secureenv, not _G. The 5 mixins exposed are: \
             CompactDispelDebuffMixin (raid-frame dispel overlay debuff icon), \
             PrivateAuraMixin (per-aura button modeled on AuraButtonMixin with \
             OnLoad/Reset/ShowTooltip/OnEnter/OnLeave/OnUpdate), \
             PrivateAuraAnchorContainerMixin (per-unit anchor container that pools \
             PrivateAuraTemplate frames), PrivateAuraAnchorSingleMixin (single-anchor \
             variant), CompactUnitFrameDispelOverlayMixin (raid-frame fill+highlight \
             overlay using RaidFrame-Dispel-Fill / RaidFrame-DispelHighlight atlases)"
        );

        let in_plain_g: String = env
            .eval(&format!("return type(rawget(_G, '{name}'))"))
            .unwrap_or_else(|err| panic!("rawget(_G, '{name}') probe failed: {err}"));
        assert_eq!(
            in_plain_g, "nil",
            "rawget(_G, '{name}') must be nil — secure-env writes do NOT propagate back \
             to _G. Insecure code reaches these mixins via __secureenv directly, or \
             through the XML mixin attribute (resolved by the loader against both envs)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_virtual_templates_are_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBAL_ENV {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual XML templates are stored in the template \
             registry (consumed via inherits=...) and never become _G entries. The 4 \
             virtual templates here (PrivateAuraUnitWatcherTemplate, PrivateAuraTemplate, \
             CompactDispelDebuffTemplate, CompactUnitFrameDispelOverlayTemplate) define \
             the per-unit watcher Frame, the per-aura button (with Cooldown child \
             inheriting CooldownFrameTemplate, hideCountdownNumbers/reverse/useParentLevel), \
             the 14x14 dispel-debuff Texture (parentArray=dispelDebuffFrames), and the \
             dispel overlay Frame (frameLevel=200, with 3 inherited \
             CompactDispelDebuffTemplate textures). All wrapped in a `<ScopedModifier \
             forbidden=true hideFromGlobalEnv=true>` block"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_scoped_named_frames_stay_out_of_global_env(env: &WowLuaEnv) {

    for frame in SCOPED_NAMED_FRAMES_HIDDEN_FROM_GLOBAL_ENV {
        let kind: String = env
            .eval(&format!("return type(rawget(_G, '{frame}'))"))
            .unwrap_or_else(|err| panic!("rawget(_G, '{frame}') probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "rawget(_G, '{frame}') must be nil — the XML wraps \
             RaidBossEmoteFramePrivate and PrivateAurasTooltip in \
             `<ScopedModifier forbidden=\"true\" hideFromGlobalEnv=\"true\">`, so named \
             frames inside the scope are created without publishing global bindings"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_private_auras_ui_xml_uses_scoped_modifier_wrapper() {
    let raw = std::fs::read_to_string(private_auras_dir().join("Blizzard_PrivateAurasUI.xml"))
        .expect("Blizzard_PrivateAurasUI.xml reads utf-8");

    assert!(
        raw.contains("<ScopedModifier forbidden=\"true\" hideFromGlobalEnv=\"true\">"),
        "XML must open the frame block with `<ScopedModifier forbidden=\"true\" \
         hideFromGlobalEnv=\"true\">` — this wrapper marks every contained frame as \
         forbidden (secure-restricted, IsForbidden() returns true) AND keeps them out of \
         _G even when they declare a `name=...` attribute. Handled by the simulator at \
         src/loader/xml_file.rs:82,103-106 via process_scoped_modifier"
    );
    assert!(
        raw.contains("</ScopedModifier>"),
        "XML must close the wrapper — every frame definition lives inside the single \
         ScopedModifier block"
    );

    assert!(
        raw.contains("inherits=\"AuraButtonArtTemplate\""),
        "PrivateAuraTemplate must inherit AuraButtonArtTemplate — the per-aura button \
         template reuses the standard aura-button art surface (icon, border, count \
         FontString, etc.) and only swaps in Private-aura-specific scripts via the \
         PrivateAuraMixin"
    );
    assert!(
        raw.contains("inherits=\"CooldownFrameTemplate\""),
        "PrivateAuraTemplate's Cooldown child must inherit CooldownFrameTemplate"
    );
    assert!(
        raw.contains(&format!(
            "inherits=\"{}\"",
            raid_boss_emote_template()
        )),
        "RaidBossEmoteFramePrivate must inherit {} — the classic body (\
         `mists|era|anniversary/…/Blizzard_PrivateAurasUI.xml:49`) uses the plain raid \
         boss emote template, while the mainline body (line 82) uses \
         RaidWarningFrameTemplate for the same frame",
        raid_boss_emote_template()
    );
    let tooltip_path = private_auras_dir().join(family_dir()).join("PrivateAurasTooltip.xml");
    let tooltip_raw = std::fs::read_to_string(&tooltip_path)
        .unwrap_or_else(|err| panic!("{} reads utf-8: {err}", tooltip_path.display()));
    assert!(
        tooltip_raw.contains(&format!("inherits=\"{}\"", tooltip_template())),
        "PrivateAurasTooltip must inherit {} — `{}` differs from the mainline \
         `Mainline/PrivateAurasTooltip.xml:4` SharedTooltipArtTemplate",
        tooltip_template(),
        tooltip_path.display()
    );
}
}

/// `AddOns/Blizzard_PrivateAurasUI/Mainline/PrivateAurasTooltip.xml:4` inherits the
/// shared tooltip art surface; `Classic/PrivateAurasTooltip.xml:4` inherits
/// `GameTooltipTemplate` instead.
fn tooltip_template() -> &'static str {
    if family_dir() == "Mainline" {
        "SharedTooltipArtTemplate"
    } else {
        "GameTooltipTemplate"
    }
}

/// `AddOns/Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.xml:82` on the mainline
/// bodies names `RaidWarningFrameTemplate`, which requires the Blizzard_RaidWarning
/// dependency their TOCs declare; the classic bodies (line 49) use
/// `RaidBossEmoteFrameTemplate` instead.
fn raid_boss_emote_template() -> &'static str {
    if family_dir() == "Mainline" {
        "RaidWarningFrameTemplate"
    } else {
        "RaidBossEmoteFrameTemplate"
    }
}

prefork_full_ui_case! {
fn blizzard_private_auras_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PrivateAurasUI")
                || message.contains("PrivateAura")
                || message.contains("CompactDispelDebuff")
                || message.contains("CompactUnitFrameDispelOverlay")
                || message.contains("PrivateAurasTooltip")
                || message.contains("RaidBossEmoteFramePrivate")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PrivateAurasUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}
