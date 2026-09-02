#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn durability_frame_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DurabilityFrame/Blizzard_DurabilityFrame.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

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

#[test]
fn blizzard_durability_frame_toc_declares_three_dependencies_and_default_enabled() {
    let toc = TocFile::from_file(&durability_frame_toc())
        .expect("Blizzard_DurabilityFrame TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DurabilityFrame omits `## LoadOnDemand` — auto-loads on Game (default per \
         src/toc.rs:311). The frame is part of the live HUD, not on-demand"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DurabilityFrame does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_DurabilityFrame declares no `## AllowLoadGameType:` filter — durability \
         alerts apply to every game type"
    );

    let deps: Vec<String> = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_Minimap".to_string(),
            "Blizzard_EditMode".to_string(),
            "Blizzard_ManagedFrameSystem".to_string(),
        ],
        "Retail 12.1.0.69497 declares Minimap, EditMode, and ManagedFrameSystem in this \
         order for the durability HUD. Got: {deps:?}"
    );

    let toc_text = std::fs::read_to_string(durability_frame_toc())
        .expect("Blizzard_DurabilityFrame TOC should read");
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_DurabilityFrame must declare `## DefaultState: enabled` — the user can \
         disable the addon via the AddOns panel, but it ships enabled by default"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DurabilityFrame omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311). Durability alerts have no meaning on the glue/login screen"
    );
}

#[test]
fn blizzard_durability_frame_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DurabilityFrame");
    assert!(
        in_game,
        "Blizzard_DurabilityFrame (no AllowLoad flag) should appear in Game-screen \
         auto-discovery — it is a live in-game HUD frame"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DurabilityFrame");
    assert!(
        !in_login,
        "Blizzard_DurabilityFrame must NOT appear on Login / glue screens — durability \
         alerts only fire while the player is logged into a character"
    );
}

#[test]
fn blizzard_durability_frame_loads_after_its_two_declared_dependencies() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let durability_idx = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_DurabilityFrame");
    let minimap_idx = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_Minimap");
    let editmode_idx = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_EditMode");

    let durability_idx = durability_idx.expect("Blizzard_DurabilityFrame should be discovered");
    let minimap_idx = minimap_idx.expect("Blizzard_Minimap should be discovered");
    let editmode_idx = editmode_idx.expect("Blizzard_EditMode should be discovered");
    assert!(
        minimap_idx < durability_idx && editmode_idx < durability_idx,
        "Blizzard_DurabilityFrame must come AFTER both declared dependencies. \
         topological_sort_addons must place Minimap (idx={minimap_idx}) and EditMode \
         (idx={editmode_idx}) before DurabilityFrame (idx={durability_idx}) — otherwise \
         EditModeDurabilityFrameSystemTemplate (defined in Blizzard_EditMode) would be \
         missing when the XML tries to inherit it"
    );
}

