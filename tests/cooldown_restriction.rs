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

#[test]
fn tainted_secret_spell_selector_resolves_configured_opaque_charge_table() {
    let env = seed_charge_fixture(true);
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture heal".into(), 19750);
    let original = env.state().borrow().spell_charges.get(&19750).copied();
    assert_eq!(
        original,
        Some(SpellChargeState {
            current_charges: 1,
            max_charges: 3,
            recharge_start: 12.0,
            recharge_duration: 40.0,
            charge_mod_rate: 2.0,
        })
    );
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let secret = rilua::table_security::wrap_host_secret_number(lua.state_mut(), 19750.0);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val("SecretChargeSpell", secret);
        lua.state_mut().pop();
        inserted.expect("install rooted host-secret charge spell identifier");
    }
    env.exec(
        r#"
        local function capture(...)
            return select('#', ...), ...
        end
        local function addon()
            local before = debug.getstacktaint()
            assert(before == 'SecretChargeSelectorProbe')
            local public = C_Spell.GetSpellCharges('fixture heal')
            assert(type(public) == 'table' and public.maxCharges == 3,
                'public alias resolves configured charge row before secret call')
            assert(debug.getstacktaint() == before)
            assert(issecretvalue(SecretChargeSpell) and not canaccessvalue(SecretChargeSpell))
            local count, info = capture(C_Spell.GetSpellCharges(SecretChargeSpell))
            assert(count == 1, 'secret spell selector returns exactly one value')
            assert(type(info) == 'table', 'secret spell selector resolves configured charge table')
            assert(not issecretvalue(info) and canaccessvalue(info))
            assert(not issecretvalue(info.maxCharges) and canaccessvalue(info.maxCharges))
            assert(info.maxCharges == 3)
            for _, field in ipairs({'currentCharges', 'cooldownStartTime',
                'cooldownDuration', 'chargeModRate'}) do
                local value = info[field]
                assert(issecretvalue(value) and not canaccessvalue(value), field)
                assert(not pcall(secretunwrap, value), field .. ' unwrap')
                assert(not pcall(function() return value + 1 end), field .. ' arithmetic')
            end
            assert(debug.getstacktaint() == before)
            assert(issecretvalue(SecretChargeSpell) and not canaccessvalue(SecretChargeSpell))
            assert(not pcall(secretunwrap, SecretChargeSpell), 'input unwrap')
            assert(not pcall(function() return SecretChargeSpell + 1 end), 'input arithmetic')
            assert(debug.getstacktaint() == before)
        end
        debug.setobjecttaint(addon, 'SecretChargeSelectorProbe')
        addon()
        "#,
    )
    .expect("AllowedWhenTainted secret spell selector resolves without declassification");
    assert_eq!(
        env.state().borrow().spell_charges.get(&19750).copied(),
        original
    );
    env.state().borrow_mut().cooldowns_restricted = false;
    env.exec(
        r#"
        local function addon()
            local before = debug.getstacktaint()
            assert(before == 'SecretChargeSelectorProbe')
            local info = C_Spell.GetSpellCharges('fixture heal')
            assert(type(info) == 'table' and not issecretvalue(info))
            assert(not issecretvalue(info.maxCharges) and info.maxCharges == 3)
            for field, expected in pairs({currentCharges = 1, cooldownStartTime = 12,
                cooldownDuration = 40, chargeModRate = 2}) do
                assert(not issecretvalue(info[field]) and canaccessvalue(info[field]), field)
                assert(info[field] == expected, field)
            end
            assert(issecretvalue(SecretChargeSpell) and not canaccessvalue(SecretChargeSpell))
            assert(debug.getstacktaint() == before)
        end
        debug.setobjecttaint(addon, 'SecretChargeSelectorProbe')
        addon()
        "#,
    )
    .expect("unrestricted public alias preserves configured ordinary charge fields");
    assert_eq!(
        env.state().borrow().spell_charges.get(&19750).copied(),
        original
    );
}

// SS-P1 staging only: proposed behavior, not executed or native-verified.
mod ssp1 {
    use super::*;
    use wow_ui_sim::lua_api::LossOfControlInfo;
    use wow_ui_sim::lua_api::state::SpellCooldownState;

    fn install_selector(env: &WowLuaEnv, name: &str, number: f64) {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic numeric-secret selector");
    }

