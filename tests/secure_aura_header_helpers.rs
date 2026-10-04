#![cfg(feature = "retail-12-0-5")]

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon, load_addon_from_toc};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn retail_cache() -> PathBuf {
    dirs::home_dir()
        .expect("home directory for retail vendor fixtures")
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns")
}

fn load_game_ui(include_aura_header: bool, spec_index: i32) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create vendor-path environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![retail_cache()];
        state.player.class_index = 2;
        state.player.active_spec_index = spec_index;
        let spec_id = match spec_index {
            1 => 65,
            2 => 66,
            3 => 70,
            _ => panic!("invalid Paladin fixture specialization"),
        };
        state.talents.switch_to_spec(spec_id);
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&retail_cache(), ScreenKind::Game) {
        if include_aura_header && name == "Blizzard_RestrictedAddOnEnvironment" {
            load_restricted_environment_with_aura_header(&env, &toc);
        } else {
            load_addon(&env.loader_env(), &toc)
                .unwrap_or_else(|error| panic!("load cached {name}: {error}"));
        }
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn load_restricted_environment_with_aura_header(env: &WowLuaEnv, toc: &std::path::Path) {
    // Extend the real TOC before its first load. Reloading the same folder
    // after startup is a silent already-loaded no-op, regardless of Title.
    let source = std::fs::read_to_string(toc).expect("read restricted environment TOC");
    let source = format!("{source}\nSecureAuraHeader.lua\nSecureAuraHeader.xml\n");
    let probe_toc = TocFile::parse(toc.parent().expect("TOC directory"), &source);
    let loaded = load_addon_from_toc(&env.loader_env(), &probe_toc)
        .expect("load actual secure aura files with restricted environment");
    assert!(loaded.lua_files > 0 && loaded.xml_files > 0);
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

fn aura(id: i32, duration: f64, expiration_time: f64) -> AuraInfo {
    AuraInfo {
        name: format!("Header aura {id}"),
        spell_id: 98000 + id,
        icon: 134973,
        duration,
        expiration_time,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

fn load_aura_header() -> WowLuaEnv {
    let env = load_game_ui(true, 3);
    // Isolate existing whole-UI startup errors, never errors from the probe.
    env.state().borrow_mut().lua_errors.clear();
    env.exec(
        r#"
        assert(type(SecureAuraHeader_Update) == 'function', 'cached aura Lua must load')
        HeaderProbe = CreateFrame('Frame', 'HeaderProbe', UIParent, 'SecureAuraHeaderTemplate')
        assert(type(HeaderProbe:GetScript('OnShow')) == 'function', 'cached aura XML must load')
        assert(not HeaderProbe:IsShown(), 'header template starts hidden')
        HeaderProbe:SetAttribute('unit', 'player')
        HeaderProbe:SetAttribute('filter', 'HELPFUL')
        HeaderProbe:SetAttribute('sortMethod', 'TIME')
        HeaderProbe:SetAttribute('sortDirection', '+')
        HeaderProbe:SetAttribute('separateOwn', 0)
        HeaderProbe:SetAttribute('template', 'SecureAuraButtonTemplate')
        HeaderProbe:SetAttribute('point', 'TOPLEFT')
        HeaderProbe:SetAttribute('xOffset', 20)
        HeaderProbe:SetAttribute('minWidth', 100)
        HeaderProbe:SetAttribute('minHeight', 20)
        HeaderProbe:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 0, 0)
        function HeaderOrder()
            local result = {}
            local position = 1
            while true do
                local child = HeaderProbe:GetAttribute('child' .. position)
                if not child then break end
                if child:IsShown() then
                    local index = child:GetAttribute('index')
                    assert(child:GetID() == index)
                    local record = C_UnitAuras.GetAuraDataByIndex('player', index, child:GetAttribute('filter'))
                    assert(record, 'child must reference a live aura')
                    result[#result + 1] = record.auraInstanceID
                end
                position = position + 1
            end
            return table.concat(result, ',')
        end
        "#,
    )
    .expect("construct real cached header template");
    env
}

fn header_order(env: &WowLuaEnv) -> String {
    env.eval("return HeaderOrder()")
        .expect("read observable header child order")
}

fn assert_no_lua_errors(env: &WowLuaEnv) {
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "vendor-path Lua errors: {errors:?}");
}

#[test]
fn cached_secure_aura_header_sorts_finite_before_permanent() {
    let env = load_aura_header();
    let now: f64 = env.eval("return GetTime()").expect("current aura clock");
    let permanent = aura(301, 0.0, 0.0);
    let long = aura(302, 3600.0, now + 3600.0);
    let short = aura(303, 30.0, now + 30.0);
    let second_permanent = aura(304, 0.0, 0.0);
    env.state().borrow_mut().player.buffs = vec![
        permanent.clone(),
        long.clone(),
        short.clone(),
        second_permanent.clone(),
    ];
    env.exec("HeaderProbe:Show(); assert(HeaderProbe:IsVisible()); assert(HeaderProbe:IsEventRegistered('UNIT_AURA'))")
        .expect("real template OnShow updates aura children");
    let first_order = header_order(&env);
    assert!(
        first_order == "303,302,301,304" || first_order == "303,302,304,301",
        "TIME ascending must place both permanents last: {first_order}"
    );

    env.state().borrow_mut().player.buffs = vec![short, second_permanent, long, permanent];
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("real template OnEvent updates a shuffled aura stream");
    let shuffled_order = header_order(&env);
    assert!(
        shuffled_order == "303,302,301,304" || shuffled_order == "303,302,304,301",
        "ordering must not depend on producer insertion order: {shuffled_order}"
    );
    assert_no_lua_errors(&env);
}

#[test]
fn cached_secure_aura_header_reorders_and_hides_stale_children() {
    let env = load_aura_header();
    let now: f64 = env.eval("return GetTime()").expect("current aura clock");
    env.state().borrow_mut().player.buffs = vec![
        aura(311, 0.0, 0.0),
        aura(312, 3600.0, now + 3600.0),
        aura(313, 30.0, now + 30.0),
    ];
    env.exec("HeaderProbe:Show(); assert(HeaderProbe:IsVisible()); assert(HeaderProbe:IsEventRegistered('UNIT_AURA'))")
        .expect("populate real header children");
    assert_eq!(header_order(&env), "313,312,311");
    env.exec("OriginalFirstChild = HeaderProbe:GetAttribute('child1')")
        .expect("retain original child identity");

    {
        let mut state = env.state().borrow_mut();
        state.player.buffs[2].duration = 0.0;
        state.player.buffs[2].expiration_time = 0.0;
    }
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("finite-to-permanent update");
    let order = header_order(&env);
    assert!(order == "312,311,313" || order == "312,313,311", "{order}");
    env.exec("assert(HeaderProbe:GetAttribute('child1') == OriginalFirstChild)")
        .expect("vendor header reuses its positional child");

    env.state().borrow_mut().player.buffs.remove(0);
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("aura removal update");
    assert_eq!(header_order(&env), "312,313");
    env.exec("assert(not HeaderProbe:GetAttribute('child3'):IsShown())")
        .expect("removed aura leaves no stale shown child");
    env.state()
        .borrow_mut()
        .player
        .buffs
        .push(aura(311, 20.0, now + 20.0));
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("aura readdition update");
    assert_eq!(header_order(&env), "311,312,313");
    assert_no_lua_errors(&env);
}


fn load_helper_ui() -> WowLuaEnv {
    load_helper_ui_for_spec(3)
}

fn load_helper_ui_for_spec(spec_index: i32) -> WowLuaEnv {
    let env = load_game_ui(false, spec_index);
    let toc = retail_cache().join("Blizzard_PlayerSpells/Blizzard_PlayerSpells.toc");
    load_addon(&env.loader_env(), &toc).expect("load real PlayerSpells UI");
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("Blizzard_PlayerSpells")])
        .expect("initialize real talent saved-variable lifecycle");
    env.exec(
        r#"
        assert(ClassTalentHelper and PlayerSpellsFrame)
        assert(PlayerCastingBarFrame and OverlayPlayerCastingBarFrame)
        function FrameFields(frame)
            local environment = debug.getfenv(frame)
            assert(environment and type(environment[1]) == 'table', 'real frame backing table')
            return environment[1]
        end
        function AssertCleanField(frame, field)
            local clean, owner = issecurevariable(FrameFields(frame), field)
            assert(clean, field .. ' tainted by ' .. tostring(owner))
        end
        function RunAddonHelper(helper, argument)
            local function addon()
                assert(not issecure(), 'actual addon caller taint')
                assert(debug.getstacktaint() == 'HeaderHelperAudit')
                helper(argument)
                assert(not issecure(), 'producer must not launder caller')
                assert(debug.getstacktaint() == 'HeaderHelperAudit', 'caller owner restored')
            end
            debug.setobjecttaint(addon, 'HeaderHelperAudit')
            assert(issecure(), 'secure outer context before addon')
            addon()
            assert(issecure(), 'secure outer context after addon')
        end
        -- Observe the real frame fields table, not an unconditional success on
        -- an unsupported value. Prime a slot before testing a tainted overwrite.
        local control = CreateFrame('Frame')
        control.probe = false
        local function dirtyControl() control.probe = true end
        debug.setobjecttaint(dirtyControl, 'HeaderHelperAudit')
        dirtyControl()
        local clean, owner = issecurevariable(FrameFields(control), 'probe')
        assert(not clean and owner == 'HeaderHelperAudit', 'field-taint negative control')
        "#,
    )
    .expect("prepare real helper calls and frame-field taint observations");
    env.state().borrow_mut().lua_errors.clear();
    env
}

#[test]
fn cached_class_talent_spec_helpers_keep_real_ui_and_castbar_fields_clean() {
    for helper in [
        "SwitchToSpecializationByIndex",
        "SwitchToSpecializationByName",
    ] {
        let env = load_helper_ui();
        env.exec(
            r#"
            PlayerSpellsFrame:SetTab(PlayerSpellsFrame.specTabID)
            PlayerSpellsFrame:Show()
            assert(PlayerSpellsFrame.SpecFrame:IsVisible())
            assert(PlayerSpellsFrame.SpecFrame.isInitialized)
            assert(not PlayerSpellsFrame.SpecFrame:IsActivateInProgress())
            AssertCleanField(PlayerSpellsFrame.SpecFrame, 'activatedSpecIndex')
            AssertCleanField(PlayerCastingBarFrame, 'showCastbar')
            AssertCleanField(OverlayPlayerCastingBarFrame, 'overrideBarType')
            "#,
        )
        .expect("initialize actual visible specialization UI");
        let argument = if helper.ends_with("ByIndex") {
            "2"
        } else {
            "'Protection'"
        };
        env.exec(&format!(
            "RunAddonHelper(ClassTalentHelper.{helper}, {argument})"
        ))
        .expect("addon enters cached helper, real callback and real UI methods");
        env.exec(
            r#"
            assert(PlayerSpellsFrame.SpecFrame.activatedSpecIndex == 2, 'real spec activation UI was driven')
            assert(OverlayPlayerCastingBarFrame:GetParent() == PlayerSpellsFrame.SpecFrame.DisabledOverlay)
            assert(OverlayPlayerCastingBarFrame.overrideBarType == CastingBarType.ApplyingTalents)
            assert(PlayerCastingBarFrame.showCastbar == false, 'real player castbar was changed')
            assert(OverlayPlayerCastingBarFrame.showCastbar == true)
            AssertCleanField(PlayerSpellsFrame.SpecFrame, 'activatedSpecIndex')
            AssertCleanField(PlayerCastingBarFrame, 'showCastbar')
            AssertCleanField(OverlayPlayerCastingBarFrame, 'showCastbar')
            AssertCleanField(OverlayPlayerCastingBarFrame, 'overrideBarType')
            "#,
        )
        .expect("documented neutrality on fields actually mutated by vendor code");
        assert_eq!(env.state().borrow().player.pending_spec_change, Some(2));
        assert_eq!(
            env.state()
                .borrow()
                .casting
                .as_ref()
                .expect("modeled spec cast")
                .spell_id,
            200749
        );
        complete_spec_cast(&env);
        assert_eq!(env.state().borrow().player.active_spec_index, 2);
        assert_eq!(env.state().borrow().talents.active_spec_id, 66);
        env.exec(
            "assert(UnitCastingInfo('player') == nil); \
             AssertCleanField(PlayerSpellsFrame.SpecFrame, 'activatedSpecIndex'); \
             AssertCleanField(PlayerCastingBarFrame, 'showCastbar'); \
             AssertCleanField(OverlayPlayerCastingBarFrame, 'showCastbar')",
        )
        .expect("completed helper cast keeps real UI fields clean");
        assert_no_lua_errors(&env);
    }
}

#[test]
fn cached_class_talent_loadout_helpers_keep_real_talent_fields_clean() {
    for helper in ["SwitchToLoadoutByIndex", "SwitchToLoadoutByName"] {
        let env = load_helper_ui();
        let expected: f64 = env.eval(
            r#"
            PlayerSpellsFrame:SetTab(PlayerSpellsFrame.talentTabID)
            PlayerSpellsFrame:Show()
            local frame = PlayerSpellsFrame.TalentsFrame
            assert(frame.variablesLoaded and frame.configIDs and #frame.configIDs >= 2)
            assert(not frame:IsCommitInProgress() and not frame:IsSpecActivationInProgress())
            TargetLoadoutID = frame.configIDs[2]
            TargetLoadoutName = frame.configIDToName[TargetLoadoutID]
            assert(TargetLoadoutName and #TargetLoadoutName > 0)
            assert(C_ClassTalents.GetActiveConfigID() ~= TargetLoadoutID, 'nontrivial state transition')
            -- Real vendor priming avoids giving first-write raw-set taint gaps
            -- credit: the tested helper must overwrite an existing clean slot.
            frame:LoadConfigInternal(C_ClassTalents.GetActiveConfigID(), true)
            assert(frame.isConfigReadyToApply == false)
            AssertCleanField(frame, 'isConfigReadyToApply')
            return TargetLoadoutID
            "#,
        ).expect("initialize actual loadout UI and choose a different seeded loadout");
        let argument = if helper.ends_with("ByIndex") {
            "2"
        } else {
            "TargetLoadoutName"
        };
        env.exec(&format!(
            "RunAddonHelper(ClassTalentHelper.{helper}, {argument})"
        ))
        .expect("addon enters cached helper and real loadout callback");
        let actual: f64 = env
            .eval("return C_ClassTalents.GetActiveConfigID()")
            .expect("state-backed loadout ID");
        assert_eq!(actual, expected);
        env.exec(
            r#"
            assert(PlayerSpellsFrame.TalentsFrame.isConfigReadyToApply == true, 'real LoadConfigInternal wrote its Ready result')
            AssertCleanField(PlayerSpellsFrame.TalentsFrame, 'isConfigReadyToApply')
            AssertCleanField(PlayerSpellsFrame.TalentsFrame, 'stagedPurchaseNodesForNextCommit')
            "#,
        ).expect("loadout UI writes are untainted");
        // LoadConfig currently reports Ready rather than LoadInProgress. No
        // claim that this fixture exercises the loadout commit/castbar path.
        assert_no_lua_errors(&env);
    }
}

#[test]
fn cached_helper_delegate_restores_caller_after_addon_callback_error() {
    let env = load_helper_ui();
    env.exec(
        r#"
        local function failingObserver()
            assert(debug.getstacktaint() == 'UntrustedObserver', 'do not sanitize addon closures')
            error('intentional observer failure')
        end
        debug.setobjecttaint(failingObserver, 'UntrustedObserver')
        RegisterEventCallback('CLASS_TALENTS_SWITCH_TO_SPECIALIZATION_BY_INDEX', failingObserver)
        local function addon()
            assert(debug.getstacktaint() == 'HeaderHelperAudit')
            local ok, message = pcall(ClassTalentHelper.SwitchToSpecializationByIndex, -1)
            assert(not ok and tostring(message):find('intentional observer failure', 1, true))
            assert(not issecure(), 'exception must not launder caller')
            assert(debug.getstacktaint() == 'HeaderHelperAudit', 'restore owner after delegate exception')
        end
        debug.setobjecttaint(addon, 'HeaderHelperAudit')
        assert(issecure())
        addon()
        assert(issecure(), 'outer context recovered after handled error')
        "#,
    )
    .expect("real helper delegates, invalid vendor selection does not start a cast, addon observer stays tainted");
    let state = env.state().borrow();
    assert_eq!(state.player.active_spec_index, 3);
    assert!(state.player.pending_spec_change.is_none());
    assert!(state.casting.is_none());
    drop(state);
    assert_no_lua_errors(&env);
}

fn complete_spec_cast(env: &WowLuaEnv) {
    let pending = env.state().borrow().player.pending_spec_change;
    assert!(pending.is_some(), "helper initiated a specialization cast");
    wow_ui_sim::lua_api::cast_completion::tick_casting(env);
    assert_eq!(env.state().borrow().player.pending_spec_change, pending);
    assert!(
        env.state().borrow().casting.is_some(),
        "no premature completion"
    );
    // Advance only the cast deadline. Do not assign the resulting spec/loadout,
    // clear the cast, or synthesize completion notifications in the fixture.
    env.state()
        .borrow_mut()
        .casting
        .as_mut()
        .expect("spec cast")
        .end_time = 0.0;
    wow_ui_sim::lua_api::cast_completion::tick_casting(env);
    assert!(env.state().borrow().casting.is_none());
    assert!(env.state().borrow().player.pending_spec_change.is_none());
}

fn listen_for_helper_lifecycle(env: &WowLuaEnv) {
    env.exec(r#"
        HelperLifecycleEvents = {}
        local listener = CreateFrame('Frame')
        for _, event in ipairs({'UNIT_SPELLCAST_START', 'UNIT_SPELLCAST_STOP',
                               'UNIT_SPELLCAST_SUCCEEDED', 'PLAYER_SPECIALIZATION_CHANGED',
                               'ACTIVE_COMBAT_CONFIG_CHANGED', 'ACTIVE_PLAYER_SPECIALIZATION_CHANGED'}) do
            listener:RegisterEvent(event)
        end
        listener:SetScript('OnEvent', function(_, event, ...)
            local row = {event = event, count = select('#', ...), spec = GetSpecialization(),
                         config = C_ClassTalents.GetActiveConfigID(), ...}
            table.insert(HelperLifecycleEvents, row)
        end)
    "#).expect("observe actual producer and completion notifications");
}

fn assert_spec_switch_completes(
    env: &WowLuaEnv,
    command: &str,
    expected_index: i32,
    expected_spec_id: u32,
    expected_config: i32,
) {
    let previous_index = env.state().borrow().player.active_spec_index;
    let previous_config = env.state().borrow().talents.active_config_id;
    env.exec("PlayerSpellsFrame:SetTab(PlayerSpellsFrame.specTabID); PlayerSpellsFrame:Show(); HelperLifecycleEvents = {}")
        .expect("initialize actual specialization contents");
    env.exec(command)
        .expect("call actual specialization helper");
    assert_eq!(
        env.state().borrow().player.active_spec_index,
        previous_index
    );
    assert_eq!(
        env.state().borrow().talents.active_config_id,
        previous_config
    );
    assert_eq!(
        env.state().borrow().player.pending_spec_change,
        Some(expected_index)
    );
    complete_spec_cast(env);
    env.exec(&format!(
        r#"
        local events = {{}}
        for _, row in ipairs(HelperLifecycleEvents) do
            if row.event ~= 'ACTIVE_COMBAT_CONFIG_CHANGED' then table.insert(events, row) end
        end
        assert(#events == 5, 'START, STOP, SUCCEEDED, and both specialization notifications')
        assert(events[1].event == 'UNIT_SPELLCAST_START')
        assert(events[2].event == 'UNIT_SPELLCAST_STOP')
        assert(events[3].event == 'UNIT_SPELLCAST_SUCCEEDED')
        assert(events[4].event == 'PLAYER_SPECIALIZATION_CHANGED' and events[4][1] == 'player')
        for i = 1, 3 do
            assert(events[i].spec == {previous_index} and events[i].config == {previous_config})
            assert(events[i][1] == 'player' and events[i][3] == 200749)
            assert(events[i][2] == events[1][2] and events[i][4] == events[1][4])
        end
        assert(events[4].spec == {expected_index} and events[4].config == {expected_config})
        assert(events[5].event == 'ACTIVE_PLAYER_SPECIALIZATION_CHANGED' and events[5].count == 0)
        assert(events[5].spec == {expected_index} and events[5].config == {expected_config})
        assert(C_ClassTalents.GetLastSelectedSavedConfigID({expected_spec_id}) == {expected_config})
        local heroes = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, {expected_spec_id})
        assert(tContains(heroes, C_ClassTalents.GetActiveHeroTalentSpec()))
        assert(UnitCastingInfo('player') == nil)
    "#
    ))
    .expect("completed state and real event ordering retain old mapping guarantees");
    let event_count: i32 = env
        .eval("return #HelperLifecycleEvents")
        .expect("event count");
    wow_ui_sim::lua_api::cast_completion::tick_casting(env);
    assert_eq!(
        env.eval::<i32>("return #HelperLifecycleEvents").unwrap(),
        event_count
    );
    assert_no_lua_errors(env);
}

// The existing hero_talents test delegates here for the retail epoch, so its
// seeded mapping contract is still asserted through the actual completion path.
#[cfg(feature = "gui")]
pub(crate) fn assert_seeded_helper_switch_lifecycle() {
    let env = load_helper_ui_for_spec(2);
    listen_for_helper_lifecycle(&env);
    env.exec(
        r#"
        PlayerSpellsFrame:SetTab(PlayerSpellsFrame.talentTabID)
        PlayerSpellsFrame:Show()
        local configs = C_ClassTalents.GetConfigIDsBySpecID(66)
        assert(#configs == 2 and C_ClassTalents.GetActiveConfigID() == configs[1])
        RunAddonHelper(ClassTalentHelper.SwitchToLoadoutByName, 'Protection Mythic+')
        assert(C_ClassTalents.GetActiveConfigID() == configs[2] and configs[2] == 202)
        assert(C_ClassTalents.GetLastSelectedSavedConfigID(66) == 202)
        assert(C_Traits.GetConfigInfo(202).name == 'Protection Mythic+')
        local changed = HelperLifecycleEvents[#HelperLifecycleEvents]
        assert(changed.event == 'ACTIVE_COMBAT_CONFIG_CHANGED' and changed[1] == 202)
    "#,
    )
    .expect("real loadout callback reaches existing instant completion provider");
    assert_spec_switch_completes(
        &env,
        "RunAddonHelper(ClassTalentHelper.SwitchToSpecializationByName, 'Holy')",
        1,
        65,
        101,
    );
    env.exec(
        r#"
        PlayerSpellsFrame:SetTab(PlayerSpellsFrame.talentTabID)
        PlayerSpellsFrame:Show()
        local configs = C_ClassTalents.GetConfigIDsBySpecID(65)
        assert(#configs == 2 and configs[2] == 102)
        RunAddonHelper(ClassTalentHelper.SwitchToLoadoutByIndex, 2)
        assert(C_ClassTalents.GetActiveConfigID() == 102)
        assert(C_ClassTalents.GetLastSelectedSavedConfigID(65) == 102)
        assert(C_Traits.GetConfigInfo(102).name == 'Holy Raid')
        local changed = HelperLifecycleEvents[#HelperLifecycleEvents]
        assert(changed.event == 'ACTIVE_COMBAT_CONFIG_CHANGED' and changed[1] == 102)
    "#,
    )
    .expect("index helper retains current specialization loadout ordering");
    assert_spec_switch_completes(
        &env,
        "RunAddonHelper(ClassTalentHelper.SwitchToSpecializationByIndex, 3)",
        3,
        70,
        301,
    );
}

#[test]
fn cached_helper_completion_updates_talent_state_before_specialization_notification() {
    let env = load_helper_ui();
    listen_for_helper_lifecycle(&env);
    assert_spec_switch_completes(
        &env,
        "RunAddonHelper(ClassTalentHelper.SwitchToSpecializationByName, 'Holy')",
        1,
        65,
        101,
    );
}
