use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

const ROOT: &str = "Blizzard_AuctionHouseUI";
const COLORS: &str = "Blizzard_Colors";
const HELP_PLATE: &str = "Blizzard_HelpPlate";
const MANAGED_FRAME_SYSTEM: &str = "Blizzard_ManagedFrameSystem";

post_wrath_ui_fixture! {
#[test]
fn auction_house_load_addon_loads_toc_dependencies_before_root() {
    let toc = TocFile::from_file(&auction_house_toc()).expect("AuctionHouse TOC should parse");
    assert_eq!(
        toc.dependencies(),
        expected_dependencies()
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>(),
        "`{ROOT}` must keep the TOC dependency order used by runtime LoadAddOn"
    );

    // The TOC-order assertion above is this test's subject. The smoke shape
    // additionally needs Blizzard_TimerunningUtil, whose pinned classic TOC is
    // `## AllowLoadGameType: mainline`, so the classic caches have no eligible
    // Game TOC for that harness dependency. The dependency-load behaviour
    // itself stays covered on Retail, PTR and Forever.
    if toc_layout::classic_cache() {
        return;
    }

    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        mark_declared_dependencies_unloaded(env);
        clear_recorded_lua_errors(env);
        install_addon_loaded_trace(env);
        assert_addons_start_unloaded(env);

        let (loaded, reason): (bool, Option<String>) = env
            .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuctionHouseUI")"#)
            .expect("C_AddOns.LoadAddOn should return");
        assert!(loaded, "`{ROOT}` should load: {reason:?}");

        assert_dependency_events_precede_root(env);

        let errors = recorded_lua_errors(env);
        assert!(
            errors.is_empty(),
            "`{ROOT}` dependency load emitted Lua errors:\n{}",
            errors.join("\n")
        );
    });
}
}

/// Retail, PTR and Forever declare Colors, HelpPlate and ManagedFrameSystem.
/// The classic pinned `_Classic.toc` declares HelpPlate first and Colors
/// second; Blizzard_ManagedFrameSystem is not part of the classic caches.
fn expected_dependencies() -> &'static [&'static str] {
    if matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Mists
            | wow_ui_sim::client_profile::ClientProfile::Era
            | wow_ui_sim::client_profile::ClientProfile::Anniversary
    ) {
        &[HELP_PLATE, COLORS]
    } else {
        &[COLORS, HELP_PLATE, MANAGED_FRAME_SYSTEM]
    }
}

fn auction_house_toc() -> std::path::PathBuf {
    wow_ui_sim::loader::find_toc_file(&blizzard_ui_dir().join(ROOT))
        .expect("Blizzard_AuctionHouseUI TOC must resolve")
}

fn install_addon_loaded_trace(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        __auctionHouseDependencyLoadEvents = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("ADDON_LOADED")
        frame:SetScript("OnEvent", function(_, _, addonName)
            table.insert(__auctionHouseDependencyLoadEvents, addonName)
        end)
        "#,
    )
    .expect("ADDON_LOADED dependency trace should install");
}

fn mark_declared_dependencies_unloaded(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let dependencies = expected_dependencies();
    let mut state = env.state().borrow_mut();
    for addon in &mut state.addons {
        if dependencies.iter().any(|name| addon.folder_name == *name) {
            addon.loaded = false;
        }
    }
}

fn assert_addons_start_unloaded(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (colors_loaded, help_plate_loaded, managed_frame_system_loaded, root_loaded): (
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            return C_AddOns.IsAddOnLoaded("Blizzard_Colors"),
                   C_AddOns.IsAddOnLoaded("Blizzard_HelpPlate"),
                   C_AddOns.IsAddOnLoaded("Blizzard_ManagedFrameSystem"),
                   C_AddOns.IsAddOnLoaded("Blizzard_AuctionHouseUI")
            "#,
        )
        .expect("initial addon loaded-state probe should run");

    assert!(
        !colors_loaded
            && !help_plate_loaded
            && !managed_frame_system_loaded
            && !root_loaded,
        "the declared dependencies and `{ROOT}` must start unloaded"
    );
}

fn assert_dependency_events_precede_root(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let load_events: Vec<String> = env
        .eval("return __auctionHouseDependencyLoadEvents")
        .expect("ADDON_LOADED dependency trace should be readable");

    let root_index = addon_event_index(&load_events, ROOT);
    for dependency in expected_dependencies() {
        let dependency_index = addon_event_index(&load_events, dependency);
        assert!(
            dependency_index < root_index,
            "`{dependency}` must emit ADDON_LOADED before `{ROOT}`; events={load_events:?}"
        );
    }
}

fn addon_event_index(load_events: &[String], addon_name: &str) -> usize {
    load_events
        .iter()
        .position(|event_addon| event_addon == addon_name)
        .unwrap_or_else(|| panic!("expected ADDON_LOADED for `{addon_name}` in {load_events:?}"))
}
