//! Pre-12.0.5 cast counts remain ordinary numbers under cooldown restriction.
#![cfg(all(feature = "retail-12-0-0", not(feature = "retail-12-0-5")))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn supplied_cast_count_stays_ordinary_when_cooldowns_are_restricted() {
    let env = WowLuaEnv::new().expect("bare early cast-count environment");
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = true;
        state.spell_cast_counts.insert(19750, 7);
    }
    env.exec(
        r#"
        assert(select('#', C_Spell.GetSpellCastCount(19750)) == 1)
        local count = C_Spell.GetSpellCastCount(19750)
        assert(type(count) == 'number')
        assert(not issecretvalue(count))
        assert(count == 7)
        local absent = C_Spell.GetSpellCastCount(642)
        assert(type(absent) == 'number')
        assert(not issecretvalue(absent))
        assert(absent == 0)
        "#,
    )
    .expect("early restriction policy retains ordinary supplied and absent counts");
}
