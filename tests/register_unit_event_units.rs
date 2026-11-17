//! R88aj RegisterUnitEvent unit-argument scan, observed through return values.

use wow_ui_sim::lua_api::WowLuaEnv;

/// Unit arguments for `f:RegisterUnitEvent('UNIT_HEALTH', ...)` in sequence,
/// then the VERIFIED single boolean (true = registered or filter changed).
const UNIT_CALLS: [(&str, bool); 12] = [
    ("{}", true),
    ("{'player'}", false),
    ("'player', 'party1'", true),
    ("'PLAYER', 'Party1'", false),
    ("'party1', 'player'", true),
    ("'party1', 'player', {}, 'pet'", false),
    ("'party1', 'pettarget', 'player'", false),
    ("'party1', 'player', nil, 'pet'", false),
    ("'party1', 'party2', 'party3', 'party4', 'player'", true),
    ("'party1', 'party2', 'party3', 'party4'", false),
    ("'0', '', 'playerpet', 'pettarget', 'player'", true),
    ("", false),
];

fn register_units(env: &WowLuaEnv, units: &str) -> (i64, bool) {
    let separator = if units.is_empty() { "" } else { ", " };
    env.eval(&format!(
        "local function count(...) return select('#', ...), ... end return count(f:RegisterUnitEvent('UNIT_HEALTH'{separator}{units}))"
    ))
    .expect("RegisterUnitEvent should return")
}

#[test]
fn register_unit_event_scans_up_to_four_string_units_and_replaces_filters() {
    let env = WowLuaEnv::new().expect("create unit-event environment");
    env.exec("f = CreateFrame('Frame')")
        .expect("frame should be created");
    for (units, expected) in UNIT_CALLS {
        assert_eq!(register_units(&env, units), (1, expected), "units: {units}");
    }
}

#[test]
fn register_event_clears_and_unregister_removes_the_unit_filter() {
    let env = WowLuaEnv::new().expect("create unit-event environment");
    env.exec("f = CreateFrame('Frame')")
        .expect("frame should be created");
    assert_eq!(register_units(&env, "'player'"), (1, true));
    let cleared: bool = env
        .eval("return f:RegisterEvent('UNIT_HEALTH')")
        .expect("RegisterEvent should return");
    assert!(
        cleared,
        "RegisterEvent replaces the unit filter with an empty one"
    );
    assert_eq!(register_units(&env, "{}"), (1, false));
    assert_eq!(register_units(&env, "'player'"), (1, true));
    // PRESERVATION of the pre-existing simulator removal (also covered on all
    // profiles by tests/unit_event_second_unit.rs:61-66). R88aj VERIFIES it
    // natively only for Mists 70631; it is UNKNOWN for Retail 70688 and
    // Forever 70707 (UnregisterEvent not decoded).
    env.exec("f:UnregisterEvent('UNIT_HEALTH')")
        .expect("UnregisterEvent should run");
    assert_eq!(register_units(&env, "'player'"), (1, true));
}