    fn fixture(restricted: bool) -> WowLuaEnv {
        let env = seed_charge_fixture(restricted);
        {
            let mut state = env.state().borrow_mut();
            state.gcd = None;
            state.spell_cooldowns.clear();
            state.spell_cooldowns.insert(
                19750,
                SpellCooldownState {
                    start: 12.0,
                    duration: 80.0,
                },
            );
            state.spell_loss_of_control.clear();
            state.spell_loss_of_control.insert(
                19750,
                LossOfControlInfo {
                    start_time: 14.0,
                    duration: 24.0,
                    mod_rate: 2.0,
                    is_active: true,
                    should_replace_normal_cooldown: false,
                },
            );
            state.spell_id_aliases.insert("900001".into(), 19750);
        }
        install_selector(&env, "Ssp1Selector", 19750.0);
        install_selector(&env, "Ssp1Alias", 900001.0);
        install_selector(&env, "Ssp1Missing", 900002.0);
        env
    }

    fn capture(env: &WowLuaEnv, query: &str, tainted: bool) {
        env.exec(&format!(
            r#"
            local function capture(...) Ssp1Count = select('#', ...); Ssp1Result = ... end
            local function invoke()
                local before = debug.getstacktaint()
                assert(before == {}, 'fixture caller security')
                assert(issecretvalue(Ssp1Selector))
                capture({query})
                for _, selector in ipairs({{Ssp1Selector, Ssp1Alias, Ssp1Missing}}) do
                    assert(issecretvalue(selector), 'selector remains secret')
                    if {tainted} then
                        assert(not canaccessvalue(selector))
                        assert(not pcall(secretunwrap, selector))
                        assert(not pcall(function() return selector + 1 end))
                    end
                end
                assert(debug.getstacktaint() == before, 'caller taint preserved')
            end
            if {tainted} then debug.setobjecttaint(invoke, 'Ssp1Probe') end
            invoke()
        "#,
            if tainted { "'Ssp1Probe'" } else { "nil" }
        ))
        .expect(query);
    }

    fn assert_numbers(env: &WowLuaEnv, fields: &[(&str, f64)], restricted: bool) {
        for (field, expected) in fields {
            let value: Val = env.eval(&format!("return Ssp1Result.{field}")).unwrap();
            let lua = env.lua();
            assert!(rilua::api::state_is_secure(lua.state()));
            assert_eq!(
                rilua::table_security::is_secret_value(lua.state(), value),
                restricted
            );
            let payload = rilua::table_security::unwrap_secret(lua.state(), value).unwrap();
            assert_eq!(payload, Val::Num(*expected), "{field}");
            drop(lua); // Release the VM borrow before re-entering Lua.
            env.exec(&format!(
                r#"
                local function addon()
                    local value = Ssp1Result.{field}
                    assert(issecretvalue(value) == {restricted})
                    assert(canaccessvalue(value) ~= {restricted})
                    if {restricted} then
                        assert(not pcall(secretunwrap, value))
                        assert(not pcall(function() return value + 1 end))
                    else
                        assert(value == {expected})
                    end
                    assert(debug.getstacktaint() == 'Ssp1OutputProbe')
                end
                debug.setobjecttaint(addon, 'Ssp1OutputProbe'); addon()
            "#
            ))
            .unwrap();
        }
    }

    fn assert_public_table(env: &WowLuaEnv, flags: &str) {
        env.exec(&format!(
            r#"
            assert(Ssp1Count == 1 and type(Ssp1Result) == 'table')
            assert(not issecretvalue(Ssp1Result) and canaccessvalue(Ssp1Result))
            {flags}
        "#
        ))
        .expect("public container and flags");
    }

    #[test]
    fn cooldown_secret_numeric_selector_preserves_table_policy() {
        let env = fixture(false);
        for restricted in [false, true, false] {
            env.state().borrow_mut().cooldowns_restricted = restricted;
            for tainted in [false, true] {
                for selector in ["19750", "Ssp1Selector"] {
                    capture(
                        &env,
                        &format!("C_Spell.GetSpellCooldown({selector})"),
                        tainted,
                    );
                    assert_public_table(
                        &env,
                        "assert(Ssp1Result.isEnabled == true and Ssp1Result.isActive == true); assert(not issecretvalue(Ssp1Result.isEnabled) and not issecretvalue(Ssp1Result.isActive)); assert(Ssp1Result.isOnGCD == nil and Ssp1Result.timeUntilEndOfStartRecovery == nil)",
                    );
                    assert_numbers(
                        &env,
                        &[("startTime", 12.0), ("duration", 80.0), ("modRate", 1.0)],
                        restricted,
                    );
                }
            }
        }
    }

