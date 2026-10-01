//! Input-first 12.0.5 charge secrecy contract; RED execution is parent-owned.
//! Secret spell identifiers (AllowedWhenTainted) require a separate opaque operation.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::{LuaApi, LuaApiMut, Val};
use std::time::{Duration, Instant};
use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::WowLuaEnv;

const QUERIES: [&str; 3] = [
    "C_Spell.GetSpellCharges(19750)",
    "C_ActionBar.GetActionCharges(17)",
    "C_SpellBook.GetSpellBookItemCharges(5, 0)",
];
const PRIVATE_FIELDS: [(&str, f64); 4] = [
    ("currentCharges", 1.0),
    ("cooldownStartTime", 12.0),
    ("cooldownDuration", 40.0),
    ("chargeModRate", 2.0),
];

fn seed_charge_fixture(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("cooldown restriction environment");
    {
        let mut state = env.state().borrow_mut();
        state.start_time = Instant::now() - Duration::from_secs(20);
        state.cooldowns_restricted = restricted;
        state.action_bars.insert(17, 19750);
        state.spell_charges.insert(
            19750,
            SpellChargeState {
                current_charges: 1,
                max_charges: 3,
                recharge_start: 12.0,
                recharge_duration: 40.0,
                charge_mod_rate: 2.0,
            },
        );
    }
    env.exec("assert(C_SpellBook.GetSpellBookItemInfo(5, 0).spellID == 19750)")
        .expect("actual player spellbook fixture");
    env
}

fn assert_charge_payloads(env: &WowLuaEnv, restricted: bool) {
    for query in QUERIES {
        env.exec(&format!(
            "local info = {query}; assert(type(info) == 'table', 'configured charge table'); \
             assert(not issecretvalue(info.maxCharges) and info.maxCharges == 3)"
        ))
        .expect(query);
        for (field, expected) in PRIVATE_FIELDS {
            let value: Val = env.eval(&format!("return ({query}).{field}")).expect(field);
            let lua = env.lua();
            let state = lua.state();
            assert!(rilua::api::state_is_secure(state), "secure host inspection");
            assert_eq!(
                rilua::table_security::is_secret_value(state, value),
                restricted,
                "{query}.{field}"
            );
            let payload = if restricted {
                rilua::table_security::unwrap_secret(state, value)
                    .expect("existing guarded host helper")
            } else {
                value
            };
            assert_eq!(payload, Val::Num(expected), "{query}.{field} payload");
        }
    }
}

#[test]
fn predicate_reads_only_explicit_live_input_and_returns_one_public_bool() {
    let env = WowLuaEnv::new().unwrap();
    assert!(!env.state().borrow().cooldowns_restricted);
    for (restricted, combat, stats) in [
        (false, false, false),
        (false, true, true),
        (true, false, false),
        (true, true, true),
        (false, false, true),
    ] {
        {
            let mut state = env.state().borrow_mut();
            state.cooldowns_restricted = restricted;
            state.player.in_combat = combat;
            state.unit_stats_restricted = stats;
        }
        env.exec(&format!(
            "assert(type(C_Secrets.ShouldCooldownsBeSecret) == 'function'); \
             assert(select('#', C_Secrets.ShouldCooldownsBeSecret()) == 1); \
             local value = C_Secrets.ShouldCooldownsBeSecret(); \
             assert(type(value) == 'boolean' and value == {restricted}); \
             assert(not issecretvalue(value) and canaccessvalue(value))"
        ))
        .expect("explicit predicate input, not combat or unit-stat policy");
    }
}

#[test]
fn unrestricted_three_tables_publish_exact_ordinary_charge_numbers() {
    assert_charge_payloads(&seed_charge_fixture(false), false);
}

#[test]
fn restricted_three_tables_keep_max_public_and_preserve_known_secret_payloads() {
    assert_charge_payloads(&seed_charge_fixture(true), true);
}

#[test]
fn restriction_changes_affect_next_queries_without_changing_charge_input() {
    let env = seed_charge_fixture(false);
    assert_charge_payloads(&env, false);
    env.state().borrow_mut().cooldowns_restricted = true;
    assert_charge_payloads(&env, true);
    env.state().borrow_mut().cooldowns_restricted = false;
    assert_charge_payloads(&env, false);
}

