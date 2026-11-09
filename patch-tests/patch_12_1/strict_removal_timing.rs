use super::*;

const OBSERVE_LIFECYCLE: &str = r#"
StrictTiming = { order = {}, snapshots = {} }
local probe = CreateFrame('Frame')
function StrictTiming.observe(phase)
    local snapshot = {
        inventory = type(GetInventorySlotInfo),
        housing = type(C_Housing.IsInsideOwnHouse),
        icon = type(Enum.EditModeUnitFrameSetting.IconSize),
        get = GetCVar,
        default = GetCVarDefault,
        namespaced = C_CVar.GetCVar,
        control = GetCVar('Sound_MasterVolume'),
        removed = GetCVar('slugSupersampling'),
        removedDefault = GetCVarDefault('slugSupersampling'),
        removedNamespaced = C_CVar.GetCVar('slugSupersampling'),
        eventAccepted = pcall(probe.RegisterEvent, probe, 'BATTLETAG_INVITE_SHOW'),
    }
    table.insert(StrictTiming.order, phase)
    StrictTiming.snapshots[phase] = snapshot
end
for _, event in ipairs({
    'VARIABLES_LOADED', 'PLAYER_LOGIN', 'PLAYER_ENTERING_WORLD',
    'BAG_UPDATE_DELAYED', 'FIRST_FRAME_RENDERED',
}) do probe:RegisterEvent(event) end
probe:SetScript('OnEvent', function(_, event) StrictTiming.observe(event) end)
local second = CreateFrame('Frame')
second:RegisterEvent('PLAYER_ENTERING_WORLD')
second:SetScript('OnEvent', function() StrictTiming.observe('second-world-handler') end)
StrictTiming.observe('initialized')
"#;

fn load_timing_fixture(env: &WowLuaEnv) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("create timing fixture directory");
    let addon = directory.path().join("Blizzard_StrictRemovalTiming");
    fs::create_dir(&addon).expect("create timing fixture addon");
    let toc = addon.join("Blizzard_StrictRemovalTiming.toc");
    fs::write(&toc, "## Interface: 120105\nTiming.lua\n").expect("write timing TOC");
    fs::write(
        addon.join("Timing.lua"),
        "StrictTiming.observe('addon-load')",
    )
    .expect("write timing fixture Lua");
    load_addon(&env.loader_env(), &toc).expect("load timing fixture through addon loader");
    directory
}

const ASSERT_LIFECYCLE: &str = r#"
local snapshots = StrictTiming.snapshots
local initial = assert(snapshots.initialized, 'initialized snapshot missing')
assert(initial.inventory == 'nil' and initial.housing == 'nil', 'old global surfaces absent initially')
assert(initial.icon == 'nil', 'old enum member absent initially')
assert(type(initial.control) == 'string', 'unrelated CVar initialized')
for _, phase in ipairs({
    'addon-load', 'post-load', 'VARIABLES_LOADED', 'PLAYER_LOGIN',
    'PLAYER_ENTERING_WORLD', 'second-world-handler',
}) do
    local seen = assert(snapshots[phase], phase)
    assert(seen.inventory == initial.inventory, phase .. ': inventory surface changed')
    assert(seen.housing == initial.housing, phase .. ': housing surface changed')
    assert(seen.icon == initial.icon, phase .. ': enum surface changed')
    assert(seen.get == initial.get and seen.default == initial.default, phase .. ': wrappers retired early')
    assert(seen.namespaced == initial.namespaced, phase .. ': namespace wrapper retired early')
    assert(seen.control == initial.control, phase .. ': unrelated CVar changed')
end
assert(snapshots['addon-load'].eventAccepted, 'Blizzard load compatibility event rejected')
for phase, seen in pairs(snapshots) do
    if phase ~= 'addon-load' then assert(not seen.eventAccepted, phase .. ': removed event accepted') end
end
local retired = assert(snapshots.BAG_UPDATE_DELAYED, 'BAG_UPDATE_DELAYED snapshot missing')
assert(retired.get ~= initial.get and retired.default ~= initial.default, 'global CVar wrappers should retire at BAG_UPDATE_DELAYED')
assert(retired.namespaced ~= initial.namespaced, 'C_CVar wrapper should retire at BAG_UPDATE_DELAYED')
for _, phase in ipairs({'BAG_UPDATE_DELAYED', 'FIRST_FRAME_RENDERED', 'settled'}) do
    local seen = assert(snapshots[phase], phase)
    assert(seen.inventory == 'nil' and seen.housing == 'nil' and seen.icon == 'nil', phase .. ': absent PTR names appeared')
    assert(seen.removed == nil and seen.removedDefault == nil and seen.removedNamespaced == nil, phase .. ': removed CVars returned')
    assert(seen.control == initial.control, phase .. ': unrelated CVar changed')
    assert(seen.get == retired.get and seen.default == retired.default, phase .. ': wrapper nested')
    assert(seen.namespaced == retired.namespaced, phase .. ': namespace wrapper nested')
