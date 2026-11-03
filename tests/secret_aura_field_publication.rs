//! WM61 S3: the original public aura source sees per-field secrecy.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{aura, load_env};

#[test]
fn original_aura_source_publishes_restricted_aura_fields_individually() {
    let env = load_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![aura(9072)];
        state.unit_auras_restricted = true;
    }
    env.exec(
        r#"
        local source = AuraContainerPublicAuraSource
        local info = debug.getinfo(source.GetAuraDataByAuraInstanceID, 'S')
        assert(info.what == 'Lua' and info.source:find('Blizzard_AuraContainerSources.lua', 1, true))
        local result = source:GetAuraDataByAuraInstanceID('player', 9072)
        assert(result ~= nil and not issecretvalue(result), 'outer aura value is public')
        for _, key in ipairs({'auraInstanceID', 'spellId', 'icon', 'applications',
            'name', 'duration', 'expirationTime', 'sourceUnit', 'isHelpful', 'isHarmful', 'points'}) do
            assert(issecretvalue(result[key]), key .. ' must be secret')
        end
        assert(result.dispelName == nil, 'absent dispelName stays nil')
        assert(secretunwrap(result.auraInstanceID) == 9072)
        assert(secretunwrap(result.name) == 'Audit Aura 9072')
        "#,
    )
    .unwrap();
}
