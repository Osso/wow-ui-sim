//! Integration tests for the paragon-rep `C_Reputation` lookups in
//! `src/c_api/c_reputation.rs` and `src/lua_api/globals/faction_probes.rs`:
//! `IsFactionParagon`, `IsFactionParagonForCurrentPlayer`, and
//! `GetFactionParagonInfo` driven by `state.faction_paragon`.

use wow_ui_sim::lua_api::{FactionParagonInfo, WowLuaEnv};

fn sample_paragon() -> FactionParagonInfo {
    FactionParagonInfo {
        current_value: 7_500,
        threshold: 10_000,
        reward_quest_id: 53_982,
        has_reward_pending: false,
        too_low_level_for_paragon: false,
        paragon_storage_level: 3,
    }
}

#[test]
fn is_faction_paragon_is_false_when_unregistered() {
    let env = WowLuaEnv::new().expect("env");
    let result: bool = env
        .eval("return C_Reputation.IsFactionParagon(2507)")
        .unwrap();
    assert!(!result);
}

#[test]
fn is_faction_paragon_reads_state_table() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .faction_paragon
        .insert(2507, sample_paragon());
    let listed: bool = env
        .eval("return C_Reputation.IsFactionParagon(2507)")
        .unwrap();
    let unlisted: bool = env
        .eval("return C_Reputation.IsFactionParagon(2511)")
        .unwrap();
    assert!(listed);
    assert!(!unlisted);
}

#[test]
fn is_faction_paragon_for_current_player_requires_eligible_level() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .faction_paragon
        .insert(2507, sample_paragon());
    let active: bool = env
        .eval("return C_Reputation.IsFactionParagonForCurrentPlayer(2507)")
        .unwrap();
    assert!(active);

    let mut too_low = sample_paragon();
    too_low.too_low_level_for_paragon = true;
    env.state()
        .borrow_mut()
        .faction_paragon
        .insert(2511, too_low);
    let gated: bool = env
        .eval("return C_Reputation.IsFactionParagonForCurrentPlayer(2511)")
        .unwrap();
    assert!(!gated);
}

#[test]
fn is_faction_paragon_for_current_player_is_false_when_unregistered() {
    let env = WowLuaEnv::new().expect("env");
    let result: bool = env
        .eval("return C_Reputation.IsFactionParagonForCurrentPlayer(2507)")
        .unwrap();
    assert!(!result);
}

#[test]
fn get_faction_paragon_info_returns_no_values_when_unset() {
    let env = WowLuaEnv::new().expect("env");
    let count: i32 = env
        .eval("return select('#', C_Reputation.GetFactionParagonInfo(2507))")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_faction_paragon_info_has_profile_specific_arity() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .faction_paragon
        .insert(2507, sample_paragon());
    let count: i32 = env
        .eval("return select('#', C_Reputation.GetFactionParagonInfo(2507))")
        .unwrap();
    assert_eq!(count, if cfg!(feature = "retail-12-0-0") { 6 } else { 5 });
}

#[test]
fn get_faction_paragon_info_preserves_first_five_values() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .faction_paragon
        .insert(2507, sample_paragon());
    env.exec(
        "current, threshold, questID, pending, tooLow = C_Reputation.GetFactionParagonInfo(2507)",
    )
    .unwrap();
    let current: f64 = env.eval("return current").unwrap();
    let threshold: f64 = env.eval("return threshold").unwrap();
    let quest_id: f64 = env.eval("return questID").unwrap();
    let pending: bool = env.eval("return pending").unwrap();
    let too_low: bool = env.eval("return tooLow").unwrap();
    assert!((current - 7_500.0).abs() < 1e-6);
    assert!((threshold - 10_000.0).abs() < 1e-6);
    assert!((quest_id - 53_982.0).abs() < 1e-6);
    assert!(!pending);
    assert!(!too_low);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn get_faction_paragon_info_storage_updates_are_faction_local() {
    let env = WowLuaEnv::new().expect("env");
    let mut second = sample_paragon();
    second.paragon_storage_level = 7;
    second.current_value = 12_500;
    second.has_reward_pending = true;
    second.too_low_level_for_paragon = true;
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.faction_paragon.insert(2507, sample_paragon());
        sim.faction_paragon.insert(2511, second);
    }
    env.exec(
        r#"
        function CheckParagon(id, current, pending, tooLow, storage)
            local values = { C_Reputation.GetFactionParagonInfo(id) }
            assert(select('#', C_Reputation.GetFactionParagonInfo(id)) == 6)
            assert(type(values[1]) == 'number' and values[1] == current)
            assert(type(values[2]) == 'number' and values[2] == 10000)
            assert(type(values[3]) == 'number' and values[3] == 53982)
            assert(type(values[4]) == 'boolean' and values[4] == pending)
            assert(type(values[5]) == 'boolean' and values[5] == tooLow)
            assert(type(values[6]) == 'number' and values[6] == storage)
        end
        CheckParagon(2507, 7500, false, false, 3)
        CheckParagon(2507, 7500, false, false, 3)
        CheckParagon(2511, 12500, true, true, 7)
        assert(select('#', C_Reputation.GetFactionParagonInfo(9999)) == 0)
        "#,
    ).unwrap();
    env.state().borrow_mut().faction_paragon.get_mut(&2507).unwrap().paragon_storage_level = 9;
    env.exec(
        r#"
        CheckParagon(2507, 7500, false, false, 9)
        CheckParagon(2511, 12500, true, true, 7)
        "#,
    ).unwrap();
}

#[test]
fn get_faction_paragon_info_reflects_pending_reward() {
    let env = WowLuaEnv::new().expect("env");
    let mut info = sample_paragon();
    info.has_reward_pending = true;
    info.current_value = 12_500;
    env.state().borrow_mut().faction_paragon.insert(2507, info);
    env.exec("_, _, _, pending = C_Reputation.GetFactionParagonInfo(2507)")
        .unwrap();
    let pending: bool = env.eval("return pending").unwrap();
    assert!(pending);
}

#[test]
fn reputation_bar_paragon_branch_uses_pending_overflow() {
    let env = WowLuaEnv::new().expect("env");
    let mut info = sample_paragon();
    info.has_reward_pending = true;
    info.current_value = 13_000;
    info.threshold = 10_000;
    env.state().borrow_mut().faction_paragon.insert(2507, info);
    env.exec(
        r#"
        local function paragonOverlay(factionID)
            if not C_Reputation.IsFactionParagonForCurrentPlayer(factionID) then
                return nil
            end
            local current, threshold, _, hasReward = C_Reputation.GetFactionParagonInfo(factionID)
            local value = current % threshold
            if hasReward then
                value = value + threshold
            end
            return value, threshold
        end
        value, threshold = paragonOverlay(2507)
        missing = paragonOverlay(2511)
    "#,
    )
    .unwrap();
    let value: f64 = env.eval("return value").unwrap();
    let threshold: f64 = env.eval("return threshold").unwrap();
    let missing: bool = env.eval("return missing == nil").unwrap();
    assert!((value - 13_000.0).abs() < 1e-6);
    assert!((threshold - 10_000.0).abs() < 1e-6);
    assert!(missing);
}