end
local position = {}
for index, phase in ipairs(StrictTiming.order) do position[phase] = index end
assert(position.initialized < position['addon-load'], 'fixture loads after initialization')
assert(position['addon-load'] < position['post-load'], 'post-load follows addon load')
assert(position['post-load'] < position.VARIABLES_LOADED, 'variables event follows post-load')
assert(position.VARIABLES_LOADED < position.PLAYER_LOGIN, 'login follows variables')
assert(position.PLAYER_LOGIN < position.PLAYER_ENTERING_WORLD, 'world entry follows login')
assert(position.PLAYER_ENTERING_WORLD < position.BAG_UPDATE_DELAYED, 'cleanup follows first world entry')
assert(position['second-world-handler'] < position.BAG_UPDATE_DELAYED, 'cleanup follows second world handler')
assert(position.BAG_UPDATE_DELAYED < position.FIRST_FRAME_RENDERED, 'first frame follows cleanup')
assert(position.FIRST_FRAME_RENDERED < position.settled, 'settled snapshot follows first frame')
"#;

#[test]
fn strict_removal_timing_retires_after_world_handlers_without_rewrapping() {
    let env = WowLuaEnv::new().expect("initialize environment");
    env.exec(OBSERVE_LIFECYCLE)
        .expect("install lifecycle observer");
    let _fixture = load_timing_fixture(&env);
    env.apply_post_load_workarounds();
    env.exec("StrictTiming.observe('post-load')")
        .expect("observe loaded compatibility surface");

    wow_ui_sim::startup::fire_startup_events(&env);
    env.exec("StrictTiming.observe('settled')")
        .expect("observe completed startup");
    env.exec(ASSERT_LIFECYCLE)
        .expect("retirement follows world handlers and preserves unrelated CVar values");

    env.exec("StrictTiming.savedWrappers = {GetCVar, GetCVarDefault, C_CVar.GetCVar}")
        .expect("retain installed wrapper identities");
    env.apply_post_event_workarounds();
    env.fire_event("PLAYER_ENTERING_WORLD")
        .expect("dispatch subsequent world entry");
    env.exec(
        r#"
        assert(GetCVar == StrictTiming.savedWrappers[1])
        assert(GetCVarDefault == StrictTiming.savedWrappers[2])
        assert(C_CVar.GetCVar == StrictTiming.savedWrappers[3])
        for _, phase in ipairs({'PLAYER_ENTERING_WORLD', 'second-world-handler'}) do
            local seen = StrictTiming.snapshots[phase]
            assert(seen.inventory == 'nil' and seen.housing == 'nil' and seen.icon == 'nil')
            assert(not seen.eventAccepted)
        end
        assert(GetCVar('SLUGSUPERSAMPLING') == nil)
        assert(GetCVarDefault('lastLockedDelvesCompanionAbilities') == nil)
        assert(C_CVar.GetCVar('LASTLOCKEDDELVESCOMPANIONABILITIES') == nil)
        "#,
    )
    .expect("repeat cleanup and subsequent world entry preserve installed wrappers");
}

/// Proves the PTR startup surface has no invented phase-retirement names.
#[test]
fn ptr_retirement_surfaces_match_native_absence() {
    let env = new_game_env();
    let surfaces_match: bool = env
        .eval(
            r#"
            return GetInventorySlotInfo == nil
                and type(C_PaperDollInfo.GetInventorySlotInfo) == "function"
                and C_Housing.IsInsideOwnHouse == nil
                and Enum.EditModeUnitFrameSetting.IconSize == nil
                and Enum.EditModeUnitFrameSetting.DebuffIconSize == 19
                and Enum.EditModeUnitFrameSetting.BigDefensiveIconSize == 21
                and Enum.EditModeUnitFrameSetting.BuffIconSize ~= nil
                and GetCVar("lastLockedDelvesCompanionAbilities") == nil
                and GetCVar("SlugSupersampling") == nil
            "#,
        )
        .expect("PTR native retirement-name probe succeeds");

    assert!(
        surfaces_match,
        "PTR uses the C_PaperDollInfo inventory namespace, omits the old enum member and aliases, and keeps removed CVars absent"
    );
}
