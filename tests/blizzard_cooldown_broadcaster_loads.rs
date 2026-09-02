#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn cooldown_broadcaster_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster.toc")
}

fn load_game_ui_before_startup() -> WowLuaEnv {
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
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = load_game_ui_before_startup();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn expose_cooldown_broadcaster_frame(env: &WowLuaEnv) {
    let frame_found: bool = env
        .eval(
            r#"
            local object = EnumerateFrames()
            while object do
                if type(object.GetSupportedTrackedSpells) == "function"
                    and type(object.SendSpellCooldownMessage) == "function"
                    and type(object.GetOnlineGroupMemberGUIDs) == "function"
                    and type(object.OnLoad) == "function"
                then
                    __test_cooldown_broadcaster_frame = object
                    return true
                end
                object = EnumerateFrames(object)
            end
            return false
            "#,
        )
        .expect("anonymous cooldown broadcaster frame query should succeed");
    assert!(
        frame_found,
        "Blizzard_CooldownBroadcaster should create its private relay frame"
    );
}

fn load_cooldown_broadcaster() -> WowLuaEnv {
    let env = load_full_game_ui();
    load_addon(&env.loader_env(), &cooldown_broadcaster_toc())
        .expect("Blizzard_CooldownBroadcaster should load via Rust loader");
    expose_cooldown_broadcaster_frame(&env);
    env
}

fn load_cooldown_broadcaster_before_startup() -> WowLuaEnv {
    let env = load_game_ui_before_startup();
    load_addon(&env.loader_env(), &cooldown_broadcaster_toc())
        .expect("Blizzard_CooldownBroadcaster should load via Rust loader");
    expose_cooldown_broadcaster_frame(&env);
    env
}

#[test]
fn blizzard_cooldown_broadcaster_toc_is_lod_without_game_type_restriction() {
    let toc = TocFile::from_file(&cooldown_broadcaster_toc())
        .expect("Blizzard_CooldownBroadcaster TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_CooldownBroadcaster declares `## LoadOnDemand: 1`"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_CooldownBroadcaster does not declare AllowLoadGameType"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_CooldownBroadcaster does not declare UseSecureEnvironment"
    );
}

#[test]
fn blizzard_cooldown_broadcaster_is_promoted_before_game_startup() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let broadcaster_position = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_CooldownBroadcaster")
        .expect("Blizzard_CooldownBroadcaster should be promoted into game discovery");
    let game_position = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_Game")
        .expect("Blizzard_Game should appear in game discovery");

    assert!(
        broadcaster_position < game_position,
        "Blizzard_CooldownBroadcaster must load before Blizzard_Game because the current game \
         startup publisher requires it"
    );
}

