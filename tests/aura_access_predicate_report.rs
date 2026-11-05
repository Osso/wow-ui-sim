//! `RequiresUnitAuraAccess` denial through the original `AuraUtil` index consumer.
//!
//! `Blizzard_FrameXMLUtil/AuraUtil.lua:74` reaches `GetAuraDataByIndex` through
//! `CallDataProviderMethod`. WM105 defines the gate in this order: execution
//! context, recorded taint source (null permits), the generated-secrecy-policy
//! flag, then any of restriction entries 0..3 exactly `Active`. The accessor
//! raises `<ApiName>(): <aura report> <taint suffix>`.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{aura, load_env};
use wow_ui_sim::lua_api::WowLuaEnv;

const DENIAL: &str = "C_UnitAuras.GetAuraDataByIndex(): Auras cannot be accessed when secret \
                      while tainted by 'AuraAccessProbe' Lua Taint: AuraAccessProbe";
const FORCED: &str = "C_UnitAuras.GetAuraDataByIndex(): Auras cannot be accessed when secret \
                      while tainted by '*** TaintForced ***' Lua Taint: *** TaintForced ***";

fn restricted_env(restricted_actions: [u8; 6]) -> WowLuaEnv {
    let env = load_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![aura(1126)];
        state.restricted_actions = restricted_actions;
    }
    env
}

fn find_aura_from_addon(env: &WowLuaEnv) {
    env.exec(
        r#"
        local function addon()
            local ok, err = pcall(AuraUtil.FindAura, function() return true end, 'player', 'HELPFUL')
            DENIED = not ok
            DENIAL_MESSAGE = err
        end
        debug.setobjecttaint(addon, 'AuraAccessProbe')
        addon()
    "#,
    )
    .unwrap();
}

#[test]
fn aura_util_index_consumer_raises_native_aura_access_denial() {
    let env = restricted_env([2, 0, 0, 0, 0, 0]);
    env.exec(
        r#"
        local source = debug.getinfo(AuraUtil.FindAura, 'S').source
        assert(source:find('AuraUtil.lua', 1, true), 'must use the shipped AuraUtil consumer')
    "#,
    )
    .unwrap();
    find_aura_from_addon(&env);
    assert_eq!(env.eval::<bool>("return DENIED").unwrap(), true);
    assert_eq!(env.eval::<String>("return DENIAL_MESSAGE").unwrap(), DENIAL);
}

#[test]
fn forceinsecure_installs_named_forced_taint_that_denies() {
    let env = restricted_env([2, 0, 0, 0, 0, 0]);
    env.exec(
        r#"
        local function addon()
            forceinsecure()
            FORCED_TAINT = debug.getstacktaint()
            local ok, err = pcall(AuraUtil.FindAura, function() return true end, 'player', 'HELPFUL')
            DENIED = not ok
            DENIAL_MESSAGE = err
        end
        debug.setobjecttaint(addon, 'AuraAccessProbe')
        addon()
    "#,
    )
    .unwrap();
    assert_eq!(
        env.eval::<String>("return FORCED_TAINT").unwrap(),
        "*** TaintForced ***"
    );
    assert_eq!(env.eval::<bool>("return DENIED").unwrap(), true);
    assert_eq!(env.eval::<String>("return DENIAL_MESSAGE").unwrap(), FORCED);
}

#[test]
fn any_active_content_restriction_denies_access() {
    for kind in 0..4 {
        let mut states = [0u8; 6];
        states[kind] = 2;
        let env = restricted_env(states);
        find_aura_from_addon(&env);
        assert_eq!(
            env.eval::<bool>("return DENIED").unwrap(),
            true,
            "restriction entry {kind} Active must deny"
        );
        assert_eq!(env.eval::<String>("return DENIAL_MESSAGE").unwrap(), DENIAL);
    }
}

#[test]
fn map_and_chat_restrictions_are_never_read() {
    let env = restricted_env([0, 0, 0, 0, 2, 2]);
    find_aura_from_addon(&env);
    assert_eq!(
        env.eval::<bool>("return DENIED").unwrap(),
        false,
        "Map and Chat states must not deny aura access"
    );
}

#[test]
fn activating_state_does_not_deny() {
    let env = restricted_env([1, 1, 1, 1, 0, 0]);
    find_aura_from_addon(&env);
    assert_eq!(
        env.eval::<bool>("return DENIED").unwrap(),
        false,
        "Activating is not Active and must not deny aura access"
    );
}

#[test]
fn caller_without_a_recorded_source_keeps_aura_access() {
    let env = restricted_env([2, 0, 0, 0, 0, 0]);
    env.exec(
        r#"
        local found = AuraUtil.FindAura(function() return true end, 'player', 'HELPFUL')
        assert(found ~= nil, 'a null source permits aura access')
    "#,
    )
    .unwrap();
}
