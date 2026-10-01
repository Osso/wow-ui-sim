//! Current local AutoRoll TOC, not an identified archive or native-parity proof.
//! Raid contexts are configured-world Rust fixtures, not public raid-entry proof.
#![cfg(feature = "client-wowforever")]

use std::path::Path;

use wow_ui_sim::loader::{
    MissingRequirement, MissingRequirementKind,
    discover_blizzard_addon_closure_for_screen_with_overrides, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{NilSymbolAccess, NilSymbolEnvironment};
use wow_ui_sim::screen::ScreenKind;

// OptionPanel.lua consumes Settings registration and section-header initializers.
// Their definitions live in this cached root and its actual TOC dependency closure.
const BLIZZARD_ROOTS: &[&str] = &["Blizzard_Settings_Shared"];
const ITEM_NAME: &str = "AutoRoll Integration Sword";
const ITEM_TEXTURE: &str = "Interface\\Icons\\INV_Sword_04";

// Existing truthy C_Reveal stub changes DebugBarManager's debug-event setup,
// not a harmless nil probe. Reveal/native/public raid-entry/UI-scale/capture
// and no-op behavior remain unverified. Retire this exact exception when the
// underlying behavior/classification is corrected; no debug events are fired here.
fn is_known_sharedxml_reveal_setup(requirement: &MissingRequirement) -> bool {
    matches!(
        &requirement.kind,
        MissingRequirementKind::CNamespace { namespace } if namespace == "C_Reveal"
    ) && requirement.attribution.addon_name == "Blizzard_SharedXML"
        && requirement.attribution.environment == NilSymbolEnvironment::Public
        && requirement.attribution.source.as_deref() == Some("DebugBarManager.lua")
        && requirement.attribution.line == Some(106)
}

// Mainline captures the club autocomplete function without calling it; the base
// file eagerly calls the hardcore query only to select popup definitions.
// Diagnostic lines are loader attribution, not physical Mainline source lines.
fn is_known_static_popup_setup(requirement: &MissingRequirement) -> bool {
    let known_method = match (&requirement.kind, requirement.attribution.line) {
        (MissingRequirementKind::CMethod { namespace, method }, Some(1371)) => {
            namespace == "C_Club" && method == "GetInvitationCandidates"
        }
        (MissingRequirementKind::CMethod { namespace, method }, Some(3412)) => {
            namespace == "C_GameRules" && method == "IsHardcoreActive"
        }
        _ => false,
    };
    known_method
        && requirement.attribution.addon_name == "Blizzard_StaticPopup_Game"
        && requirement.attribution.environment == NilSymbolEnvironment::Public
        && requirement.attribution.source.as_deref() == Some("GameDialogDefs.lua")
}

fn load_checked_toc(env: &WowLuaEnv, toc: &Path, is_cached_dependency: bool) {
    let result = load_addon(&env.loader_env(), toc)
        .unwrap_or_else(|error| panic!("{} must load unchanged: {error}", toc.display()));
    assert!(
        result.warnings.is_empty(),
        "{}: {:?}",
        result.name,
        result.warnings
    );
    let mut unexpected_requirements = Vec::new();
    for requirement in &result.missing_requirements {
        if is_cached_dependency
            && result.name == "Blizzard_SharedXML"
            && is_known_sharedxml_reveal_setup(requirement)
        {
            eprintln!(
                "retained known debug-setup limitation (not native absence): {requirement:?}"
            );
        } else if is_cached_dependency
            && result.name == "Blizzard_StaticPopup_Game"
            && is_known_static_popup_setup(requirement)
        {
            eprintln!("retained unrelated popup-setup limitation: {requirement:?}");
        } else {
            unexpected_requirements.push(requirement);
        }
    }
    assert!(
        unexpected_requirements.is_empty(),
        "{}: unexpected dependency/local requirements: {unexpected_requirements:?}",
        result.name
    );
    assert!(
        result.lua_files + result.xml_files > 0,
        "empty TOC load: {result:?}"
    );
}

fn load_auto_roll_in_configured_world(
    instance_id: i32,
    instance_name: &str,
) -> (WowLuaEnv, Vec<NilSymbolAccess>) {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("Forever Blizzard cache must already exist; no downloads in this test");
    for root in BLIZZARD_ROOTS {
        assert!(ui.join(root).join(format!("{root}.toc")).is_file());
    }
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    assert_no_reveal_method_use(&env);
    // Same discovery and environment as the common closure harness, retaining
    // each LoadResult here so dependency warnings/requirements cannot be lost.
    let dependencies = discover_blizzard_addon_closure_for_screen_with_overrides(
        &ui,
        ScreenKind::Game,
        BLIZZARD_ROOTS,
        &[],
    );
    assert!(
        !dependencies.is_empty(),
        "Settings dependency closure must exist"
    );
    for (_, toc) in dependencies {
        load_checked_toc(&env, &toc, true);
    }
    assert_no_lua_errors(&env);
    let setup_requirements = c_requirement_history(&env);
    install_popup_call_observer(&env);
    assert_consumed_path_checkpoint(&env, &setup_requirements);
    env.exec("assert(AutoRollDB == nil and AutoRollFrame == nil)")
        .expect("no persisted AutoRoll globals may precede local TOC loading");
    {
        let mut state = env.state().borrow_mut();
        let world = &mut state.world;
        world.instance_id = instance_id;
        world.instance_name = instance_name.into();
        world.instance_type = "raid".into();
        world.instance_difficulty = 16;
        world.instance_difficulty_name = "Mythic".into();
        world.instance_max_players = 20;
        world.instance_group_size = 20;
        world.in_instance = true;
        world.instance_dynamic_difficulty = 0;
        world.instance_is_dynamic = false;
        world.instance_lfg_dungeon_id = None;
    }
    let addons = Path::new(env!("CARGO_MANIFEST_DIR")).join("Interface/AddOns");
    env.state()
        .borrow_mut()
        .addon_base_paths
        .push(addons.clone());
    load_checked_toc(&env, &addons.join("AutoRoll/AutoRoll.toc"), false);
    assert_consumed_path_checkpoint(&env, &setup_requirements);
    env.exec(
        r#"
        assert(AutoRollFrame:IsEventRegistered('ADDON_LOADED'))
        assert(not AutoRollFrame:IsEventRegistered('START_LOOT_ROLL'))
        assert(type(AutoRollFrame:GetScript('OnEvent')) == 'function')
        assert(type(SlashCmdList.AUTOROLL) == 'function' and SLASH_AUTOROLL1 == '/autoroll')
        "#,
    )
    .expect("whole TOC must publish its frame, handler and slash command before lifecycle");
    // The executable emits this after the TOC. AutoRoll has no login handler.
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("AutoRoll")])
        .expect("normal ADDON_LOADED must initialize AutoRoll Settings and events");
    assert_default_lifecycle(&env);
    assert_consumed_path_checkpoint(&env, &setup_requirements);
    let context: (String, String, i32, String, i32, i32, bool, i32) = env
        .eval("return GetInstanceInfo()")
        .expect("configured world must be visible through the real instance query");
    assert_eq!(
        context,
        (
            instance_name.into(),
            "raid".into(),
            16,
            "Mythic".into(),
            20,
            0,
            false,
            instance_id
        )
    );
    assert_consumed_path_checkpoint(&env, &setup_requirements);
    (env, setup_requirements)
}

