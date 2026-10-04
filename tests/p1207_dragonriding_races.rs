//! INFERRED row048 contract: ordered numeric race POI IDs, not native parity.
#![cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_races() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().quest_hub_dragonriding_races.insert(100, vec![901, 803]);
    env.state().borrow_mut().quest_hub_dragonriding_races.insert(200, vec![702]);
    env
}

#[test]
fn empty_default_returns_exactly_one_empty_table() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local get = C_QuestHub.GetDragonridingRacesForAreaPOI
        assert(select('#', get(100)) == 1)
        local result = get(100)
        assert(type(result) == 'table' and next(result) == nil)
    "#).unwrap();
}

#[test]
fn distinct_pois_preserve_declared_order_and_unknown_is_empty() {
    let env = env_with_races();
    env.exec(r#"
        local get = C_QuestHub.GetDragonridingRacesForAreaPOI
        local a, b = get(100), get(200)
        assert(#a == 2 and a[1] == 901 and a[2] == 803)
        assert(#b == 1 and b[1] == 702)
        assert(next(get(999)) == nil)
    "#).unwrap();
}

#[test]
fn replacement_and_removal_are_read_live() {
    let env = env_with_races();
    env.exec("snapshot = C_QuestHub.GetDragonridingRacesForAreaPOI(100); assert(snapshot[1] == 901)").unwrap();
    env.state().borrow_mut().quest_hub_dragonriding_races.insert(100, vec![604, 503, 402]);
    env.exec(r#"
        local result = C_QuestHub.GetDragonridingRacesForAreaPOI(100)
        assert(#result == 3 and result[1] == 604 and result[2] == 503 and result[3] == 402)
        assert(#snapshot == 2 and snapshot[1] == 901 and snapshot[2] == 803)
    "#).unwrap();
    env.state().borrow_mut().quest_hub_dragonriding_races.remove(&100);
    env.exec("assert(next(C_QuestHub.GetDragonridingRacesForAreaPOI(100)) == nil); assert(snapshot[1] == 901)").unwrap();
}

#[test]
fn snapshots_are_detached() {
    let env = env_with_races();
    env.exec(r#"
        local get = C_QuestHub.GetDragonridingRacesForAreaPOI
        local first = get(100)
        first[1], first[2], first[3], first.extra = 1, nil, 3, 'mutated'
        local second = get(100)
        assert(first ~= second and #second == 2)
        assert(second[1] == 901 and second[2] == 803 and second.extra == nil)
        local empty = get(999); empty[1] = 123
        assert(next(get(999)) == nil)
    "#).unwrap();
}

#[test]
fn environments_are_isolated() {
    let first = env_with_races();
    let second = WowLuaEnv::new().unwrap();
    second.state().borrow_mut().quest_hub_dragonriding_races.insert(100, vec![321]);
    first.exec("assert(C_QuestHub.GetDragonridingRacesForAreaPOI(100)[1] == 901)").unwrap();
    second.exec("assert(C_QuestHub.GetDragonridingRacesForAreaPOI(100)[1] == 321); assert(next(C_QuestHub.GetDragonridingRacesForAreaPOI(200)) == nil)").unwrap();
}

#[test]
fn missing_and_invalid_selectors_are_rejected() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local get = C_QuestHub.GetDragonridingRacesForAreaPOI
        assert(not pcall(get))
        for _, value in ipairs({false, '100', {}, 1.5, math.huge, -math.huge, 0/0, 2147483648}) do
            local ok, message = pcall(get, value)
            assert(not ok and string.find(message, 'areaPoiID', 1, true))
        end
        assert(not pcall(get, nil))
        assert(type(get(-1, false, {})) == 'table')
    "#).unwrap();
}

#[test]
fn secure_secrets_are_accepted_and_all_tainted_secrets_precede_validation() {
    let env = env_with_races();
    env.exec(r#"
        debug.settaintmode(true)
        local secretID, secretExtra = secretwrap(100), secretwrap('extra')
        local get = C_QuestHub.GetDragonridingRacesForAreaPOI
        assert(get(secretID, secretExtra)[1] == 901, 'secure secret selector')
        local ok, message = pcall(get, false, secretExtra)
        assert(not ok and string.find(message, 'areaPoiID', 1, true), 'secure invalid selector: ' .. tostring(message))
        local function addon()
            assert(debug.getstacktaint() == 'RaceProbe', 'addon entry taint')
            assert(get(100, 'public')[2] == 803, 'tainted public selector')
            for _, args in ipairs({{secretID}, {100, secretExtra}, {false, secretExtra}, {nil, secretExtra}}) do
                local accepted, errorText = pcall(get, unpack(args, 1, 2))
                assert(not accepted and string.find(errorText, 'untainted caller', 1, true), 'secret denial: ' .. tostring(errorText))
                assert(not string.find(errorText, 'areaPoiID', 1, true), 'authentication must precede validation')
                assert(debug.getstacktaint() == 'RaceProbe', 'taint after rejection')
            end
        end
        debug.setobjecttaint(addon, 'RaceProbe')
        addon()
        assert(debug.getstacktaint() == nil, 'secure caller restored')
    "#).unwrap();
}