#[test]
fn tainted_public_selectors_receive_opaque_fields_without_clearing_taint() {
    let env = seed_charge_fixture(true);
    env.exec(
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'CooldownRestrictionProbe')
            local queries = {
                function() return C_Spell.GetSpellCharges(19750) end,
                function() return C_ActionBar.GetActionCharges(17) end,
                function() return C_SpellBook.GetSpellBookItemCharges(5, 0) end,
            }
            for _, query in ipairs(queries) do
                local info = query()
                assert(type(info) == 'table', 'tainted public selector must resolve')
                assert(not issecretvalue(info.maxCharges) and info.maxCharges == 3)
                for _, field in ipairs({'currentCharges', 'cooldownStartTime',
                    'cooldownDuration', 'chargeModRate'}) do
                    local value = info[field]
                    assert(issecretvalue(value) and not canaccessvalue(value), field)
                    assert(not pcall(secretunwrap, value), field .. ' unwrap')
                    assert(not pcall(function() return value + 1 end), field .. ' arithmetic')
                end
                assert(debug.getstacktaint() == 'CooldownRestrictionProbe')
            end
            assert(C_Secrets.ShouldCooldownsBeSecret() == true)
            assert(debug.getstacktaint() == 'CooldownRestrictionProbe')
        end
        debug.setobjecttaint(addon, 'CooldownRestrictionProbe')
        addon()
        "#,
    )
    .expect("public selector access preserves addon taint and opaque output");
}

#[test]
fn restriction_never_fabricates_missing_spell_action_or_book_charge_data() {
    let env = seed_charge_fixture(false);
    env.state().borrow_mut().spell_charges.clear();
    env.state().borrow_mut().action_bars.remove(&99);
    for restricted in [false, true] {
        env.state().borrow_mut().cooldowns_restricted = restricted;
        env.exec(
            r#"
            assert(C_Spell.GetSpellCharges(19750) == nil, 'known spell without charge row')
            assert(C_Spell.GetSpellCharges(99999) == nil, 'unknown spell')
            assert(C_ActionBar.GetActionCharges(17) == nil, 'assigned noncharge spell')
            assert(C_ActionBar.GetActionCharges(99) == nil, 'empty action')
            assert(C_SpellBook.GetSpellBookItemCharges(5, 0) == nil, 'known book without row')
            assert(C_SpellBook.GetSpellBookItemCharges(99999, 0) == nil, 'unknown book slot')
            assert(C_SpellBook.GetSpellBookItemCharges(5, 1) == nil, 'unmodeled pet bank')
            assert(C_SpellBook.GetSpellBookItemCharges(5, 99) == nil, 'invalid bank')
            "#,
        )
        .expect("restriction is not a source of charge records");
    }
}

#[test]
fn restriction_does_not_make_charge_durations_or_their_timing_secret() {
    let env = seed_charge_fixture(false);
    for restricted in [false, true] {
        env.state().borrow_mut().cooldowns_restricted = restricted;
        env.exec(
            r#"
            local function addon()
                for _, query in ipairs({
                    function() return C_Spell.GetSpellChargeDuration(19750) end,
                    function() return C_ActionBar.GetActionChargeDuration(17) end,
                    function() return C_SpellBook.GetSpellBookItemChargeDuration(5, 0) end,
                }) do
                    local duration = query()
                    assert(duration ~= nil and not issecretvalue(duration))
                    local values = {duration:GetStartTime(), duration:GetTotalDuration(),
                        duration:GetTotalDuration(1), duration:GetModRate(), duration:GetEndTime()}
                    local expected = {12, 20, 40, 2, 32}
                    for index, value in ipairs(values) do
                        assert(not issecretvalue(value) and value == expected[index])
                    end
                    assert(debug.getstacktaint() == 'CooldownDurationProbe')
                end
            end
            debug.setobjecttaint(addon, 'CooldownDurationProbe')
            addon()
            "#,
        )
        .expect("no return-secret annotation on charge durations; original timing preserved");
    }
}

#[test]
fn secret_action_and_book_selectors_require_untainted_callers_separately_from_output() {
    let env = seed_charge_fixture(false);
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        for (name, number) in [
            ("SecretAction", 17.0),
            ("SecretBookSlot", 5.0),
            ("SecretBank", 0.0),
        ] {
            let value = rilua::table_security::wrap_secret(lua.state_mut(), Val::Num(number))
                .expect("host opaque selector");
            lua.state_mut().push(value);
            lua.set_global_val(name, value).unwrap();
            lua.state_mut().pop();
        }
    }
    env.exec(
        r#"
        local queries = {
            function() return C_ActionBar.GetActionCharges(SecretAction) end,
            function() return C_SpellBook.GetSpellBookItemCharges(SecretBookSlot, 0) end,
            function() return C_SpellBook.GetSpellBookItemCharges(5, SecretBank) end,
            function() return C_SpellBook.GetSpellBookItemCharges(SecretBookSlot, SecretBank) end,
        }
        for _, query in ipairs(queries) do
            local info = query()
            assert(type(info) == 'table' and info.currentCharges == 1 and info.maxCharges == 3,
                'secure secret selector must resolve actual charge row')
            assert(not issecretvalue(info.currentCharges), 'output policy remains false')
            local function addon()
                assert(debug.getstacktaint() == 'CooldownSelectorProbe')
                assert(not pcall(query), 'tainted secret selector must reject')
                assert(debug.getstacktaint() == 'CooldownSelectorProbe')
            end
            debug.setobjecttaint(addon, 'CooldownSelectorProbe')
            addon()
        end
        "#,
    )
    .expect("AllowedWhenUntainted selector matrix, not secret-output policy");
}