// History catches new manufacture/lookup, not calls through cached functions.
// Keep the entire C-requirement history, including original setup attribution.
fn c_requirement_history(env: &WowLuaEnv) -> Vec<NilSymbolAccess> {
    env.state()
        .borrow()
        .nil_symbol_accesses
        .iter()
        .filter(|access| {
            access.container.starts_with("C_")
                || matches!(access.container.as_str(), "_G" | "__secureenv")
                    && access.key.starts_with("C_")
        })
        .cloned()
        .collect()
}

fn install_popup_call_observer(env: &WowLuaEnv) {
    let console_before = env.state().borrow().console_output.clone();
    env.exec(
        r#"
        assert(debug.gethook() == nil, 'fixture must not replace an existing hook')
        assert(AutoRollDB == nil and AutoRollFrame == nil)
        assert(A_Admin.GetLastLootRollChoice() == nil and #GetActiveLootRollIDs() == 0)
        local club = rawget(rawget(_G, 'C_Club'), 'GetInvitationCandidates')
        local hardcore = rawget(rawget(_G, 'C_GameRules'), 'IsHardcoreActive')
        assert(type(club) == 'function' and type(hardcore) == 'function')
        local invite = StaticPopupDialogs.INVITE_COMMUNITY_MEMBER
        local death = StaticPopupDialogs.HARDCORE_DEATH
        assert(invite.autoCompleteSource == club and death == nil)
        local counts = {[club] = 0, [hardcore] = 0}
        local witness = function() return 'observer witness' end
        local nativeWitness = math.abs
        counts[witness] = 0
        counts[nativeWitness] = 0
        local observer = function(event)
            if event == 'call' then
                local func = debug.getinfo(2, 'f').func
                if counts[func] ~= nil then counts[func] = counts[func] + 1 end
            end
        end
        debug.sethook(observer, 'c')
        -- Calibration precedes world/local TOC/lifecycle. Invoke the actual cached
        -- nil closures, not replacement functions or popup/vendor callbacks.
        local outerTaint = debug.getstacktaint()
        local calibrate = function()
            local before = debug.getstacktaint()
            assert(before == 'AutoRollFixtureCalibration')
            assert(invite.autoCompleteSource(nil, nil, nil, nil, 0) == nil)
            assert(hardcore() == nil)
            assert(witness() == 'observer witness' and nativeWitness(-7) == 7)
            assert(debug.getstacktaint() == before, 'observer changed caller taint')
        end
        debug.setobjecttaint(calibrate, 'AutoRollFixtureCalibration')
        calibrate()
        assert(debug.getstacktaint() == outerTaint, 'calibration taint escaped caller')
        assert(counts[club] == 1 and counts[hardcore] == 1
            and counts[witness] == 1 and counts[nativeWitness] == 1,
            'call hook must observe cached Lua and native functions by identity')
        __AutoRollFixtureAssertPopupIsolation = function()
            local hook, mask = debug.gethook()
            assert(hook == observer and mask == 'c', 'observer must stay installed')
            local before, nativeBefore = counts[witness], counts[nativeWitness]
            local taintBefore = debug.getstacktaint()
            assert(witness() == 'observer witness' and counts[witness] == before + 1)
            assert(nativeWitness(-7) == 7 and counts[nativeWitness] == nativeBefore + 1)
            assert(debug.getstacktaint() == taintBefore)
            assert(rawget(C_Club, 'GetInvitationCandidates') == club)
            assert(rawget(C_GameRules, 'IsHardcoreActive') == hardcore)
            assert(counts[club] == 1 and counts[hardcore] == 1,
                'excluded popup queries called after calibration')
            assert(StaticPopupDialogs.INVITE_COMMUNITY_MEMBER == invite)
            assert(invite.autoCompleteSource == club)
            assert(StaticPopupDialogs.HARDCORE_DEATH == death)
        end
        assert(AutoRollDB == nil and AutoRollFrame == nil)
        assert(A_Admin.GetLastLootRollChoice() == nil and #GetActiveLootRollIDs() == 0)
        "#,
    )
    .expect("supported VM call observer must calibrate without supplying AutoRoll outcomes");
    assert_eq!(env.state().borrow().console_output, console_before);
    assert_no_lua_errors(env);
}

fn assert_consumed_path_checkpoint(env: &WowLuaEnv, setup_requirements: &[NilSymbolAccess]) {
    env.exec("__AutoRollFixtureAssertPopupIsolation()")
        .expect("consumed path must leave excluded cached calls and popup fixtures untouched");
    assert_eq!(
        c_requirement_history(env),
        setup_requirements,
        "consumed path introduced or changed C requirement history"
    );
    assert_no_lua_errors(env);
}

fn assert_default_lifecycle(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(AutoRollDB.enabled == true and AutoRollDB.debugMode == false)
        assert(AutoRollDB.defaultRollMode == 2 and AutoRollDB.fallbackRollMode == 5)
        assert(AutoRollDB.version == 1 and AutoRollDB.autoCloseLootWindow == true)
        local enabled = {2296, 2450, 2481, 2522, 2569, 2549}
        local disabled = {2649, 2650, 2651}
        for _, id in ipairs(enabled) do assert(AutoRollDB.raidSettings[id] == true) end
        for _, id in ipairs(disabled) do assert(AutoRollDB.raidSettings[id] == false) end
        local count = 0
        for _ in pairs(AutoRollDB.raidSettings) do count = count + 1 end
        assert(count == 9)
        assert(AutoRollFrame:IsEventRegistered('ADDON_LOADED'))
        assert(AutoRollFrame:IsEventRegistered('START_LOOT_ROLL'))
        assert(Settings.GetSetting('AutoRoll_enabled'):GetValue() == true)
        assert(Settings.GetSetting('AutoRoll_defaultRollMode'):GetValue() == 2)
        assert(Settings.GetSetting('AutoRoll_raid_2522'):GetValue() == true)
        assert(Settings.GetSetting('AutoRoll_raid_2649'):GetValue() == false)
        assert(A_Admin.GetLastLootRollChoice() == nil)
        assert(#GetActiveLootRollIDs() == 0)
        "#,
    )
    .expect("clean defaults, real Settings publication and event registration must initialize");
    assert_no_lua_errors(env);
}

fn assert_no_reveal_method_use(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_Glue.IsOnGlueScreen() == false)
        assert(type(rawget(_G, '__wow_record_nil_symbol_access')) == 'function')
        assert(type(rawget(_G, '__wow_log_nil_symbol_access')) == 'function')
        local reveal = rawget(_G, 'C_Reveal')
        if reveal ~= nil then
            for _, value in pairs(reveal) do
                assert(type(value) ~= 'function', 'cached Reveal method must not supply behavior')
            end
        end
        "#,
    )
    .expect(
        "Game mode excludes glue debug watcher; Reveal inspection must not manufacture methods",
    );
    // Full history, never cleared/sliced: __index records first manufacture.
    // Together with the raw table scan this rejects every cached Reveal method;
    // history alone does not observe later reuse of an already cached function.
    let state = env.state().borrow();
    let method_accesses: Vec<_> = state
        .nil_symbol_accesses
        .iter()
        .filter(|access| access.container == "C_Reveal")
        .collect();
    assert!(
        method_accesses.is_empty(),
        "Reveal methods must not supply this workflow's behavior: {method_accesses:?}"
    );
}

