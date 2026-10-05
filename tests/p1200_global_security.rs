#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_secret_unit_arguments_deny_addon_access_without_mutating_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local unit = secretwrap('player')
        assert(not UnitIsLieutenant(unit))
        assert(not UnitIsMinion(unit))
        assert(not UnitIsNPCAsPlayer(unit))
        assert(not UnitShouldDisplaySpellTargetName(unit))
        assert(UnitSpellTargetClass(unit) == nil)
        assert(UnitThreatLeadSituation(unit, unit) == nil)
        assert(select('#', UnitEmpoweredStageDurations(unit)) == 0)
        local addon = function()
            for _, query in ipairs({UnitIsLieutenant, UnitIsMinion, UnitIsNPCAsPlayer,
                UnitShouldDisplaySpellTargetName, UnitSpellTargetClass, UnitEmpoweredStageDurations}) do
                assert(not pcall(query, unit))
            end
            assert(not pcall(UnitThreatLeadSituation, unit, 'player'))
            assert(not pcall(UnitThreatLeadSituation, 'player', unit))
        end
        debug.setobjecttaint(addon, 'SecretUnitAddon'); addon()
    "#).unwrap();
}

#[test]
fn p1200_cursor_invalid_or_secret_arguments_preserve_mouse_and_grant() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .gamepad_cursor_input_available = true;
    env.exec(
        r#"
        SetCursorPosition(8, 12)
        assert(not pcall(SetCursorPosition, 0/0, 12))
        assert(not pcall(SetCursorPosition, 8, math.huge))
        local secret = secretwrap(100)
        local addon = function() assert(not pcall(SetCursorPosition, secret, 12)) end
        debug.setobjecttaint(addon, 'SecretCursorAddon'); addon()
        local x,y = GetCursorPosition(); assert(x == 8 and y == 12)
    "#,
    )
    .unwrap();
    assert!(
        env.state()
            .borrow()
            .plain_global_inputs
            .gamepad_cursor_input_available
    );
}

#[test]
fn p1200_threat_categories_and_missing_units_use_explicit_host_state() {
    use wow_ui_sim::lua_api::globals::real::publication_12_0_0::ThreatLeadSnapshot;
    let env = WowLuaEnv::new().unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    for category in 0..=3 {
        env.state()
            .borrow_mut()
            .plain_global_inputs
            .threat_lead
            .insert(
                (guid.clone(), guid.clone()),
                ThreatLeadSnapshot {
                    is_first: true,
                    lead_state: category,
                },
            );
        assert_eq!(
            env.eval::<u8>("return UnitThreatLeadSituation('player','player')")
                .unwrap(),
            category
        );
    }
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .threat_lead
        .insert(
            (guid.clone(), guid.clone()),
            ThreatLeadSnapshot {
                is_first: true,
                lead_state: 4,
            },
        );
    env.exec("assert(not pcall(UnitThreatLeadSituation,'player','player'))")
        .unwrap();
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .lieutenant_guids
        .insert("Creature-absent".into());
    env.exec("assert(not UnitIsLieutenant('nonexistent'))")
        .unwrap();
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .threat_state_restricted = true;
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .threat_lead
        .insert(
            (guid.clone(), guid),
            ThreatLeadSnapshot {
                is_first: false,
                lead_state: 0,
            },
        );
    env.exec(
        r#"
        local addon = function()
            local lead = UnitThreatLeadSituation('player','player')
            assert(issecretvalue(lead))
            assert(not pcall(function() return lead < 4 end))
        end
        debug.setobjecttaint(addon, 'ThreatAddon'); addon()
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_unavailable_secret_helpers_are_not_faked() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "assert(rawget(_G,'dropsecretaccess') == nil); assert(rawget(_G,'issecrettable') == nil)",
    )
    .unwrap();
}