prefork_full_ui_case! {
fn blizzard_cooldown_broadcaster_loads_via_load_addon_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &cooldown_broadcaster_toc())
        .expect("Blizzard_CooldownBroadcaster should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_CooldownBroadcaster emitted Lua errors during explicit load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

#[test]
fn blizzard_cooldown_broadcaster_private_frame_carries_current_mixin_methods() {
    let env = load_cooldown_broadcaster();

    let frame_present: bool = env
        .eval(
            "local f = __test_cooldown_broadcaster_frame; \
             return rawget(_G, 'CooldownBroadcasterFrame') == nil \
                and type(f.GetSupportedTrackedSpells) == 'function' \
                and type(f.GetChannel) == 'function' \
                and type(f.SendComm) == 'function' \
                and type(f.SendSpellInfoMessage) == 'function' \
                and type(f.RefreshSpellData) == 'function' \
                and type(f.HasTrackedSpellsChanged) == 'function' \
                and type(f.GetSpellCooldown) == 'function' \
                and type(f.EnableSync) == 'function' \
                and type(f.DisableSync) == 'function' \
                and type(f.ShouldSyncBeEnabled) == 'function' \
                and type(f.GetOnlineGroupMemberGUIDs) == 'function' \
                and type(f.UpdateSyncState) == 'function' \
                and type(f.BuildSpellCooldownRows) == 'function' \
                and type(f.SendSpellCooldownMessage) == 'function' \
                and type(f.PLAYER_ENTERING_WORLD) == 'function' \
                and type(f.GROUP_ROSTER_UPDATE) == 'function' \
                and type(f.UNIT_CONNECTION) == 'function' \
                and type(f.SPELLS_CHANGED) == 'function' \
                and type(f.SPELL_UPDATE_COOLDOWN) == 'function' \
                and type(f.OnLoad) == 'function'",
        )
        .expect("private cooldown broadcaster frame method query should succeed");
    assert!(
        frame_present,
        "Current Blizzard_CooldownBroadcaster keeps its relay frame private and mixes in the \
         current spell-info, cooldown-message, group-member, event, and lifecycle methods"
    );
}

#[test]
fn blizzard_cooldown_broadcaster_state_is_initialized_by_on_load() {
    let env = load_cooldown_broadcaster_before_startup();

    let state_initialized: bool = env
        .eval(
            "local f = __test_cooldown_broadcaster_frame; \
             return f.playerSpecID == nil \
                and type(f.spellData) == 'table' and next(f.spellData) == nil \
                and type(f.spellOrder) == 'table' and next(f.spellOrder) == nil \
                and f.syncEnabled == false \
                and type(f.lastGroupGUIDs) == 'table' and next(f.lastGroupGUIDs) == nil",
        )
        .expect("state init query should succeed");
    assert!(
        state_initialized,
        "OnLoad should initialize empty spellData, spellOrder, and lastGroupGUIDs tables, a nil \
         playerSpecID, and syncEnabled=false before startup events mutate relay state"
    );
}

#[test]
fn blizzard_cooldown_broadcaster_get_channel_tracks_group_type() {
    let env = load_cooldown_broadcaster();

    let channels_are_correct: bool = env
        .eval(
            "local f = __test_cooldown_broadcaster_frame; \
             IsInRaid = function() return false end; \
             IsInGroup = function() return false end; \
             local solo = f:GetChannel() == nil and f:ShouldSyncBeEnabled() == false; \
             IsInGroup = function(category) \
               return category == nil or category == LE_PARTY_CATEGORY_INSTANCE \
             end; \
             local instance = f:GetChannel() == 'INSTANCE_CHAT' \
               and f:ShouldSyncBeEnabled() == true; \
             IsInRaid = function() return true end; \
             IsInGroup = function() return true end; \
             local raid = f:GetChannel() == 'RAID'; \
             return solo and instance and raid",
        )
        .expect("GetChannel group-state query should succeed");
    assert!(
        channels_are_correct,
        "GetChannel should return nil while solo, INSTANCE_CHAT for an instance-only group, and \
         RAID when IsInRaid is true; ShouldSyncBeEnabled should track IsInGroup"
    );
}

#[test]
fn blizzard_cooldown_broadcaster_outlaw_spells_include_racial_and_cap_at_six() {
    let env = load_cooldown_broadcaster();

    let outlaw_order: bool = env
        .eval(
            "local f = __test_cooldown_broadcaster_frame; \
             PlayerUtil.GetCurrentSpecID = function() return 260 end; \
             UnitRace = function() return 'Orc', 'Orc' end; \
             UnitClass = function() return 'Warrior', 'WARRIOR' end; \
             C_SpellBook.IsSpellKnown = function() return true end; \
             C_SpellBook.FindBaseSpellByID = function(id) return id end; \
             C_SpellBook.FindSpellOverrideByID = function() return nil end; \
             local spells, specID = f:GetSupportedTrackedSpells(); \
             f:RefreshSpellData(spells, specID); \
             local o = f.spellOrder; \
             return specID == 260 and #o == 6 \
                and o[1] == 20572 and o[2] == 13750 and o[3] == 51690 \
                and o[4] == 31224 and o[5] == 5277 and o[6] == 1856",
        )
        .expect("Outlaw tracked-spell query should succeed");
    assert!(
        outlaw_order,
        "Current Outlaw tracking should prepend the known Orc racial, retain the first five \
         spec cooldowns, and cap spellOrder at MAX_COOLDOWNS=6"
    );
}

#[test]
fn blizzard_cooldown_broadcaster_warrior_specs_include_base_and_interrupt_orders() {
    let env = load_cooldown_broadcaster();

    let warrior_orders: bool = env
        .eval(
            "local f = __test_cooldown_broadcaster_frame; \
             local currentSpecID; \
             PlayerUtil.GetCurrentSpecID = function() return currentSpecID end; \
             UnitRace = function() return 'Orc', 'Orc' end; \
             UnitClass = function() return 'Warrior', 'WARRIOR' end; \
             C_SpellBook.IsSpellKnown = function() return true end; \
             C_SpellBook.IsSpellKnownOrInSpellBook = function() return true end; \
             C_SpellBook.FindBaseSpellByID = function(id) return id end; \
             C_SpellBook.FindSpellOverrideByID = function() return nil end; \
             local function order_for(specID) \
               currentSpecID = specID; \
               return f:GetSupportedTrackedSpells(); \
             end; \
             local arms = order_for(71); \
             local fury = order_for(72); \
             local prot = order_for(73); \
             return #arms == 7 \
                and arms[1] == 20572 and arms[2] == 107574 and arms[3] == 97462 \
                and arms[4] == 118038 and arms[5] == 46968 and arms[6] == 228920 \
                and arms[7] == 6552 \
                and #fury == 7 \
                and fury[1] == 20572 and fury[2] == 107574 and fury[3] == 227847 \
                and fury[4] == 97462 and fury[5] == 184364 and fury[6] == 46968 \
                and fury[7] == 6552 \
                and #prot == 8 \
                and prot[1] == 20572 and prot[2] == 107574 and prot[3] == 871 \
                and prot[4] == 97462 and prot[5] == 46968 and prot[6] == 1160 \
                and prot[7] == 6552 and prot[8] == 386071",
        )
        .expect("Warrior tracked-spell query should succeed");
    assert!(
        warrior_orders,
        "Current Arms, Fury, and Protection tracking should include six base slots with the \
         racial plus up to two interrupt orders"
    );
}
