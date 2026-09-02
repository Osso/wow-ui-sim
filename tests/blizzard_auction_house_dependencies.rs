use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AuctionHouseUI";
const COLORS: &str = "Blizzard_Colors";
const HELP_PLATE: &str = "Blizzard_HelpPlate";
const MANAGED_FRAME_SYSTEM: &str = "Blizzard_ManagedFrameSystem";

#[test]
fn auction_house_load_addon_loads_toc_dependencies_before_root() {
    let toc = TocFile::from_file(&auction_house_toc()).expect("AuctionHouse TOC should parse");
    assert_eq!(
        toc.dependencies(),
        [COLORS, HELP_PLATE, MANAGED_FRAME_SYSTEM],
        "`{ROOT}` must keep the TOC dependency order used by runtime LoadAddOn"
    );

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

fn auction_house_toc() -> std::path::PathBuf {
    blizzard_ui_dir()
        .join(ROOT)
        .join("Blizzard_AuctionHouseUI_Mainline.toc")
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
    let mut state = env.state().borrow_mut();
    for addon in &mut state.addons {
        if addon.folder_name == COLORS
            || addon.folder_name == HELP_PLATE
            || addon.folder_name == MANAGED_FRAME_SYSTEM
        {
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
        !colors_loaded && !help_plate_loaded && !managed_frame_system_loaded && !root_loaded,
        "`{COLORS}`, `{HELP_PLATE}`, `{MANAGED_FRAME_SYSTEM}`, and `{ROOT}` must start unloaded"
    );
}

fn assert_dependency_events_precede_root(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let load_events: Vec<String> = env
        .eval("return __auctionHouseDependencyLoadEvents")
        .expect("ADDON_LOADED dependency trace should be readable");

    let colors_index = addon_event_index(&load_events, COLORS);
    let help_plate_index = addon_event_index(&load_events, HELP_PLATE);
    let managed_frame_system_index = addon_event_index(&load_events, MANAGED_FRAME_SYSTEM);
    let root_index = addon_event_index(&load_events, ROOT);

    assert!(
        colors_index < root_index,
        "`{COLORS}` must emit ADDON_LOADED before `{ROOT}`; events={load_events:?}"
    );
    assert!(
        help_plate_index < root_index,
        "`{HELP_PLATE}` must emit ADDON_LOADED before `{ROOT}`; events={load_events:?}"
    );
    assert!(
        managed_frame_system_index < root_index,
        "`{MANAGED_FRAME_SYSTEM}` must emit ADDON_LOADED before `{ROOT}`; events={load_events:?}"
    );
}

fn addon_event_index(load_events: &[String], addon_name: &str) -> usize {
    load_events
        .iter()
        .position(|event_addon| event_addon == addon_name)
        .unwrap_or_else(|| panic!("expected ADDON_LOADED for `{addon_name}` in {load_events:?}"))
}