fn assert_no_lua_errors(env: &WowLuaEnv) {
    assert_no_reveal_method_use(env);
    let state = env.state().borrow();
    assert!(state.lua_errors.is_empty(), "{:?}", state.lua_errors);
    assert!(
        state.lua_error_records.is_empty(),
        "{:?}",
        state.lua_error_records
    );
    assert!(
        state.lua_error_counts.is_empty(),
        "{:?}",
        state.lua_error_counts
    );
}

fn start_real_loot_roll(env: &WowLuaEnv, roll_id: i32) {
    assert!(env.state().borrow().world.loot_rolls.is_empty());
    assert_eq!(env.state().borrow().last_loot_roll_choice, None);
    let requirements_before = c_requirement_history(env);
    assert_consumed_path_checkpoint(env, &requirements_before);
    env.exec(&format!(
        "A_Admin.StartLootRoll({roll_id}, 30, {ITEM_NAME:?}, {ITEM_TEXTURE:?}, 4, 639)"
    ))
    .expect("existing producer must store the roll then synchronously dispatch START_LOOT_ROLL");
    assert_consumed_path_checkpoint(env, &requirements_before);
}

fn assert_greed_result(env: &WowLuaEnv, roll_id: i32) {
    let choice: Option<i32> = env.eval("return A_Admin.GetLastLootRollChoice()").unwrap();
    assert_eq!(choice, Some(2), "unchanged AutoRoll must choose Greed");
    let state = env.state().borrow();
    assert_eq!(state.last_loot_roll_choice, Some(2));
    assert!(!state.world.loot_rolls.contains_key(&roll_id));
    assert!(
        state
            .console_output
            .iter()
            .any(|line| { line.contains(ITEM_NAME) && line.contains("Greed") }),
        "real addon roll log missing: {:?}",
        state.console_output
    );
}

