//! Role assignment over represented identities. Unsupported-value, authority,
//! clearing, no-op and event-name policies are modeled assumptions, not native proof.
#![cfg(feature = "unit-role-assignment")]

use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn role_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("role environment");
    {
        let mut sim = env.state().borrow_mut();
        assert!(
            sim.party_members.len() >= 2,
            "two represented party members"
        );
        sim.party_members.truncate(2);
        sim.party_group_active = true;
        sim.party_leader_index = None;
        sim.everyone_assistant = false;
        sim.player.name = "RoleOwner-Realm".into();
        sim.party_members[0].name = "RoleFirst-Realm".into();
        sim.party_members[1].name = "RoleSecond-Realm".into();
    }
    env.exec(
        r#"
        local enumSetter = rawget(_G, 'UnitSetRoleEnum')
        local stringSetter = rawget(_G, 'UnitSetRole')
        assert(type(enumSetter) == 'function', 'RED: missing raw UnitSetRoleEnum registration')
        assert(type(stringSetter) == 'function', 'RED: missing raw UnitSetRole registration')
        function RoleResult(expected, ...)
            assert(select('#', ...) == 1, 'setter returns exactly one value')
            local result = ...
            assert(type(result) == 'boolean' and result == expected, 'setter boolean result')
            assert(not issecretvalue(result), 'setter result remains public')
        end
        function RoleView(unit, text, number)
            assert(select('#', UnitGroupRolesAssigned(unit)) == 1, 'string getter arity')
            assert(select('#', UnitGroupRolesAssignedEnum(unit)) == 1, 'enum getter arity')
            assert(UnitGroupRolesAssigned(unit) == text, 'live string role for ' .. tostring(unit))
            assert(UnitGroupRolesAssignedEnum(unit) == number, 'live enum role for ' .. tostring(unit))
        end
        RoleEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ROLE_CHANGED_INFORM')
        listener:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'ROLE_CHANGED_INFORM')
            assert(select('#', ...) == 4, 'exact documented payload arity')
            local changed, from, oldRole, newRole = ...
            for _, value in ipairs({changed, from, oldRole, newRole}) do
                assert(type(value) == 'string', 'documented cstring payload')
            end
            RoleEvents[#RoleEvents + 1] = {changed, from, oldRole, newRole}
        end)
        "#,
    )
    .expect("role API registration boundary and public observation helpers");
    env
}

#[test]
fn unit_role_assignment_roundtrips_both_setters_and_nullable_clear() {
    let env = role_env();
    env.exec(
        r#"
        assert(Enum.LFGRole.Tank == 0 and Enum.LFGRole.Healer == 1 and Enum.LFGRole.Damage == 2)
        assert(Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE == -1)
        RoleView('player', 'NONE', -1)
        RoleView('party1', 'NONE', -1)
        RoleView('party2', 'NONE', -1)
        RoleView(nil, 'NONE', -1)
        local roles = {{0, 'TANK'}, {1, 'HEALER'}, {2, 'DAMAGER'}}
        for _, role in ipairs(roles) do
            RoleResult(true, UnitSetRoleEnum('player', role[1]))
            RoleView('player', role[2], role[1])
            RoleResult(true, UnitSetRole('player', role[2]))
            RoleView('player', role[2], role[1])
        end
        RoleResult(true, UnitSetRoleEnum('player', nil))
        RoleView('player', 'NONE', -1)
        RoleResult(true, UnitSetRole('player', 'HEALER'))
        RoleResult(true, UnitSetRole('player', nil))
        RoleView('player', 'NONE', -1)
        RoleResult(true, UnitSetRoleEnum('player', Enum.LFGRole.Tank))
        RoleResult(true, UnitSetRoleEnum('player', Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE))
        RoleView('player', 'NONE', -1)
        RoleResult(true, UnitSetRole('player', 'TANK'))
        RoleResult(true, UnitSetRole('player', 'NONE'))
        RoleView('player', 'NONE', -1)
        RoleResult(true, UnitSetRoleEnum('player'))
        RoleResult(true, UnitSetRole('player'))
        RoleView('player', 'NONE', -1)
        "#,
    )
    .expect("both setter representations and modeled unset policies");
}

