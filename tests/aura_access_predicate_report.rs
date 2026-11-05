//! `RequiresUnitAuraAccess` denial through the original `AuraUtil` index consumer.
//!
//! `Blizzard_FrameXMLUtil/AuraUtil.lua:74` reaches `GetAuraDataByIndex` through
//! `CallDataProviderMethod`. The native aura-access decision reports
//! `Auras cannot be accessed when secret while tainted by '%s'` (S2, S9 (b)).
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{aura, load_env};

#[test]
fn aura_util_index_consumer_raises_native_aura_access_denial() {    let env = load_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![aura(1126)];
        state.unit_auras_restricted = true;
    }
    env.exec(
        r#"
        local source = debug.getinfo(AuraUtil.FindAura, 'S').source
        assert(source:find('AuraUtil.lua', 1, true), 'must use the shipped AuraUtil consumer')
        local function addon()
            local ok, err = pcall(AuraUtil.FindAura, function() return true end, 'player', 'HELPFUL')
            assert(not ok, 'restricted aura index access must raise')
            assert(err == "Auras cannot be accessed when secret while tainted by 'AuraAccessProbe'", tostring(err))
        end
        debug.setobjecttaint(addon, 'AuraAccessProbe')
        addon()
    "#,
    )
    .unwrap();
}

#[test]
fn aura_util_index_consumer_permits_caller_without_recorded_taint_source() {
    let env = load_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![aura(1126)];
        state.unit_auras_restricted = true;
    }
    env.exec(
        r#"
        local function addon()
            forceinsecure()
            local aura = AuraUtil.FindAura(function() return true end, 'player', 'HELPFUL')
            assert(aura ~= nil, 'a caller with no recorded taint source keeps aura access')
        end
        debug.setobjecttaint(addon, 'AuraAccessProbe')
        addon()
    "#,
    )
    .unwrap();
}
