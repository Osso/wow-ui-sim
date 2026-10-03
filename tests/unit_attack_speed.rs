//! `UnitAttackSpeed` reads explicit swing-time inputs. Values and defaults are
//! simulator policy, not native parity.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create attack speed environment");
    env.exec(
        r#"
        function AssertAttackSpeed(unit, main, offhand, restricted)
            local function check(...)
                assert(select('#', ...) == 2, 'two results')
                local gotMain, gotOffhand = ...
                assert(issecretvalue(gotMain) == restricted, 'main secrecy')
                assert(secretunwrap(gotMain) == main, 'main value')
                if offhand == nil then
                    assert(gotOffhand == nil and not issecretvalue(gotOffhand), 'plain nil off-hand')
                else
                    assert(issecretvalue(gotOffhand) == restricted, 'off-hand secrecy')
                    assert(secretunwrap(gotOffhand) == offhand, 'off-hand value')
                end
            end
            check(UnitAttackSpeed(unit))
        end
        "#,
    )
    .expect("install assertion");
    env
}

fn set_player_speeds(env: &WowLuaEnv, main: f64, offhand: Option<f64>) {
    let mut state = env.state().borrow_mut();
    state.player.stats.attack_speed = main;
    state.player.stats.offhand_attack_speed = offhand;
}

#[test]
fn player_speeds_follow_configured_inputs_live() {
    let env = env();
    env.exec("AssertAttackSpeed('player', 2, 2, false)")
        .expect("default player pair");
    set_player_speeds(&env, 2.4, Some(1.7));
    env.exec("AssertAttackSpeed('player', 2.4, 1.7, false)")
        .expect("configured pair");
    set_player_speeds(&env, 2.8, Some(1.7));
    env.exec("AssertAttackSpeed('player', 2.8, 1.7, false)")
        .expect("main-hand update leaves off-hand");
}

#[test]
fn missing_offhand_is_a_plain_nil_second_result() {
    let env = env();
    set_player_speeds(&env, 3.1, None);
    env.exec("AssertAttackSpeed('player', 3.1, nil, false)")
        .expect("no off-hand");
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec("AssertAttackSpeed('player', 3.1, nil, true)")
        .expect("restricted main, plain nil off-hand");
}

#[test]
fn other_units_do_not_read_player_inputs() {
    let env = env();
    set_player_speeds(&env, 2.4, Some(1.7));
    env.exec(
        r#"
        assert(UnitExists('party1'))
        AssertAttackSpeed('party1', 2, nil, false)
        assert(not UnitExists('missing-unit'))
        AssertAttackSpeed('missing-unit', 0, nil, false)
        "#,
    )
    .expect("party synthetic speed and unknown-unit zero");
}

#[test]
fn restriction_toggle_and_tainted_callers_keep_values_private() {
    let env = env();
    set_player_speeds(&env, 2.4, Some(1.7));
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            "AssertAttackSpeed('player', 2.4, 1.7, {restricted})"
        ))
        .expect("toggle preserves values");
    }
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        local function probe()
            local function check(...)
                assert(select('#', ...) == 2)
                for index = 1, 2 do
                    local value = select(index, ...)
                    assert(issecretvalue(value) and not canaccessvalue(value))
                    assert(not pcall(secretunwrap, value))
                    assert(not pcall(function() return value + 1 end))
                end
            end
            check(UnitAttackSpeed('player'))
            assert(debug.getstacktaint() == 'AttackSpeedProbe')
        end
        debug.setobjecttaint(probe, 'AttackSpeedProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertAttackSpeed('player', 2.4, 1.7, true)
        "#,
    )
    .expect("opaque to tainted callers");
}