prefork_full_ui_case! {
fn blizzard_durability_frame_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("DurabilityFrame") || message.contains("Durability"))
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DurabilityFrame emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_inventory_alert_status_slots_has_eleven_entries(env: &WowLuaEnv) {

    let count: i64 = env
        .eval("return #INVENTORY_ALERT_STATUS_SLOTS")
        .expect("#INVENTORY_ALERT_STATUS_SLOTS query should succeed");
    assert_eq!(
        count, 11,
        "DurabilityFrame.lua:1-12 declares INVENTORY_ALERT_STATUS_SLOTS with exactly 11 \
         indexed entries: Head, Shoulders, Chest, Waist, Legs, Feet, Wrists, Hands, Weapon, \
         Shield, Ranged. The last three (Weapon/Shield/Ranged) carry `showSeparate = 1`. \
         Got: {count}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_inventory_alert_status_slots_carries_correct_slot_names(env: &WowLuaEnv) {

    let slots: (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            "return INVENTORY_ALERT_STATUS_SLOTS[1].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[2].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[3].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[4].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[5].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[6].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[7].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[8].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[9].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[10].slot, \
                    INVENTORY_ALERT_STATUS_SLOTS[11].slot",
        )
        .expect("INVENTORY_ALERT_STATUS_SLOTS slot-name query should succeed");
    let expected = (
        "Head".to_string(),
        "Shoulders".to_string(),
        "Chest".to_string(),
        "Waist".to_string(),
        "Legs".to_string(),
        "Feet".to_string(),
        "Wrists".to_string(),
        "Hands".to_string(),
        "Weapon".to_string(),
        "Shield".to_string(),
        "Ranged".to_string(),
    );
    assert_eq!(
        slots, expected,
        "INVENTORY_ALERT_STATUS_SLOTS slot names must match the order at \
         DurabilityFrame.lua:2-12 exactly. The mixin loop at SetAlerts() relies on \
         _G[\"Durability\"..value.slot] to find the matching XML-named texture, so any \
         drift here breaks every alert visual"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_inventory_alert_status_slots_marks_separate_weapons(env: &WowLuaEnv) {

    let separate_flags: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = env
        .eval(
            "return INVENTORY_ALERT_STATUS_SLOTS[1].showSeparate, \
                    INVENTORY_ALERT_STATUS_SLOTS[9].showSeparate, \
                    INVENTORY_ALERT_STATUS_SLOTS[10].showSeparate, \
                    INVENTORY_ALERT_STATUS_SLOTS[11].showSeparate",
        )
        .expect("showSeparate flags query should succeed");
    assert_eq!(
        separate_flags,
        (None, Some(1), Some(1), Some(1)),
        "Only Weapon (idx 9), Shield (idx 10), and Ranged (idx 11) carry \
         `showSeparate = 1` (DurabilityFrame.lua:10-12). Armor slots (Head=1) have no \
         showSeparate field — the mixin's SetAlerts loop hides/shows them as a group based \
         on `showDurability`, while the separate slots are managed individually"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_inventory_alert_colors_yellow_then_red(env: &WowLuaEnv) {

    let colors: (f64, f64, f64, f64, f64, f64) = env
        .eval(
            "return INVENTORY_ALERT_COLORS[1].r, INVENTORY_ALERT_COLORS[1].g, \
                    INVENTORY_ALERT_COLORS[1].b, INVENTORY_ALERT_COLORS[2].r, \
                    INVENTORY_ALERT_COLORS[2].g, INVENTORY_ALERT_COLORS[2].b",
        )
        .expect("INVENTORY_ALERT_COLORS rgb query should succeed");
    let (r1, g1, b1, r2, g2, b2) = colors;
    assert!(
        (r1 - 1.0).abs() < 1e-6
            && (g1 - 0.82).abs() < 1e-6
            && (b1 - 0.18).abs() < 1e-6
            && (r2 - 0.93).abs() < 1e-6
            && (g2 - 0.07).abs() < 1e-6
            && (b2 - 0.07).abs() < 1e-6,
        "INVENTORY_ALERT_COLORS must encode two states: index 1 = yellow warning \
         (1.0, 0.82, 0.18), index 2 = red broken (0.93, 0.07, 0.07). These are sourced from \
         DurabilityFrame.lua:15-16 and indexed by the return value of \
         GetInventoryAlertStatus. Got: ({r1}, {g1}, {b1}) / ({r2}, {g2}, {b2})"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_mixin_table_exposes_lifecycle_methods(env: &WowLuaEnv) {

    let kinds: (String, String, String, String, String, String) = env
        .eval(
            "return type(DurabilityFrameMixin), \
                    type(DurabilityFrameMixin.OnLoad), \
                    type(DurabilityFrameMixin.OnEvent), \
                    type(DurabilityFrameMixin.OnEnter), \
                    type(DurabilityFrameMixin.OnLeave), \
                    type(DurabilityFrameMixin.SetAlerts)",
        )
        .expect("DurabilityFrameMixin method-types probe should succeed");
    let expected = (
        "table".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
    );
    assert_eq!(
        kinds, expected,
        "DurabilityFrameMixin (DurabilityFrame.lua:18) must be a table with five lifecycle / \
         logic methods: OnLoad (registers UPDATE_INVENTORY_ALERTS + PLAYER_ENTERING_WORLD), \
         OnEvent, OnEnter (tooltip), OnLeave (tooltip hide), SetAlerts (the main per-event \
         layout function reading GetInventoryAlertStatus)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_named_global_frame_exists_with_uiparent_parent(env: &WowLuaEnv) {

    let (kind, name): (String, String) = env
        .eval(
            "return type(DurabilityFrame), \
                    (type(DurabilityFrame) == 'table') and DurabilityFrame:GetName() or ''",
        )
        .expect("DurabilityFrame existence/name probe should succeed");
    assert_eq!(
        kind, "table",
        "DurabilityFrame.xml:3 declares `<Frame name=\"DurabilityFrame\" parent=\"UIParent\">` \
         — the XML loader must register this as a global addressable as a table"
    );
    assert_eq!(
        name, "DurabilityFrame",
        "DurabilityFrame:GetName() must round-trip the XML `name` attribute. The mixin \
         relies on `_G[\"Durability\"..value.slot]` to find child textures, and other \
         Blizzard code addresses the parent frame by global name"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_xml_creates_twelve_named_durability_textures(env: &WowLuaEnv) {

    let all_present: bool = env
        .eval(
            "return type(DurabilityHead) == 'table' \
                and type(DurabilityShoulders) == 'table' \
                and type(DurabilityChest) == 'table' \
                and type(DurabilityWrists) == 'table' \
                and type(DurabilityHands) == 'table' \
                and type(DurabilityWaist) == 'table' \
                and type(DurabilityLegs) == 'table' \
                and type(DurabilityFeet) == 'table' \
                and type(DurabilityWeapon) == 'table' \
                and type(DurabilityShield) == 'table' \
                and type(DurabilityOffWeapon) == 'table' \
                and type(DurabilityRanged) == 'table'",
        )
        .expect("Durability* texture globals probe should succeed");
    assert!(
        all_present,
        "DurabilityFrame.xml's <Layers><Layer level=\"BACKGROUND\"> block declares 12 \
         named textures (lines 10-93): DurabilityHead, DurabilityShoulders, DurabilityChest, \
         DurabilityWrists, DurabilityHands, DurabilityWaist, DurabilityLegs, DurabilityFeet, \
         DurabilityWeapon, DurabilityShield, DurabilityOffWeapon, DurabilityRanged. The \
         mixin's SetAlerts loop reads `_G[\"Durability\"..value.slot]` so each must be a \
         globally addressable table. Note 12 textures vs 11 SLOTS entries — \
         DurabilityOffWeapon is shown in dual-wield via the Shield-slot branch \
         (DurabilityFrame.lua:64-71), not by an INVENTORY_ALERT_STATUS_SLOTS entry"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_off_weapon_starts_hidden_per_xml(env: &WowLuaEnv) {

    let off_weapon_hidden: bool = env
        .eval("return DurabilityOffWeapon:IsShown() == false")
        .expect("DurabilityOffWeapon visibility probe should succeed");
    assert!(
        off_weapon_hidden,
        "DurabilityOffWeapon must start HIDDEN — DurabilityFrame.xml:80 declares the \
         texture with `hidden=\"true\"`. The mixin's SetAlerts swaps DurabilityShield ↔ \
         DurabilityOffWeapon based on C_PaperDollInfo.OffhandHasWeapon(): when the player \
         dual-wields, OffWeapon is shown and Shield hidden. Initial state per XML must be \
         hidden because the default char isn't dual-wielding at load time"
    );
}
}

prefork_full_ui_case! {
fn blizzard_durability_frame_registers_two_inventory_events_in_onload(env: &WowLuaEnv) {

    let registered: bool = env
        .eval(
            "return DurabilityFrame:IsEventRegistered('UPDATE_INVENTORY_ALERTS') \
                and DurabilityFrame:IsEventRegistered('PLAYER_ENTERING_WORLD')",
        )
        .expect("DurabilityFrame event-registration probe should succeed");
    assert!(
        registered,
        "DurabilityFrameMixin:OnLoad (DurabilityFrame.lua:20-24) must register two events: \
         UPDATE_INVENTORY_ALERTS (the durability state change signal) and \
         PLAYER_ENTERING_WORLD (zone-in refresh). The XML declares \
         `<OnLoad method=\"OnLoad\" inherit=\"prepend\"/>` so the inherited \
         EditModeDurabilityFrameSystemTemplate's OnLoad runs FIRST, then the mixin's runs \
         second — both event registrations must end up on the frame regardless of the \
         inherit chain order"
    );
}
}

#[test]
fn blizzard_durability_frame_directory_ships_one_lua_one_xml() {
    let dir = blizzard_ui_dir().join("Blizzard_DurabilityFrame");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DurabilityFrame dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let lua_count = entries.iter().filter(|n| n.ends_with(".lua")).count();
    let xml_count = entries.iter().filter(|n| n.ends_with(".xml")).count();
    assert_eq!(
        (lua_count, xml_count),
        (1, 1),
        "Blizzard_DurabilityFrame ships exactly 1 Lua + 1 XML — `DurabilityFrame.lua` \
         (mixin + alert constants) and `DurabilityFrame.xml` (frame + texture layout). \
         Got entries: {entries:?}"
    );
    assert!(
        entries.iter().any(|n| n == "DurabilityFrame.lua")
            && entries.iter().any(|n| n == "DurabilityFrame.xml"),
        "The Lua/XML pair must be named `DurabilityFrame.{{lua,xml}}` — note this differs \
         from the addon folder name `Blizzard_DurabilityFrame` (no `Blizzard_` prefix on \
         the inner files). Got entries: {entries:?}"
    );
}
