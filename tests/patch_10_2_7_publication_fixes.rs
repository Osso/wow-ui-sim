//! Bounded current-retail contracts, not historical/native parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
    for _, name in ipairs({'ClosePetStables', 'GetWorldPVPAreaInfo'}) do
        for i = 1, 2 do
            assert(rawget(_G, name) == nil, name)
            assert(_G[name] == nil, name)
        end
    end
    assert(type(rawget(C_StableInfo, 'ClosePetStables')) == 'function')
    assert(type(rawget(C_PvP, 'GetWorldPVPAreaInfo')) == 'function')
"#;

#[test]
fn patch_10_2_7_retired_globals_preserve_namespace_successors() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}

#[test]
fn patch_10_2_7_stable_close_clears_state_and_emits_event() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().pet_stables_open = true;
    env.state().borrow_mut().events.clear();
    env.exec(r#"
        assert(type(rawget(C_StableInfo, 'ClosePetStables')) == 'function')
        assert(C_StableInfo.IsAtPetStable())
        C_StableInfo.ClosePetStables()
        assert(not C_StableInfo.IsAtPetStable())
    "#).unwrap();
    assert!(!env.state().borrow().pet_stables_open);
    let events = env.state().borrow_mut().events.drain();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "PET_STABLE_CLOSED");
    assert!(events[0].args.is_empty());
}