    #[test]
    fn loc_secret_numeric_selector_preserves_table_policy() {
        let env = fixture(false);
        for restricted in [false, true, false] {
            env.state().borrow_mut().cooldowns_restricted = restricted;
            for tainted in [false, true] {
                for selector in ["19750", "Ssp1Selector"] {
                    capture(
                        &env,
                        &format!("C_Spell.GetSpellLossOfControlCooldownInfo({selector})"),
                        tainted,
                    );
                    assert_public_table(
                        &env,
                        "assert(Ssp1Result.isActive == true and Ssp1Result.shouldReplaceNormalCooldown == false); assert(not issecretvalue(Ssp1Result.isActive) and not issecretvalue(Ssp1Result.shouldReplaceNormalCooldown))",
                    );
                    assert_numbers(
                        &env,
                        &[("startTime", 14.0), ("duration", 24.0), ("modRate", 2.0)],
                        restricted,
                    );
                }
            }
        }
    }

    fn assert_duration(env: &WowLuaEnv, expected: [f64; 5]) {
        env.exec(
            r#"
            assert(Ssp1Count == 1 and Ssp1Result ~= nil)
            assert(not issecretvalue(Ssp1Result) and not Ssp1Result:HasSecretValues())
        "#,
        )
        .expect("existing ordinary duration object policy");
        for (method, expected) in [
            "GetStartTime()",
            "GetTotalDuration()",
            "GetTotalDuration(1)",
            "GetModRate()",
            "GetEndTime()",
        ]
        .into_iter()
        .zip(expected)
        {
            env.exec(&format!(
                r#"
                local function addon()
                    local value = Ssp1Result:{method}
                    assert(not issecretvalue(value) and canaccessvalue(value))
                    assert(value == {expected})
                    assert(debug.getstacktaint() == 'Ssp1DurationProbe')
                end
                debug.setobjecttaint(addon, 'Ssp1DurationProbe'); addon()
            "#
            ))
            .expect(method);
        }
    }

    fn exercise_duration(api: &str, expected: [f64; 5], alias_aware: bool) {
        let env = fixture(false);
        for restricted in [false, true, false] {
            env.state().borrow_mut().cooldowns_restricted = restricted;
            for tainted in [false, true] {
                for selector in ["19750", "Ssp1Selector"] {
                    capture(&env, &format!("C_Spell.{api}({selector})"), tainted);
                    assert_duration(&env, expected);
                }
                if alias_aware {
                    for selector in ["900001", "Ssp1Alias"] {
                        capture(&env, &format!("C_Spell.{api}({selector})"), tainted);
                        assert_duration(&env, expected);
                    }
                }
            }
        }
    }

    #[test]
    fn charge_duration_secret_numeric_selector_keeps_public_rate_snapshot() {
        exercise_duration(
            "GetSpellChargeDuration",
            [12.0, 20.0, 40.0, 2.0, 32.0],
            true,
        );
    }

    #[test]
    fn cooldown_duration_secret_numeric_selector_keeps_public_snapshot() {
        exercise_duration(
            "GetSpellCooldownDuration",
            [12.0, 80.0, 80.0, 1.0, 92.0],
            true,
        );
    }

    #[test]
    fn loc_duration_secret_numeric_selector_keeps_public_rate_snapshot() {
        exercise_duration(
            "GetSpellLossOfControlCooldownDuration",
            [14.0, 12.0, 24.0, 2.0, 26.0],
            false,
        );
    }

    #[test]
    fn missing_and_no_state_secret_selectors_preserve_existing_empty_results() {
        let env = fixture(false);
        let r17 = cfg!(feature = "retail-12-1-5");
        {
            let mut state = env.state().borrow_mut();
            state.spell_cooldowns.clear();
            state.spell_charges.clear();
            state.spell_loss_of_control.clear();
        }
        for restricted in [false, true] {
            env.state().borrow_mut().cooldowns_restricted = restricted;
            for tainted in [false, true] {
                for selector in ["900002", "Ssp1Missing", "19750", "Ssp1Selector"] {
                    capture(
                        &env,
                        &format!("C_Spell.GetSpellCooldown({selector})"),
                        tainted,
                    );
                    assert_public_table(
                        &env,
                        "assert(Ssp1Result.isActive == false and Ssp1Result.isEnabled == true)",
                    );
                    assert_numbers(
                        &env,
                        &[("startTime", 0.0), ("duration", 0.0), ("modRate", 1.0)],
                        restricted,
                    );
                    capture(
                        &env,
                        &format!("C_Spell.GetSpellCooldownDuration({selector})"),
                        tainted,
                    );
                    // R17 separates valid unknown identifiers from supported inactive spells.
                    if !r17 || matches!(selector, "19750" | "Ssp1Selector") {
                        assert_duration(&env, [0.0, 0.0, 0.0, 1.0, 0.0]);
                    } else {
                        env.exec("assert(Ssp1Count == 0 and Ssp1Result == nil)")
                            .unwrap();
                    }

                    capture(
                        &env,
                        &format!("C_Spell.GetSpellChargeDuration({selector})"),
                        tainted,
                    );
                    if r17 {
                        env.exec("assert(Ssp1Count == 0 and Ssp1Result == nil)")
                            .unwrap();
                    } else {
                        env.exec("assert(Ssp1Count == 1 and Ssp1Result == nil)")
                            .unwrap();
                    }

                    capture(
                        &env,
                        &format!("C_Spell.GetSpellLossOfControlCooldownInfo({selector})"),
                        tainted,
                    );
                    env.exec("assert(Ssp1Count == 1 and Ssp1Result == nil)")
                        .unwrap();
                    capture(
                        &env,
                        &format!("C_Spell.GetSpellLossOfControlCooldownDuration({selector})"),
                        tainted,
                    );
                    env.exec("assert(Ssp1Count == 0 and Ssp1Result == nil)")
                        .unwrap();
                }
            }
        }
    }