#[test]
fn unit_role_assignment_keeps_player_and_party_views_environment_local() {
    let first = role_env();
    let second = role_env();
    first
        .exec(
            r#"
        RoleResult(true, UnitSetRoleEnum('player', 0))
        RoleResult(true, UnitSetRole('party1', 'HEALER'))
        RoleResult(true, UnitSetRoleEnum('party2', 2))
        RoleView('player', 'TANK', 0)
        RoleView('party1', 'HEALER', 1)
        RoleView('party2', 'DAMAGER', 2)
        "#,
        )
        .expect("distinct represented roles");
    second.exec(
        "RoleView('player', 'NONE', -1); RoleView('party1', 'NONE', -1); RoleView('party2', 'NONE', -1)",
    ).expect("independent environment defaults");
    {
        let mut sim = first.state().borrow_mut();
        sim.player.name = "RenamedOwner-Realm".into();
        sim.player.class_index = 8;
        sim.party_members[0].name = "RenamedFirst-Realm".into();
        sim.party_members[0].class_index = 1;
    }
    first
        .exec(
            r#"
        RoleView('player', 'TANK', 0)
        RoleView('party1', 'HEALER', 1)
        RoleResult(true, UnitSetRoleEnum('party1', nil))
        RoleView('party1', 'NONE', -1)
        RoleView('player', 'TANK', 0)
        RoleView('party2', 'DAMAGER', 2)
        "#,
        )
        .expect("live represented identities; no class/name role inference");
}

fn select_identity(env: &WowLuaEnv, unit: &str) {
    let guid: String = env
        .eval(&format!("return UnitGUID({unit:?})"))
        .expect("represented GUID");
    env.exec("A_Admin.SetTarget('Role alias', 80, 1, false)")
        .expect("target snapshot fixture");
    let mut sim = env.state().borrow_mut();
    let target = sim.current_target.as_mut().expect("target snapshot");
    target.guid = guid;
    target.is_player = true;
    sim.current_focus = sim.current_target.clone();
}

#[test]
fn unit_role_assignment_target_and_focus_aliases_follow_represented_identity() {
    let env = role_env();
    select_identity(&env, "player");
    env.exec(
        r#"
        RoleResult(true, UnitSetRoleEnum('target', 0))
        RoleView('player', 'TANK', 0)
        RoleView('target', 'TANK', 0)
        RoleView('focus', 'TANK', 0)
        RoleResult(true, UnitSetRole('focus', 'HEALER'))
        RoleView('player', 'HEALER', 1)
        "#,
    )
    .expect("player alias writes canonical role");
    select_identity(&env, "party1");
    env.exec(
        r#"
        RoleResult(true, UnitSetRoleEnum('focus', 2))
        RoleView('party1', 'DAMAGER', 2)
        RoleView('target', 'DAMAGER', 2)
        RoleView('player', 'HEALER', 1)
        RoleView('party2', 'NONE', -1)
        "#,
    )
    .expect("party identity aliases isolated from player and other member");
    env.state().borrow_mut().current_target = None;
    env.exec("RoleView('target', 'NONE', -1); RoleView('focus', 'DAMAGER', 2)")
        .expect("alias disappearance does not clear identity backing");
}

