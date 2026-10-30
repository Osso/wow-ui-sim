//! Original aura-container consumers with user-approved, non-native defaults.
#![cfg(feature = "retail-12-1-5")]

use crate::common::aura_container_harness::{aura, load_env};
use wow_ui_sim::c_api::aura_duration::SpellAuraDuration;

#[test]
fn pandemic_window_uses_stored_carryover() {
    let env = load_env();
    {
        let mut state = env.state().borrow_mut();
        let mut first = aura(1001);
        first.expiration_time = state.start_time.elapsed().as_secs_f64() + 100.0;
        let mut second = aura(1002);
        second.expiration_time = first.expiration_time;
        state.player.buffs = vec![first, second];
        for (spell, cap) in [(1001, 6.0), (1002, 3.0)] {
            state.spell_aura_durations.insert(
                spell,
                SpellAuraDuration {
                    base_duration_seconds: 20.0,
                    max_carryover_seconds: cap,
                },
            );
        }
    }
    env.exec(
        r#"
        for _, case in ipairs({{1001, 6}, {1002, 3}}) do
            local data = C_UnitAuras.GetAuraDataByAuraInstanceID('player', case[1])
            data.auraType = AuraContainerAuraDataType.Aura
            local startTime, endTime = AuraContainerUtil.GetPandemicWindow('player', data)
            assert(endTime - startTime == case[2])
        end
    "#,
    )
    .unwrap();
}

#[test]
fn private_source_returns_supplied_caster_guid() {
    let env = load_env();
    env.exec(
        r#"
        C_UnitAurasPrivate._state.auraDataByUnit.player = {
            [11] = {sourceGUID = 'Player-1-AAA'},
            [22] = {sourceGUID = 'Player-1-BBB'},
            [33] = {},
        }
        local source = AuraContainerPrivateAuraSourceMixin
        assert(source:GetAuraCasterGUID('player', 11) == 'Player-1-AAA')
        assert(source:GetAuraCasterGUID('player', 22) == 'Player-1-BBB')
        assert(source:GetAuraCasterGUID('player', 33) == nil)
        assert(source:GetAuraCasterGUID('player', 44) == nil)
    "#,
    )
    .unwrap();
}