    fn load_original_action_button_consumer(env: &WowLuaEnv) {
        let path = crate::common::blizzard_ui_dir()
            .join("Blizzard_ActionBar/Shared/ActionButton.lua");
        let source =
            std::fs::read_to_string(path).expect("original cached Retail consumer required");
        let start = source
            .find("local defaultCooldownInfo =")
            .expect("original consumer defaults");
        let end = source[start..]
            .find("function ClearActionButtonCooldowns(")
            .expect("consumer block end")
            + start;
        // Execute the contiguous original defaults/update/apply block, unmodified.
        // This is fixture isolation, not a reimplementation or mocked API trace.
        env.exec(&source[start..end])
            .expect("original Retail consumer block");
    }

    fn assert_widget_timing(env: &WowLuaEnv, name: &str, expected: (f64, f64, f64)) {
        let state = env.state().borrow();
        let id = state
            .widgets
            .get_id_by_name(name)
            .expect("real Cooldown widget");
        let frame = state.widgets.get(id).unwrap();
        assert_eq!(
            (
                frame.cooldown_start,
                frame.cooldown_duration,
                frame.cooldown_mod_rate
            ),
            expected,
            "{name}"
        );
    }

    #[test]
    fn original_action_button_consumes_cooldown_before_charges_and_loc() {
        // Restricted tainted widget setters remain outside SS-P1 acceptance.
        for (restricted, tainted) in [(false, false), (false, true), (true, false)] {
            let env = fixture(restricted);
            load_original_action_button_consumer(&env);
            env.exec(
                r#"
                Ssp1Button = CreateFrame('Button')
                Ssp1Button.cooldown = CreateFrame('Cooldown', 'Ssp1Normal', Ssp1Button)
                Ssp1Button.chargeCooldown = CreateFrame('Cooldown', 'Ssp1Charge', Ssp1Button)
                Ssp1Button.lossOfControlCooldown = CreateFrame('Cooldown', 'Ssp1Loc', Ssp1Button)
                Ssp1Button.enableLOCCooldown = true
            "#,
            )
            .unwrap();
            for selector in ["19750", "Ssp1Selector"] {
                env.exec("Ssp1Button.cooldown:Clear(); Ssp1Button.chargeCooldown:Clear(); Ssp1Button.lossOfControlCooldown:Clear()").unwrap();
                capture(
                    &env,
                    &format!(
                        "(function() Ssp1Button.spellID = {selector}; ActionButton_UpdateCooldown(Ssp1Button) end)()"
                    ),
                    tainted,
                );
                assert_widget_timing(&env, "Ssp1Normal", (12.0, 80.0, 1.0));
                assert_widget_timing(&env, "Ssp1Charge", (12.0, 40.0, 2.0));
                assert_widget_timing(&env, "Ssp1Loc", (14.0, 24.0, 2.0));
            }
            env.state()
                .borrow_mut()
                .spell_loss_of_control
                .get_mut(&19750)
                .unwrap()
                .should_replace_normal_cooldown = true;
            capture(&env, "ActionButton_UpdateCooldown(Ssp1Button)", tainted);
            assert_widget_timing(&env, "Ssp1Normal", (0.0, 0.0, 1.0));
            assert_widget_timing(&env, "Ssp1Charge", (0.0, 0.0, 1.0));
            assert_widget_timing(&env, "Ssp1Loc", (14.0, 24.0, 2.0));
        }
    }
}
