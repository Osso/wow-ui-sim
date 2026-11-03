//! WM61 S3: the runtime-selected AuraUtil source sees per-field secrecy.
//! Blizzard_FrameXMLUtil.toc:15 loads the original AuraUtil.lua.
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
        local info = debug.getinfo(AuraUtil.GetAuraDataByAuraInstanceID, 'S')
        assert(info.what == 'Lua' and info.source:find('AuraUtil.lua', 1, true))
        local result = AuraUtil.GetAuraDataByAuraInstanceID('player', 9072)
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
