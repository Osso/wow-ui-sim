#![cfg(feature = "profession-spec-paths")]

use wow_ui_sim::c_api::profession_spec_inputs::ProfessionPathRanks;
use wow_ui_sim::lua_api::WowLuaEnv;

const INVALID_ARGUMENTS_LUA: &str = r#"
    local usage = 'Usage: local state = C_ProfSpecs.GetStateForPath(pathID, configID)'
    local api = C_ProfSpecs.GetStateForPath
    local invalid = {
        function() api() end, function() api(1001) end,
        function() api(nil, 700) end, function() api(1001, nil) end,
        function() api('nope', 700) end, function() api(1001, {}) end,
        function() api(0/0, 700) end, function() api(math.huge, 700) end,
        function() api(-math.huge, 700) end,
        function() api(2147483647.5, 700) end,
        function() api(-2147483648.5, 700) end,
        function() api(2147483648, 700) end,
        function() api(1001, -2147483649) end,
    }
    for _, call in ipairs(invalid) do
        local ok, err = pcall(call)
        if ok or not string.find(err, usage, 1, true) then return false end
    end
    local range = 'outside of expected range -2147483648 to 2147483647 - ' .. usage
    for _, args in ipairs({{2147483648, 700}, {1001, 2147483648}}) do
        local ok, err = pcall(api, args[1], args[2])
        if ok or not string.find(err, range, 1, true) then return false end
    end
    return api(-2147483648, 2147483647) == Enum.ProfessionsSpecPathState.Locked
"#;

fn path_ranks(current: i32, effective_max: i32) -> ProfessionPathRanks {
    ProfessionPathRanks {
        current,
        effective_max,
    }
}

fn env_with_path_ranks() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create profession path environment");
    let mut sim = env.state().borrow_mut();
    let inputs = &mut sim.profession_spec_inputs;
    inputs.trait_config_ids.insert(700);
    inputs
        .path_required_ranks
        .extend([(1001, 2), (2001, 1), (3001, 4)]);
    inputs.path_ranks_by_config.extend([
        ((1001, 700), path_ranks(2, 4)),
        ((2001, 700), path_ranks(5, 5)),
        ((3001, 700), path_ranks(3, 5)),
    ]);
    drop(sim);
    env
}

#[test]
fn get_state_returns_locked_progressing_and_completed_for_backed_paths() {
    let env = env_with_path_ranks();
    assert!(env.eval::<bool>(r#"
        local api = C_ProfSpecs.GetStateForPath
        local states = Enum.ProfessionsSpecPathState
        if states.Locked ~= 0 or states.Progressing ~= 1 or states.Completed ~= 2 then return false end
        local n, state = select('#', api('1001.9', '700.9')), api('1001.9', '700.9')
        if n ~= 1 or state ~= states.Progressing then return false end
        if api(2001, 700) ~= states.Completed then return false end
        return api(3001, 700) == states.Locked
    "#).expect("path state follows supplied ranks"));
}

#[test]
fn get_state_locks_missing_path_and_config() {
    let env = env_with_path_ranks();
    assert!(
        env.eval::<bool>(
            r#"
        local api = C_ProfSpecs.GetStateForPath
        local states = Enum.ProfessionsSpecPathState
        return api(9999, 700) == states.Locked and api(1001, 701) == states.Locked
    "#
        )
        .expect("missing path and config are Locked")
    );
}

#[test]
fn get_state_uses_signed32_usage_and_range_errors() {
    let env = env_with_path_ranks();
    assert!(
        env.eval::<bool>(INVALID_ARGUMENTS_LUA)
            .expect("Usage and int32 errors")
    );
}

#[test]
fn entry_and_currency_queries_return_one_native_shaped_value() {
    let env = env_with_path_ranks();
    let mut sim = env.state().borrow_mut();
    sim.profession_spec_inputs
        .path_spend_entry_ids
        .insert(1001, 4101);
    sim.profession_spec_inputs
        .path_unlock_entry_ids
        .insert(1001, 9101);
    sim.profession_spec_inputs
        .path_spend_currency_ids
        .insert(1001, 5101);
    drop(sim);
    assert!(env.eval::<bool>(r#"
        local spendN, spend = select('#', C_ProfSpecs.GetSpendEntryForPath(1001)), C_ProfSpecs.GetSpendEntryForPath(1001)
        local unlockN, unlock = select('#', C_ProfSpecs.GetUnlockEntryForPath(1001)), C_ProfSpecs.GetUnlockEntryForPath(1001)
        local currencyN, currency = select('#', C_ProfSpecs.GetSpendCurrencyForPath(1001)), C_ProfSpecs.GetSpendCurrencyForPath(1001)
        return spendN == 1 and spend == 4101 and unlockN == 1 and unlock == 9101
            and currencyN == 1 and currency == 5101
            and C_ProfSpecs.GetSpendEntryForPath(9999) == 0
            and C_ProfSpecs.GetUnlockEntryForPath(9999) == 0
            and C_ProfSpecs.GetSpendCurrencyForPath(9999) == nil
    "#).expect("path entry and currency results follow supplied rows"));
}
