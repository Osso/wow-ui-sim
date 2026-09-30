//! Bounded retail scenario-unit criteria contract; native no-data policy is inferred.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_scenario_info::UnitCriteriaProgress;
use wow_ui_sim::lua_api::WowLuaEnv;

fn set_credit(env: &WowLuaEnv, unit: &str, actual: i32, percent: f64, display: &str, secret: bool) {
    env.state().borrow_mut().scenario.unit_criteria.insert(
        unit.to_owned(),
        UnitCriteriaProgress {
            actual_value: actual,
            percent_value: percent,
            percent_value_string: display.to_owned(),
            identity_restricted: secret,
        },
    );
}

#[test]
fn scenario_unit_criteria_reads_exact_rows_zero_and_updates() {
    let env = WowLuaEnv::new().unwrap();
    set_credit(&env, "target", 3, 1.5, "1.5%", false);
    set_credit(&env, "party1", 9, 4.125, "4,125 percent", false);
    set_credit(&env, "nameplate1", 0, 0.0, "0%", false);
    env.exec("assert(select('#', C_ScenarioInfo.GetUnitCriteriaProgressValues('target')) == 0)")
        .unwrap();
    env.state().borrow_mut().scenario.in_scenario = true;
    env.exec(
        r#"
        local query = C_ScenarioInfo.GetUnitCriteriaProgressValues
        assert(select('#', query('target')) == 3)
        local credit, percent, display = query('target')
        assert(credit == 3 and percent == 1.5 and display == '1.5%')
        credit, percent, display = query('party1')
        assert(credit == 9 and percent == 4.125 and display == '4,125 percent')
        assert(not issecretvalue(display))
        credit, percent, display = query('nameplate1')
        assert(credit == 0 and percent == 0 and display == '0%')
        credit, percent, display = query(secretwrap('target'))
        assert(credit == 3 and percent == 1.5 and display == '1.5%')
        "#,
    )
    .unwrap();
    set_credit(&env, "target", 7, 2.75, "2.750% supplied", false);
    env.exec(
        r#"
        local credit, percent, display = C_ScenarioInfo.GetUnitCriteriaProgressValues('target')
        assert(credit == 7 and percent == 2.75 and display == '2.750% supplied')
        assert(select(1, C_ScenarioInfo.GetUnitCriteriaProgressValues('party1')) == 9)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .scenario
        .unit_criteria
        .remove("target");
    env.exec("assert(select('#', C_ScenarioInfo.GetUnitCriteriaProgressValues('target')) == 0)")
        .unwrap();
}

#[test]
fn scenario_unit_criteria_secret_outputs_do_not_contaminate_public_values() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().scenario.in_scenario = true;
    set_credit(&env, "target", 3, 1.5, "1.5%", true);
    set_credit(&env, "party1", 3, 1.5, "1.5%", false);
    env.exec(
        r#"
        local query = C_ScenarioInfo.GetUnitCriteriaProgressValues
        local a, b, c = query('target')
        assert(select('#', query('target')) == 3)
        for _, value in ipairs({a, b, c}) do
            assert(issecretvalue(value) and not canaccessvalue(value))
        end
        assert(secretunwrap(a) == 3 and secretunwrap(b) == 1.5 and secretunwrap(c) == '1.5%')
        local x, y, z = query('party1')
        assert(x == 3 and y == 1.5 and z == '1.5%')
        for _, value in ipairs({x, y, z, 3, 1.5, '1.5%'}) do
            assert(not issecretvalue(value) and canaccessvalue(value))
        end
        local token = secretwrap('target')
        a, b, c = query(token)
        assert(issecretvalue(a) and issecretvalue(b) and issecretvalue(c))
        local function tainted()
            assert(not issecure())
            local ok, err = pcall(query, 'target')
            assert(not ok and type(err) == 'string')
            assert(not pcall(query, token))
            x, y, z = query('party1')
            assert(x == 3 and y == 1.5 and z == '1.5%')
            assert(not pcall(secretunwrap, a))
            assert(issecretvalue(a) and not canaccessvalue(a))
        end
        debug.setobjecttaint(tainted, 'ScenarioCriteriaProbe')
        tainted()
        "#,
    )
    .unwrap();
    set_credit(&env, "target", 3, 1.5, "1.5%", false);
    env.exec(
        r#"
        local a, b, c = C_ScenarioInfo.GetUnitCriteriaProgressValues('target')
        assert(a == 3 and b == 1.5 and c == '1.5%')
        assert(not issecretvalue(a) and not issecretvalue(b) and not issecretvalue(c))
        "#,
    )
    .unwrap();
}

#[test]
fn scenario_unit_criteria_no_data_returns_no_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_ScenarioInfo.GetUnitCriteriaProgressValues) == 'function')
        assert(select('#', C_ScenarioInfo.GetUnitCriteriaProgressValues('target')) == 0)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().scenario.in_scenario = true;
    env.exec("assert(select('#', C_ScenarioInfo.GetUnitCriteriaProgressValues('unknown')) == 0)")
        .unwrap();
}

#[test]
fn scenario_unit_criteria_rejects_invalid_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local query = C_ScenarioInfo.GetUnitCriteriaProgressValues
        assert(type(query) == 'function')
        for _, invoke in ipairs({
            function() query() end,
            function() query(nil) end,
            function() query(12) end,
            function() query(false) end,
            function() query({}) end,
            function() query(CreateFrame('Frame')) end,
            function() query(secretwrap(12)) end,
        }) do
            local ok, err = pcall(invoke)
            assert(not ok and type(err) == 'string')
        end
        local token = secretwrap('target')
        assert(select('#', query(token)) == 0)
        local function tainted()
            assert(not issecure())
            assert(not pcall(query, token))
            assert(select('#', query('target')) == 0)
        end
        debug.setobjecttaint(tainted, 'ScenarioCriteriaProbe')
        tainted()
        "#,
    )
    .unwrap();
}
