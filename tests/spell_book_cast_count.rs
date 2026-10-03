//! Bounded Retail 12.0.5 `C_SpellBook.GetSpellBookItemCastCount`, not native parity proof.
//! Counts are explicit host inputs; zero for missing entries follows the cached documentation.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;

const SLOT: i32 = 5;

/// Environment with a cast count for the spell in player book slot 5 and a decoy keyed by the slot number.
fn fixture_env(count: Option<u32>) -> (WowLuaEnv, u32) {
    let env = WowLuaEnv::new().expect("create spell book environment");
    let spell_id: f64 = env
        .eval(&format!(
            "return C_SpellBook.GetSpellBookItemInfo({SLOT}, Enum.SpellBookSpellBank.Player).spellID"
        ))
        .expect("player book slot resolves to a spell");
    let spell_id = spell_id as u32;
    assert_ne!(spell_id, SLOT as u32, "slot and spell ID must differ");
    {
        let mut state = env.state().borrow_mut();
        state.spell_cast_counts.insert(SLOT as u32, 88);
        if let Some(count) = count {
            state.spell_cast_counts.insert(spell_id, count);
        }
    }
    env.exec(
        r#"
        function AssertCastCount(expected, restricted, ...)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local value = ...
                assert(issecretvalue(value) == restricted, 'secrecy')
                assert(secretunwrap(value) == expected, 'count')
            end
            check(C_SpellBook.GetSpellBookItemCastCount(...))
        end
        "#,
    )
    .expect("install assertion");
    (env, spell_id)
}

fn install_secret_selectors(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("install VM security helpers");
    for (name, number) in [("SecretSlot", f64::from(SLOT)), ("SecretBank", 0.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret NUMBER");
    }
}

#[test]
fn count_follows_the_book_entry_spell_not_the_slot_number() {
    let (env, spell_id) = fixture_env(Some(9));
    env.exec("AssertCastCount(9, false, 5, Enum.SpellBookSpellBank.Player)")
        .expect("spell-keyed count");
    env.state()
        .borrow_mut()
        .spell_cast_counts
        .insert(spell_id, 4);
    env.exec("AssertCastCount(4, false, 5, Enum.SpellBookSpellBank.Player)")
        .expect("live update");
}

#[test]
fn missing_count_entry_or_bank_returns_one_zero() {
    let (env, _) = fixture_env(None);
    env.exec(
        r#"
        AssertCastCount(0, false, 5, Enum.SpellBookSpellBank.Player)
        AssertCastCount(0, false, 99999, Enum.SpellBookSpellBank.Player)
        AssertCastCount(0, false, 5, Enum.SpellBookSpellBank.Pet)
        AssertCastCount(0, false, 0, Enum.SpellBookSpellBank.Player)
        AssertCastCount(0, false, 5.5, Enum.SpellBookSpellBank.Player)
        "#,
    )
    .expect("documented zero when the item is not found");
}

#[test]
fn cooldown_restriction_alone_makes_the_count_secret() {
    let (env, _) = fixture_env(Some(9));
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec("AssertCastCount(9, false, 5, Enum.SpellBookSpellBank.Player)")
        .expect("unit-stat restriction is unrelated");
    env.state().borrow_mut().cooldowns_restricted = true;
    env.exec(
        r#"
        assert(C_Secrets.ShouldCooldownsBeSecret())
        AssertCastCount(9, true, 5, Enum.SpellBookSpellBank.Player)
        AssertCastCount(0, true, 99999, Enum.SpellBookSpellBank.Player)
        "#,
    )
    .expect("restricted counts are secret host numbers with the same payload");
    env.state().borrow_mut().cooldowns_restricted = false;
    env.exec("AssertCastCount(9, false, 5, Enum.SpellBookSpellBank.Player)")
        .expect("plain again");
}

#[test]
fn untainted_callers_may_pass_secret_selectors() {
    let (env, _) = fixture_env(Some(9));
    install_secret_selectors(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        AssertCastCount(9, false, SecretSlot, 0)
        AssertCastCount(9, false, 5, SecretBank)
        AssertCastCount(9, false, SecretSlot, SecretBank)
        assert(issecretvalue(SecretSlot) and issecretvalue(SecretBank))
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("AllowedWhenUntainted authenticates both selectors");
}

#[test]
fn tainted_callers_are_denied_secret_selectors_before_resolution() {
    let (env, _) = fixture_env(Some(9));
    install_secret_selectors(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'CastCountProbe')
            local query = C_SpellBook.GetSpellBookItemCastCount
            local ok, denial = pcall(query, SecretSlot, 0)
            assert(not ok and type(denial) == 'string' and #denial > 0)
            for _, args in ipairs({{5, SecretBank}, {SecretSlot, SecretBank}, {99999, SecretBank}, {'bad', SecretBank}}) do
                local denied, err = pcall(query, args[1], args[2])
                assert(not denied and err == denial, 'secret denial precedes resolution')
            end
            assert(debug.getstacktaint() == before)
            assert(issecretvalue(SecretSlot) and not pcall(secretunwrap, SecretSlot))
            AssertCastCount(9, false, 5, 0)
            assert(debug.getstacktaint() == before, 'public recovery must retain taint')
        end
        debug.setobjecttaint(probe, 'CastCountProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertCastCount(9, false, SecretSlot, SecretBank)
        "#,
    )
    .expect("VM-authenticated denial, unchanged taint and public recovery");
}