#[test]
fn forever_auto_roll_default_greed_consumes_enabled_vault_roll() {
    let (env, setup) = load_auto_roll_in_configured_world(2522, "Vault of the Incarnates");
    start_real_loot_roll(&env, 42);
    assert_greed_result(&env, 42);
    assert_consumed_path_checkpoint(&env, &setup);
}

#[test]
fn forever_auto_roll_unsupported_raid_leaves_produced_roll_untouched() {
    let (env, setup) = load_auto_roll_in_configured_world(1, "Unsupported Raid Fixture");
    start_real_loot_roll(&env, 43);
    assert_untouched_roll(&env, 43);
    assert_consumed_path_checkpoint(&env, &setup);
}

#[test]
fn forever_auto_roll_default_disabled_nerubar_leaves_produced_roll_untouched() {
    let (env, setup) = load_auto_roll_in_configured_world(2649, "Nerub-ar Palace");
    start_real_loot_roll(&env, 44);
    assert_untouched_roll(&env, 44);
    assert_consumed_path_checkpoint(&env, &setup);
}

fn assert_untouched_roll(env: &WowLuaEnv, roll_id: i32) {
    let choice: Option<i32> = env.eval("return A_Admin.GetLastLootRollChoice()").unwrap();
    assert_eq!(choice, None);
    let state = env.state().borrow();
    assert_eq!(state.last_loot_roll_choice, None);
    assert_eq!(state.world.loot_rolls.len(), 1);
    let roll = state
        .world
        .loot_rolls
        .get(&roll_id)
        .expect("producer roll must remain active");
    assert_eq!(roll.roll_id, roll_id);
    assert_eq!(roll.roll_time, 30.0);
    assert_eq!(roll.name, ITEM_NAME);
    assert_eq!(roll.texture, ITEM_TEXTURE);
    assert_eq!(roll.quality, 4);
    assert_eq!(roll.item_level, 639);
    assert_eq!(roll.count, 1);
    assert!(roll.can_greed, "producer's existing default allows Greed");
    assert!(
        !state
            .console_output
            .iter()
            .any(|line| line.contains(ITEM_NAME))
    );
}

