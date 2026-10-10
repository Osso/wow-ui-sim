//! Unassigned prose: failed combat-log registration has no listener side effects.
#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(all(feature = "retail-12-0-0", not(feature = "retail-12-0-5")))]
#[test]
fn prose_spell_cast_count_reads_live_explicit_inputs() {
    use wow_ui_sim::c_api::charge_state::SpellChargeState;

    let env = WowLuaEnv::new().expect("bare spell-count environment");
    let isolated = WowLuaEnv::new().expect("independent bare spell-count environment");
    let assert_count = |env: &WowLuaEnv, spell_id: u32, expected: u32| {
        env.exec(&format!(
            r#"
            assert(select('#', C_Spell.GetSpellCastCount({spell_id})) == 1)
            local count = C_Spell.GetSpellCastCount({spell_id})
            assert(type(count) == 'number')
            assert(not issecretvalue(count))
            assert(count == {expected}, 'spell {spell_id}: expected {expected}, got ' .. tostring(count))
            "#
        ))
        .expect("one ordinary scalar count from the real public getter");
    };
    {
        let mut state = env.state().borrow_mut();
        assert!(state.spell_cast_counts.is_empty());
        state.spell_cast_counts.extend([(19750, 7), (642, 2)]);
        state.spell_charges.insert(
            19750,
            SpellChargeState {
                current_charges: 1,
                max_charges: 3,
                recharge_start: 312.0,
                recharge_duration: 237.0,
                charge_mod_rate: 1.25,
            },
        );
    }
    assert_count(&env, 19750, 7);
    assert_count(&env, 642, 2);
    assert!(isolated.state().borrow().spell_cast_counts.is_empty());
    assert_count(&isolated, 19750, 0);
    assert_count(&isolated, 642, 0);

    env.state()
        .borrow_mut()
        .spell_charges
        .get_mut(&19750)
        .unwrap()
        .current_charges = 3;
    assert_count(&env, 19750, 7);
    env.state().borrow_mut().spell_charges.clear();
    assert_count(&env, 19750, 7);

    env.state().borrow_mut().spell_cast_counts.insert(19750, 11);
    isolated.state().borrow_mut().spell_cast_counts.insert(19750, 4);
    assert_count(&env, 19750, 11);
    assert_count(&env, 642, 2);
    assert_count(&isolated, 19750, 4);
    assert_count(&isolated, 642, 0);
    env.state().borrow_mut().spell_cast_counts.remove(&642);
    assert_count(&env, 642, 0);
    assert_count(&env, 19750, 11);
    env.state().borrow_mut().spell_cast_counts.clear();
    assert_count(&env, 19750, 0);
    assert_count(&isolated, 19750, 4);
}

#[test]
fn prose_combat_log_registration_errors_without_delivery() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        ProseEvents={}
        ProseFrame=CreateFrame('Frame','ProseRegistrationFrame')
        ProseFrame:RegisterEvent('PLAYER_ENTERING_WORLD')
        ProseFrame:SetScript('OnEvent',function(_,event) ProseEvents[#ProseEvents+1]=event end)
        local function addon_registration()
            assert(debug.getstacktaint() == 'ProseAddon')
            for _,event in ipairs({'COMBAT_LOG_EVENT','COMBAT_LOG_EVENT_UNFILTERED'}) do
                local ok,err=pcall(ProseFrame.RegisterEvent,ProseFrame,event)
                assert(not ok and type(err)=='string' and #err>0, event .. ' RegisterEvent succeeded')
                assert(not ProseFrame:IsEventRegistered(event))
                ok,err=pcall(ProseFrame.RegisterUnitEvent,ProseFrame,event,'player')
                assert(not ok and type(err)=='string' and #err>0, event .. ' RegisterUnitEvent succeeded')
                assert(not ProseFrame:IsEventRegistered(event))
            end
            assert(ProseFrame:IsEventRegistered('PLAYER_ENTERING_WORLD'))
        end
        debug.setobjecttaint(addon_registration,'ProseAddon')
        addon_registration()
    "#).unwrap();
    for event in [
        "COMBAT_LOG_EVENT",
        "COMBAT_LOG_EVENT_UNFILTERED",
        "PLAYER_ENTERING_WORLD",
    ] {
        env.fire_event(event).unwrap();
    }
    env.exec("assert(#ProseEvents==1 and ProseEvents[1]=='PLAYER_ENTERING_WORLD')")
        .unwrap();
}

#[test]
fn prose_callback_event_mechanism_is_separate_from_script_registration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_EventUtils.IsCallbackEvent('COMBAT_LOG_EVENT'))
        assert(not C_EventUtils.IsCallbackEvent('PLAYER_ENTERING_WORLD'))
        local f=CreateFrame('Frame')
        local callback=function() end
        local ok = pcall(f.RegisterEventCallback,f,'COMBAT_LOG_EVENT',callback)
        assert(ok and f:IsEventRegistered('COMBAT_LOG_EVENT'))
    "#,
    )
    .unwrap();
}