#[test]
fn unit_role_assignment_rejects_unknown_units_and_invalid_values_without_events() {
    let env = role_env();
    env.exec(
        r#"
        RoleResult(true, UnitSetRoleEnum('player', 0))
        local before = #RoleEvents
        for _, unit in ipairs({'', 'not-a-unit', 'party0', 'party3', 'target', 'focus', 'pet', 'vehicle', 'enemy1'}) do
            RoleResult(false, UnitSetRoleEnum(unit, 1))
            RoleResult(false, UnitSetRole(unit, 'DAMAGER'))
            RoleView(unit, 'NONE', -1)
        end
        for _, value in ipairs({-2, 3, 99, 0.5, math.huge, -math.huge, 0/0}) do
            RoleResult(false, UnitSetRoleEnum('player', value))
        end
        for _, value in ipairs({'', 'tank', 'DPS', 'GUIDE', ' TANK', 'TANK '}) do
            RoleResult(false, UnitSetRole('player', value))
        end
        RoleView('player', 'TANK', 0)
        RoleView('party1', 'NONE', -1)
        assert(#RoleEvents == before, 'invalid/missing identities cannot publish changes')
        "#,
    ).expect("modeled domain/missing-unit policy preserves existing role");
}

#[test]
fn unit_role_assignment_strict_types_validate_before_unknown_lookup() {
    let env = role_env();
    env.exec(
        r#"
        RoleResult(true, UnitSetRoleEnum('player', 0))
        local before = #RoleEvents
        local function rejected(api, ...)
            local ok, message = pcall(api, ...)
            assert(not ok and type(message) == 'string', 'strict argument type error')
        end
        for _, unit in ipairs({false, true, 12, {}, function() end, string.char(255)}) do
            rejected(UnitSetRoleEnum, unit, 0)
            rejected(UnitSetRole, unit, 'TANK')
        end
        rejected(UnitSetRoleEnum)
        rejected(UnitSetRole)
        rejected(UnitSetRoleEnum, nil, 0)
        rejected(UnitSetRole, nil, 'TANK')
        for _, value in ipairs({'0', false, {}, function() end}) do
            rejected(UnitSetRoleEnum, 'player', value)
            rejected(UnitSetRoleEnum, 'not-a-unit', value)
        end
        for _, value in ipairs({0, false, {}, function() end, string.char(255)}) do
            rejected(UnitSetRole, 'player', value)
            rejected(UnitSetRole, 'not-a-unit', value)
        end
        RoleView('player', 'TANK', 0)
        assert(#RoleEvents == before, 'type errors cannot publish changes')
        "#,
    )
    .expect("strict nullable representations, no coercion or lookup-first bypass");
}

#[test]
fn unit_role_assignment_models_self_leader_and_assistant_authority() {
    let env = role_env();
    env.state().borrow_mut().party_leader_index = Some(1);
    env.exec(
        r#"
        assert(not UnitIsGroupLeader('player') and not UnitIsGroupAssistant('player'))
        RoleResult(true, UnitSetRole('player', 'DAMAGER'))
        local before = #RoleEvents
        RoleResult(false, UnitSetRoleEnum('party1', 0))
        RoleResult(false, UnitSetRole('party2', 'HEALER'))
        RoleView('party1', 'NONE', -1)
        RoleView('party2', 'NONE', -1)
        assert(#RoleEvents == before, 'no authority means no role event')
        "#,
    )
    .expect("self writable, another member needs existing authority");
    env.state().borrow_mut().everyone_assistant = true;
    env.exec(
        "assert(UnitIsGroupAssistant('player')); RoleResult(true, UnitSetRoleEnum('party1', 0)); RoleView('party1', 'TANK', 0)",
    ).expect("existing assistant state authorizes member write");
    {
        let mut sim = env.state().borrow_mut();
        sim.party_group_active = false;
        sim.everyone_assistant = false;
    }
    env.exec(
        r#"
        RoleResult(false, UnitSetRoleEnum('party1', 1))
        RoleView('party1', 'NONE', -1)
        RoleResult(true, UnitSetRole('player', 'HEALER'))
        RoleView('player', 'HEALER', 1)
        "#,
    )
    .expect("inactive roster hidden; modeled solo self role stays writable");
}

#[test]
fn unit_role_assignment_dispatches_changed_inform_synchronously_after_write() {
    let env = role_env();
    env.exec(
        r#"
        local entered, returned = false, false
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ROLE_CHANGED_INFORM')
        listener:SetScript('OnEvent', function(_, event, changed, from, oldRole, newRole)
            assert(not returned, 'documented synchronous callback precedes setter return')
            assert(changed == 'RoleFirst-Realm' and from == 'RoleOwner-Realm', 'modeled live actor names')
            assert(oldRole == 'NONE' and newRole == 'HEALER')
            RoleView('party1', 'HEALER', 1)
            entered = true
        end)
        RoleResult(true, UnitSetRoleEnum('party1', 1))
        returned = true
        assert(entered and #RoleEvents == 1, 'event delivered without tick/queue drain')
        listener:UnregisterEvent('ROLE_CHANGED_INFORM')
        RoleResult(true, UnitSetRole('party1', 'HEALER'))
        assert(#RoleEvents == 1, 'modeled no-op success does not duplicate change event')
        RoleResult(true, UnitSetRoleEnum('party1', nil))
        assert(#RoleEvents == 2)
        local cleared = RoleEvents[2]
        assert(cleared[1] == 'RoleFirst-Realm' and cleared[2] == 'RoleOwner-Realm')
        assert(cleared[3] == 'HEALER' and cleared[4] == 'NONE', 'modeled clear payload')
        "#,
    ).expect("synchronous changed-only event with committed getter view");
}

#[test]
fn unit_role_assignment_untainted_authentic_secrets_roundtrip_without_declassification() {
    let env = role_env();
    env.exec(
        r#"
        local unit, number, text, clear = secretwrap('player'), secretwrap(0), secretwrap('HEALER'), secretwrap(nil)
        assert(debug.getstacktaint() == nil)
        RoleResult(true, UnitSetRoleEnum(unit, 2))
        RoleResult(true, UnitSetRoleEnum('player', number))
        RoleResult(true, UnitSetRoleEnum(unit, number))
        RoleView('player', 'TANK', 0)
        RoleResult(true, UnitSetRole(unit, 'DAMAGER'))
        RoleResult(true, UnitSetRole('player', text))
        RoleResult(true, UnitSetRole(unit, text))
        RoleView('player', 'HEALER', 1)
        assert(UnitGroupRolesAssigned(unit) == 'HEALER')
        assert(UnitGroupRolesAssignedEnum(unit) == 1)
        RoleResult(true, UnitSetRoleEnum(unit, clear))
        RoleView('player', 'NONE', -1)
        RoleResult(true, UnitSetRole(unit, text))
        RoleResult(true, UnitSetRole(unit, clear))
        RoleView('player', 'NONE', -1)
        for _, value in ipairs({unit, number, text, clear}) do assert(issecretvalue(value)) end
        assert(secretunwrap(unit) == 'player' and secretunwrap(number) == 0)
        assert(secretunwrap(text) == 'HEALER' and secretunwrap(clear) == nil)
        assert(debug.getstacktaint() == nil)
        "#,
    ).expect("AllowedWhenUntainted authentic VM secret inputs; original wrappers retained");
}

#[test]
fn unit_role_assignment_untainted_malformed_secret_payloads_validate_without_mutation() {
    let env = role_env();
    env.exec(
        r#"
        RoleResult(true, UnitSetRoleEnum('player', 0))
        local before = #RoleEvents
        local function rejected(api, argument, value, role)
            local ok, message = pcall(api, value, role)
            assert(not ok and type(message) == 'string', 'malformed secret payload type error')
            assert(not message:find('secret access', 1, true), 'untainted input is not access denial')
            assert(issecretvalue(argument), 'type error retains original wrapper')
            RoleView('player', 'TANK', 0)
            assert(#RoleEvents == before)
        end
        for _, payload in ipairs({false, 12, {}, function() end, string.char(255)}) do
            local value = secretwrap(payload)
            rejected(UnitSetRoleEnum, value, value, 1)
            rejected(UnitSetRole, value, value, 'HEALER')
            assert(secretunwrap(value) == payload)
        end
        local nilUnit = secretwrap(nil)
        rejected(UnitSetRoleEnum, nilUnit, nilUnit, 1)
        rejected(UnitSetRole, nilUnit, nilUnit, 'HEALER')
        assert(secretunwrap(nilUnit) == nil)
        for _, payload in ipairs({'1', false, {}, function() end}) do
            local value = secretwrap(payload)
            rejected(UnitSetRoleEnum, value, 'player', value)
            assert(secretunwrap(value) == payload)
        end
        for _, payload in ipairs({1, false, {}, function() end, string.char(255)}) do
            local value = secretwrap(payload)
            rejected(UnitSetRole, value, 'player', value)
            assert(secretunwrap(value) == payload)
        end
        RoleResult(false, UnitSetRoleEnum('player', secretwrap(99)))
        RoleResult(false, UnitSetRole('player', secretwrap('DPS')))
        RoleView('player', 'TANK', 0)
        assert(#RoleEvents == before and debug.getstacktaint() == nil)
        "#,
    )
    .expect("authenticated malformed payloads still obey strict types/domain and stay atomic");
}

#[test]
fn unit_role_assignment_tainted_secret_arguments_deny_before_validation_or_write() {
    let env = role_env();
    env.exec(
        r#"
        RoleResult(true, UnitSetRoleEnum('player', 0))
        local before = #RoleEvents
        local unit, missing, number, text, clear = secretwrap('player'), secretwrap('not-a-unit'),
            secretwrap(1), secretwrap('HEALER'), secretwrap(nil)
        local function denied(api, ...)
            local ok, message = pcall(api, ...)
            assert(not ok and type(message) == 'string')
            assert(message:find('secret access', 1, true), 'secret authentication precedes malformed type/lookup')
            assert(debug.getstacktaint() == 'RoleAssignmentAddon')
            RoleView('player', 'TANK', 0)
            assert(#RoleEvents == before, 'denied secrets do not write or dispatch')
        end
        local function addon()
            denied(UnitSetRoleEnum, unit, 1)
            denied(UnitSetRoleEnum, 'player', number)
            denied(UnitSetRoleEnum, unit, number)
            denied(UnitSetRoleEnum, missing, 1)
            denied(UnitSetRoleEnum, false, number)
            denied(UnitSetRoleEnum, 'player', clear)
            denied(UnitSetRole, unit, 'HEALER')
            denied(UnitSetRole, 'player', text)
            denied(UnitSetRole, unit, text)
            denied(UnitSetRole, missing, 'HEALER')
            denied(UnitSetRole, false, text)
            denied(UnitSetRole, 'player', clear)
            denied(UnitGroupRolesAssigned, unit)
            denied(UnitGroupRolesAssignedEnum, unit)
            for _, value in ipairs({unit, missing, number, text, clear}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
        end
        debug.setobjecttaint(addon, 'RoleAssignmentAddon')
        addon()
        assert(debug.getstacktaint() == nil)
        assert(secretunwrap(unit) == 'player' and secretunwrap(missing) == 'not-a-unit')
        assert(secretunwrap(number) == 1 and secretunwrap(text) == 'HEALER' and secretunwrap(clear) == nil)
        "#,
    ).expect("genuine caller taint; authentication ordering and no side effects");
}

#[test]
fn unit_role_assignment_plain_addon_calls_preserve_taint_and_live_backing() {
    let env = role_env();
    env.exec(
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'RoleAssignmentAddon')
            RoleResult(true, UnitSetRoleEnum('player', 0))
            RoleResult(true, UnitSetRole('party1', 'HEALER'))
            RoleView('player', 'TANK', 0)
            RoleView('party1', 'HEALER', 1)
            assert(debug.getstacktaint() == 'RoleAssignmentAddon')
        end
        debug.setobjecttaint(addon, 'RoleAssignmentAddon')
        addon()
        assert(debug.getstacktaint() == nil)
        RoleView('party2', 'NONE', -1)
        "#,
    )
    .expect("HasRestrictions does not invent blanket addon-write prohibition");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn unit_role_assignment_getters_keep_identity_restricted_secret_payloads() {
    let env = role_env();
    env.exec("RoleResult(true, UnitSetRoleEnum('party1', 1))")
        .expect("public role write before identity restriction");
    let guid: String = env.eval("return UnitGUID('party1')").expect("party GUID");
    env.state().borrow_mut().identity_secret_guids.insert(guid);
    env.exec(
        r#"
        local text = UnitGroupRolesAssigned('party1')
        local number = UnitGroupRolesAssignedEnum('party1')
        assert(issecretvalue(text) and issecretvalue(number))
        assert(secretunwrap(text) == 'HEALER' and secretunwrap(number) == 1)
        local function addon()
            local t, n = UnitGroupRolesAssigned('party1'), UnitGroupRolesAssignedEnum('party1')
            assert(issecretvalue(t) and issecretvalue(n))
            assert(not pcall(secretunwrap, t) and not pcall(secretunwrap, n))
            RoleView('player', 'NONE', -1)
            RoleView('party2', 'NONE', -1)
        end
        debug.setobjecttaint(addon, 'RoleAssignmentAddon')
        addon()
        "#,
    )
    .expect("SecretWhenUnitIdentityRestricted wraps real role rather than constant default");
}

fn load_role_callers(env: &WowLuaEnv) {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let roots = ["Blizzard_UnitPopupShared", "Blizzard_StaticPopup"];
    let addons = discover_blizzard_addon_closure_for_screen(&ui, ScreenKind::Game, &roots);
    for root in roots {
        assert!(
            addons.iter().any(|(name, _)| name == root),
            "missing cached role caller root {root}"
        );
    }
    for (name, toc) in addons {
        let result = load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("cached role caller {name}: {error}"));
        assert!(
            result.warnings.is_empty(),
            "role caller {name}: {:?}",
            result.warnings
        );
        if !result.missing_requirements.is_empty() {
            eprintln!(
                "role caller optional nil-probe observations {name}: {:?}",
                result.missing_requirements
            );
        }
    }
    // Load the entire original file, not an extracted/reimplemented handler.
    // This bounds proof to the original handler on real test-created widgets,
    // not full RolePoll XML/startup/menu interaction or rendered acceptance.
    let path = ui.join("Blizzard_FrameXML/Mainline/RolePoll.lua");
    let source = std::fs::read_to_string(&path).expect("cached original RolePoll source");
    env.exec_named(&source, &format!("@{}", path.display()))
        .expect("whole original RolePoll file");
}

#[test]
fn unit_role_assignment_real_popup_and_role_poll_callers_observe_committed_roles() {
    let env = role_env();
    load_role_callers(&env);
    env.exec(
        r#"
        local function source(handler, suffix)
            assert(type(handler) == 'function', 'original cached caller')
            local path = debug.getinfo(handler, 'S').source:gsub('\\', '/')
            assert(path:sub(-#suffix) == suffix, 'original source identity: ' .. path)
        end
        source(UnitPopupSetRoleNoneButton.OnClick, 'Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua')
        source(RolePollPopupAccept_OnClick, 'Blizzard_FrameXML/Mainline/RolePoll.lua')
        local context = {unit = 'party1'}
        assert(UnitPopupSetRoleTankButton:GetRole() == Enum.LFGRole.Tank)
        assert(UnitPopupSetRoleTankButton:OnClick(context) == MenuResponse.Close)
        RoleView('party1', 'TANK', 0)
        assert(UnitPopupSetRoleTankButton:IsChecked(context))
        assert(not UnitPopupSetRoleHealerButton:IsChecked(context))
        assert(UnitPopupSetRoleHealerButton:OnClick(context) == MenuResponse.Close)
        RoleView('party1', 'HEALER', 1)
        assert(UnitPopupSetRoleHealerButton:IsChecked(context))
        assert(UnitPopupSetRoleNoneButton:OnClick(context) == MenuResponse.Close)
        RoleView('party1', 'NONE', -1)
        assert(RoleEvents[3][3] == 'HEALER' and RoleEvents[3][4] == 'NONE')
        -- No assertion that the popup None radio checks: original GetRole is nil,
        -- but API docs require a nonnil numeric getter and existing sentinel is -1.
        local dialog = CreateFrame('Frame')
        dialog.role = Enum.LFGRole.Damage
        local button = CreateFrame('Button', nil, dialog)
        local entered = false
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ROLE_CHANGED_INFORM')
        listener:SetScript('OnEvent', function(_, _, changed, from, oldRole, newRole)
            assert(changed == 'RoleOwner-Realm' and from == 'RoleOwner-Realm')
            assert(oldRole == 'NONE' and newRole == 'DAMAGER')
            RoleView('player', 'DAMAGER', 2)
            entered = true
        end)
        assert(select('#', RolePollPopupAccept_OnClick(button, 'LeftButton')) == 0, 'original accept is void')
        assert(entered, 'original caller dispatches synchronously')
        RoleView('player', 'DAMAGER', 2)
        listener:UnregisterEvent('ROLE_CHANGED_INFORM')
        dialog.role = Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE
        RolePollPopupAccept_OnClick(button, 'LeftButton')
        RoleView('player', 'NONE', -1)
        RoleView('party2', 'NONE', -1)
        assert(RoleEvents[#RoleEvents][4] == 'NONE', 'real RolePoll sentinel clears')
        "#,
    ).expect("unchanged cached popup/RolePoll whole handlers and synchronous real API effect");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "cached caller Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
}