#[test]
fn forever_auto_roll_environments_isolate_database_events_rolls_and_logs() {
    let (first, first_setup) = load_auto_roll_in_configured_world(2522, "Vault of the Incarnates");
    start_real_loot_roll(&first, 45);
    assert_greed_result(&first, 45);
    first
        .exec(
            r#"
        AutoRollDB.debugMode = true
        AutoRollDB.raidSettings[2522] = false
        AutoRollDB.isolationMarker = 'FIRST_ENV_ONLY'
        AutoRollFrame:UnregisterEvent('START_LOOT_ROLL')
        print('FIRST_ENV_ONLY')
        "#,
        )
        .expect("mutate only first environment's database, event registration and log");
    let first_log = first.state().borrow().console_output.clone();

    assert_consumed_path_checkpoint(&first, &first_setup);
    let (second, second_setup) =
        load_auto_roll_in_configured_world(2522, "Vault of the Incarnates");
    second
        .exec("assert(AutoRollDB.isolationMarker == nil)")
        .expect("saved-variable table must not cross environments");
    assert!(
        !second
            .state()
            .borrow()
            .console_output
            .iter()
            .any(|line| line.contains("FIRST_ENV_ONLY"))
    );
    start_real_loot_roll(&second, 46);
    assert_greed_result(&second, 46);

    first
        .exec(
            r#"
        assert(AutoRollDB.debugMode == true and AutoRollDB.raidSettings[2522] == false)
        assert(AutoRollDB.isolationMarker == 'FIRST_ENV_ONLY')
        assert(not AutoRollFrame:IsEventRegistered('START_LOOT_ROLL'))
        assert(A_Admin.GetLastLootRollChoice() == 2)
        "#,
        )
        .expect("second lifecycle/producer must not alter first environment");
    assert_eq!(first.state().borrow().console_output, first_log);
    assert!(first.state().borrow().world.loot_rolls.is_empty());
    assert_consumed_path_checkpoint(&first, &first_setup);
    assert_consumed_path_checkpoint(&second, &second_setup);
}
