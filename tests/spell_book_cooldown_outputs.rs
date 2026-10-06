//! Batch 68: exact output rows 305/322. Policies are bounded simulator
//! inferences, not native parity. Compilation and RED remain main-owned.
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod current {
    use rilua::{LuaApi, LuaApiMut, Val};
    use std::time::{Duration, Instant};
    use wow_ui_sim::lua_api::WowLuaEnv;
    use wow_ui_sim::lua_api::state::SpellCooldownState;

    const FIELDS: [&str; 3] = ["startTime", "duration", "modRate"];
    const SPELL: (f64, f64) = (312.0, 237.0);
    const GCD: (f64, f64) = (330.0, 300.0);
    const EMPTY: (f64, f64) = (0.0, 0.0);
    const SPELL_QUERY: &str = "C_Spell.GetSpellCooldown(19750)";
    const BOOK_QUERY: &str = "C_SpellBook.GetSpellBookItemCooldown(5, 0)";
    const QUERIES: [&str; 2] = [SPELL_QUERY, BOOK_QUERY];

    fn fixture(restricted: bool, spell: bool, gcd: bool) -> WowLuaEnv {
        let env = WowLuaEnv::new().expect("rows 305/322 environment");
        {
            let mut state = env.state().borrow_mut();
            state.start_time = Instant::now() - Duration::from_secs(335);
            state.cooldowns_restricted = restricted;
            state.spell_cooldowns.clear();
            if spell {
                state.spell_cooldowns.insert(
                    19750,
                    SpellCooldownState {
                        start: SPELL.0,
                        duration: SPELL.1,
                    },
                );
            }
            state.gcd = gcd.then_some(GCD);
        }
        env.exec(
            "local info = C_SpellBook.GetSpellBookItemInfo(5, 0)\nassert(info.spellID == 19750)",
        )
        .expect("actual book DTO verifies slot 5, player bank 0 identity");
        env
    }

    fn capture(env: &WowLuaEnv, query: &str) {
        env.exec(&format!(
            r#"
            assert(select('#', {query}) == 1)
            CDInfo = {query}
            assert(type(CDInfo) == 'table')
            assert(not issecretvalue(CDInfo))
            assert(canaccessvalue(CDInfo))
            CDNumbers = {{CDInfo.startTime, CDInfo.duration, CDInfo.modRate}}
            "#
        ))
        .expect("real namespace query returns one ordinary rooted DTO");
    }

    fn assert_numbers(env: &WowLuaEnv, expected: (f64, f64), restricted: bool) {
        for (field, number) in FIELDS.into_iter().zip([expected.0, expected.1, 1.0]) {
            let value: Val = env.eval(&format!("return CDInfo.{field}")).unwrap();
            let lua = env.lua();
            assert!(rilua::api::state_is_secure(lua.state()));
            let payload = rilua::table_security::unwrap_secret(lua.state(), value)
                .expect("authenticated secure host payload inspection");
            // Meaningful model first: a constant book provider must fail here.
            assert_eq!(payload, Val::Num(number), "{field} selected interval");
            assert_eq!(
                rilua::table_security::is_secret_value(lua.state(), value),
                restricted,
                "{field}"
            );
            drop(lua);
            env.exec(&format!(
                "assert(issecretvalue(CDInfo.{field}) == {restricted})\n\
                 if not {restricted} then assert(type(CDInfo.{field}) == 'number') end"
            ))
            .expect("public numeric type or authentic secret observation");
        }
    }

    fn assert_shape(env: &WowLuaEnv, active: bool) {
        let state = env.state();
        let state = state.borrow();
        let now = state.start_time.elapsed().as_secs_f64();
        let gcd_known = state.gcd.is_some();
        let recovering = state.gcd.is_some_and(|(start, duration)| start + duration > now);
        let is_on_gcd = state.gcd.is_some_and(|(start, duration)| {
            recovering && state.spell_cooldowns.get(&19750)
                .is_none_or(|cooldown| cooldown.start + cooldown.duration <= start + duration)
        });
        let restricted = state.cooldowns_restricted;
        drop(state);
        let expected_count = 5 + usize::from(gcd_known) + usize::from(recovering);
        env.exec(&format!(
            r#"
            assert(type(CDInfo.isEnabled) == 'boolean')
            assert(not issecretvalue(CDInfo.isEnabled))
            assert(CDInfo.isEnabled == true)
            assert(type(CDInfo.isActive) == 'boolean')
            assert(not issecretvalue(CDInfo.isActive))
            assert(CDInfo.isActive == {active})
            assert(CDInfo.activeCategory == nil)
            if {recovering} then
                assert(issecretvalue(CDInfo.timeUntilEndOfStartRecovery) == {restricted})
                if not {restricted} then assert(CDInfo.timeUntilEndOfStartRecovery > 0) end
            else assert(CDInfo.timeUntilEndOfStartRecovery == nil) end
            if {gcd_known} then
                assert(CDInfo.isOnGCD == {is_on_gcd})
                assert(not issecretvalue(CDInfo.isOnGCD))
            else assert(CDInfo.isOnGCD == nil) end
            local count = 0
            for _ in pairs(CDInfo) do count = count + 1 end
            assert(count == {expected_count})
            "#
        ))
        .expect("five required fields and model-derived optional metadata; only numbers may be private");
    }

    fn check(env: &WowLuaEnv, query: &str, expected: (f64, f64), restricted: bool) {
        capture(env, query);
        assert_numbers(env, expected, restricted);
        assert_shape(env, expected.1 > 0.0);
    }

    fn check_both(env: &WowLuaEnv, expected: (f64, f64), restricted: bool) {
        for query in QUERIES {
            check(env, query, expected, restricted);
        }
    }

    fn read_wrappers(env: &WowLuaEnv, global: &str) -> Vec<(Val, u64)> {
        (1..=3)
            .map(|index| {
                let value: Val = env.eval(&format!("return {global}[{index}]")).unwrap();
                let lua = env.lua();
                assert!(rilua::api::state_is_secure(lua.state()));
                assert!(rilua::table_security::is_secret_value(lua.state(), value));
                let Val::Userdata(reference) = value else {
                    panic!("VM wrapper missing")
                };
                let userdata = lua
                    .state()
                    .gc
                    .userdata
                    .get(reference)
                    .expect("rooted live wrapper");
                let payload = rilua::table_security::unwrap_secret(lua.state(), value)
                    .expect("authenticate original VM wrapper metadata");
                assert!(matches!(payload, Val::Num(_)));
                (value, userdata.alloc_seq())
            })
            .collect()
    }

    fn publish_secret(env: &WowLuaEnv, name: &str, number: f64) {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        assert!(rilua::api::state_is_secure(lua.state_mut()));
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.set_global_val(name, value)
            .expect("root actual typed selector");
    }

    fn assert_book_miss(env: &WowLuaEnv, slot: &str, bank: &str) {
        env.exec(&format!(
            "assert(select('#', C_SpellBook.GetSpellBookItemCooldown({slot}, {bank})) == 1)\n\
             assert(C_SpellBook.GetSpellBookItemCooldown({slot}, {bank}) == nil)"
        ))
        .expect("chosen bounded book policy: one nil, never fabricated DTO");
    }

    fn expire(env: &WowLuaEnv) {
        env.state().borrow_mut().start_time = Instant::now() - Duration::from_secs(900);
    }

    #[test]
    fn spell_public_spell_only_interval() {
        check(&fixture(false, true, false), SPELL_QUERY, SPELL, false);
    }

    #[test]
    fn spell_public_gcd_only_interval() {
        check(&fixture(false, false, true), SPELL_QUERY, GCD, false);
    }

    #[test]
    fn spell_public_overlap_selects_latest_ending_gcd() {
        check(&fixture(false, true, true), SPELL_QUERY, GCD, false);
    }

    #[test]
    fn spell_public_no_cooldown_is_enabled_inactive() {
        check(&fixture(false, false, false), SPELL_QUERY, EMPTY, false);
    }

    #[test]
    fn spell_public_expired_intervals_are_zero() {
        let env = fixture(false, true, true);
        expire(&env);
        check(&env, SPELL_QUERY, EMPTY, false);
    }

    #[test]
    fn book_public_spell_only_uses_real_model() {
        check(&fixture(false, true, false), BOOK_QUERY, SPELL, false);
    }

    #[test]
    fn book_public_gcd_only_uses_real_model() {
        check(&fixture(false, false, true), BOOK_QUERY, GCD, false);
    }

    #[test]
    fn book_public_overlap_selects_latest_ending_gcd() {
        check(&fixture(false, true, true), BOOK_QUERY, GCD, false);
    }

    #[test]
    fn book_public_no_cooldown_is_enabled_inactive() {
        check(&fixture(false, false, false), BOOK_QUERY, EMPTY, false);
    }

    #[test]
    fn book_public_expired_intervals_are_zero() {
        let env = fixture(false, true, true);
        expire(&env);
        check(&env, BOOK_QUERY, EMPTY, false);
    }

    #[test]
    fn restricted_both_namespaces_preserve_all_interval_payloads() {
        for (spell, gcd, expected) in [
            (true, false, SPELL),
            (false, true, GCD),
            (true, true, GCD),
            (false, false, EMPTY),
        ] {
            check_both(&fixture(true, spell, gcd), expected, true);
        }
        let env = fixture(true, true, true);
        expire(&env);
        check_both(&env, EMPTY, true);
    }

    #[test]
    fn book_dynamic_real_slot_selects_alternate_spell_not_slot_number() {
        let env = fixture(false, true, false);
        env.exec(
            r#"
            AlternateSlot = C_SpellBook.FindSpellBookSlotForSpell(642)
            assert(type(AlternateSlot) == 'number')
            assert(C_SpellBook.GetSpellBookItemInfo(AlternateSlot, 0).spellID == 642)
        "#,
        )
        .expect("available API discovers actual alternate catalog entry");
        env.state().borrow_mut().spell_cooldowns.insert(
            642,
            SpellCooldownState {
                start: 310.0,
                duration: 270.0,
            },
        );
        check(
            &env,
            "C_SpellBook.GetSpellBookItemCooldown(AlternateSlot, 0)",
            (310.0, 270.0),
            false,
        );
        check(&env, "C_Spell.GetSpellCooldown(642)", (310.0, 270.0), false);
        check_both(&env, SPELL, false);
    }

    #[test]
    fn required_booleans_and_five_field_shape_stay_public() {
        for restricted in [false, true] {
            let env = fixture(restricted, true, false);
            check_both(&env, SPELL, restricted);
            env.state().borrow_mut().spell_cooldowns.clear();
            check_both(&env, EMPTY, restricted);
        }
    }

    #[test]
    fn restriction_flag_is_independent_of_combat_and_stat_policy() {
        let env = fixture(false, true, false);
        for (restricted, combat, stats) in [
            (false, true, true),
            (true, false, false),
            (false, false, true),
        ] {
            {
                let mut state = env.state().borrow_mut();
                state.cooldowns_restricted = restricted;
                state.player.in_combat = combat;
                state.unit_stats_restricted = stats;
            }
            env.exec(&format!(
                "assert(C_Secrets.ShouldCooldownsBeSecret() == {restricted})"
            ))
            .unwrap();
            check_both(&env, SPELL, restricted);
        }
    }

    #[test]
    fn tainted_public_selectors_preserve_caller_and_public_booleans() {
        let env = fixture(true, true, false);
        for query in QUERIES {
            env.exec(&format!(
                r#"
                assert(issecure())
                local function addon()
                    assert(debug.getstacktaint() == 'CooldownOutputProbe')
                    CDInfo = {query}
                    assert(type(CDInfo) == 'table')
                    assert(not issecretvalue(CDInfo))
                    for _, field in ipairs({{'startTime', 'duration', 'modRate'}}) do
                        assert(issecretvalue(CDInfo[field]))
                        assert(not canaccessvalue(CDInfo[field]))
                    end
                    assert(not issecretvalue(CDInfo.isEnabled))
                    assert(CDInfo.isEnabled == true)
                    assert(not issecretvalue(CDInfo.isActive))
                    assert(CDInfo.isActive == true)
                    assert(debug.getstacktaint() == 'CooldownOutputProbe')
                end
                debug.setobjecttaint(addon, 'CooldownOutputProbe')
                addon()
                assert(issecure())
            "#
            ))
            .unwrap();
            assert_numbers(&env, SPELL, true);
        }
    }

    #[test]
    fn tainted_arithmetic_denial_preserves_roots_and_recovery() {
        let env = fixture(true, true, false);
        for query in QUERIES {
            capture(&env, query);
            let original = read_wrappers(&env, "CDNumbers");
            env.exec(
                r#"
                local function addon()
                    for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                        local ok, err = pcall(function() return CDInfo[field] + 1 end)
                        assert(not ok)
                        assert(type(err) == 'string' and #err > 0)
                        assert(not string.find(err, '312', 1, true))
                        assert(not string.find(err, '237', 1, true))
                        assert(issecretvalue(CDInfo[field]))
                        assert(debug.getstacktaint() == 'CooldownOutputProbe')
                    end
                end
                debug.setobjecttaint(addon, 'CooldownOutputProbe')
                addon()
                assert(issecure())
            "#,
            )
            .unwrap();
            assert_eq!(read_wrappers(&env, "CDNumbers"), original);
            assert_numbers(&env, SPELL, true);
            check(&env, query, SPELL, true);
        }
    }

    #[test]
    fn tainted_copy_and_gc_keep_actual_wrapper_metadata_and_payloads() {
        let env = fixture(true, true, false);
        for query in QUERIES {
            capture(&env, query);
            let original = read_wrappers(&env, "CDNumbers");
            env.exec(
                r#"
                local function addon()
                    CDCopy = {}
                    for key, value in pairs(CDInfo) do CDCopy[key] = value end
                    assert(not issecretvalue(CDCopy))
                    for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                        assert(issecretvalue(CDCopy[field]))
                        assert(not canaccessvalue(CDCopy[field]))
                    end
                    CDCopyNumbers = {CDCopy.startTime, CDCopy.duration, CDCopy.modRate}
                    assert(debug.getstacktaint() == 'CooldownOutputProbe')
                end
                debug.setobjecttaint(addon, 'CooldownOutputProbe')
                addon()
                assert(issecure())
            "#,
            )
            .unwrap();
            env.exec(&format!(
                r#"
                for i = 1, 80 do
                    CDFresh = {query}
                    assert(CDFresh ~= CDInfo)
                    local garbage = {{}}
                    for j = 1, 40 do garbage[j] = {{i, j, tostring(i)}} end
                    if i % 8 == 0 then collectgarbage('collect') end
                end
                collectgarbage('collect')
            "#
            ))
            .unwrap();
            assert_eq!(read_wrappers(&env, "CDNumbers"), original);
            assert_eq!(read_wrappers(&env, "CDCopyNumbers"), original);
            assert_numbers(&env, SPELL, true);
            env.exec("CDInfo = CDFresh").unwrap();
            assert_numbers(&env, SPELL, true);
        }
    }

    #[test]
    fn flag_off_returns_public_snapshot_without_declassifying_old_roots() {
        let env = fixture(false, true, false);
        for query in QUERIES {
            env.state().borrow_mut().cooldowns_restricted = false;
            check(&env, query, SPELL, false);
            env.state().borrow_mut().cooldowns_restricted = true;
            check(&env, query, SPELL, true);
            let original = read_wrappers(&env, "CDNumbers");
            env.state().borrow_mut().cooldowns_restricted = false;
            env.exec(&format!("CDFresh = {query}\nassert(CDFresh ~= CDInfo)"))
                .unwrap();
            assert_eq!(read_wrappers(&env, "CDNumbers"), original);
            assert_numbers(&env, SPELL, true);
            env.exec("CDInfo = CDFresh").unwrap();
            assert_numbers(&env, SPELL, false);
        }
    }

    #[test]
    fn table_mutation_and_replacement_do_not_mutate_other_snapshots() {
        let env = fixture(true, true, false);
        for query in QUERIES {
            check(&env, query, SPELL, true);
            env.exec(&format!(
                r#"
                CDOther = {query}
                assert(CDOther ~= CDInfo)
                CDInfo.startTime = -99
                CDInfo.duration = 999
                CDInfo.modRate = 9
                CDInfo.isEnabled = false
                CDInfo.isActive = false
                CDInfo = {{duration = 123}}
                CDInfo = CDOther
            "#
            ))
            .unwrap();
            assert_numbers(&env, SPELL, true);
            assert_shape(&env, true);
            check(&env, query, SPELL, true);
        }
    }

    #[test]
    fn live_model_changes_and_independent_environments_remain_isolated() {
        let env = fixture(true, true, true);
        let independent = fixture(false, true, false);
        check_both(&env, GCD, true);
        env.exec("OldInfo = CDInfo\nOldNumbers = CDNumbers")
            .unwrap();
        let old_wrappers = read_wrappers(&env, "OldNumbers");
        env.state().borrow_mut().gcd = None;
        check_both(&env, SPELL, true);
        env.state().borrow_mut().spell_cooldowns.insert(
            19750,
            SpellCooldownState {
                start: 320.0,
                duration: 400.0,
            },
        );
        check_both(&env, (320.0, 400.0), true);
        expire(&env);
        check_both(&env, EMPTY, true);
        env.exec("CDInfo = OldInfo").unwrap();
        assert_numbers(&env, GCD, true);
        assert_eq!(read_wrappers(&env, "OldNumbers"), old_wrappers);
        check_both(&independent, SPELL, false);
        assert!(!independent.state().borrow().cooldowns_restricted);
    }

    #[test]
    fn queries_are_read_only_without_charge_or_action_fabrication() {
        let env = fixture(true, true, true);
        let (clock, actions, charges) = {
            let state = env.state().borrow();
            (
                state.start_time,
                state.action_bars.clone(),
                state.spell_charges.clone(),
            )
        };
        check_both(&env, GCD, true);
        let state = env.state().borrow();
        assert_eq!(state.start_time, clock);
        assert_eq!(state.action_bars, actions);
        assert_eq!(state.spell_charges, charges);
        assert_eq!(state.gcd, Some(GCD));
        assert_eq!(state.spell_cooldowns.len(), 1);
        let spell = state.spell_cooldowns.get(&19750).unwrap();
        assert_eq!((spell.start, spell.duration), SPELL);
        assert!(state.cooldowns_restricted);
    }

    #[test]
    fn book_invalid_missing_and_actual_offspec_slots_return_one_nil() {
        for restricted in [false, true] {
            let env = fixture(restricted, true, true);
            for slot in [
                "0",
                "-1",
                "5.5",
                "2147483647",
                "2147483648",
                "0/0",
                "math.huge",
                "nil",
            ] {
                assert_book_miss(&env, slot, "0");
            }
            env.exec(
                r#"
                OffSpecSlot = C_SpellBook.FindSpellBookSlotForSpell(20473)
                assert(type(OffSpecSlot) == 'number')
                local info = C_SpellBook.GetSpellBookItemInfo(OffSpecSlot, 0)
                assert(info.spellID == 20473 and info.isOffSpec == true)
            "#,
            )
            .expect("actual offspec identity, not a fabricated missing slot");
            assert_book_miss(&env, "OffSpecSlot", "0");
        }
    }

    #[test]
    fn book_other_or_missing_banks_never_fabricate_player_results() {
        for restricted in [false, true] {
            let env = fixture(restricted, true, true);
            for bank in ["1", "2", "-1", "0.5", "nil", "0/0", "math.huge"] {
                assert_book_miss(&env, "5", bank);
            }
        }
    }

    #[test]
    fn book_public_wrong_types_use_inferred_nil_not_new_type_errors() {
        let env = fixture(false, true, false);
        for value in ["'5'", "false", "{}", "function() end"] {
            assert_book_miss(&env, value, "0");
            assert_book_miss(&env, "5", value);
        }
    }

    #[test]
    fn book_secure_actual_secret_numeric_selectors_resolve_real_identity() {
        for restricted in [false, true] {
            let env = fixture(restricted, true, false);
            publish_secret(&env, "SecretSlot", 5.0);
            publish_secret(&env, "SecretBank", 0.0);
            for query in [
                "C_SpellBook.GetSpellBookItemCooldown(SecretSlot, 0)",
                "C_SpellBook.GetSpellBookItemCooldown(5, SecretBank)",
                "C_SpellBook.GetSpellBookItemCooldown(SecretSlot, SecretBank)",
            ] {
                check(&env, query, SPELL, restricted);
            }
        }
    }

    #[test]
    fn book_tainted_secret_selectors_deny_before_unknown_identity() {
        let env = fixture(false, true, false);
        publish_secret(&env, "SecretSlot", 5.0);
        publish_secret(&env, "SecretBank", 0.0);
        for (position, arguments) in [
            (1, "SecretSlot, 0"),
            (2, "5, SecretBank"),
            (1, "SecretSlot, 1"),
            (2, "2147483647, SecretBank"),
            (2, "nil, SecretBank"),
            (1, "SecretSlot, SecretBank"),
        ] {
            env.exec(&format!(
                r#"
                local function addon()
                    local ok, err = pcall(C_SpellBook.GetSpellBookItemCooldown, {arguments})
                    assert(not ok)
                    assert(type(err) == 'string' and #err > 0)
                    assert(string.find(err, 'untainted', 1, true))
                    assert(debug.getstacktaint() == 'CooldownSelectorProbe')
                end
                debug.setobjecttaint(addon, 'CooldownSelectorProbe')
                addon()
                assert(issecure())
            "#
            ))
            .unwrap_or_else(|error| panic!("GetSpellBookItemCooldown arg{position}: {error}"));
        }
        check_both(&env, SPELL, false);
    }

    #[test]
    fn spell_strict_u32_parser_and_existing_secret_rejection_are_preserved() {
        let env = fixture(false, true, false);
        publish_secret(&env, "SecretSpell", 19750.0);
        for argument in [
            "nil",
            "'19750'",
            "'Flash of Light'",
            "false",
            "{}",
            "-1",
            "19750.5",
            "4294967296",
            "0/0",
            "math.huge",
            "SecretSpell",
        ] {
            env.exec(&format!(
                r#"
                local ok, err = pcall(C_Spell.GetSpellCooldown, {argument})
                assert(not ok)
                assert(type(err) == 'string' and #err > 0)
                assert(issecure())
            "#
            ))
            .expect("output-only row 305 preserves current parser gap");
        }
        check(&env, "C_Spell.GetSpellCooldown(0)", EMPTY, false);
        check(&env, "C_Spell.GetSpellCooldown(4294967295)", EMPTY, false);
        check_both(&env, SPELL, false);
    }
}

#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
#[test]
fn earlier_profiles_keep_public_spell_and_disabled_four_field_book_payloads() {
    use std::time::{Duration, Instant};
    use wow_ui_sim::lua_api::WowLuaEnv;
    use wow_ui_sim::lua_api::state::SpellCooldownState;

    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.start_time = Instant::now() - Duration::from_secs(335);
        state.cooldowns_restricted = true;
        state.spell_cooldowns.clear();
        state.spell_cooldowns.insert(
            19750,
            SpellCooldownState {
                start: 312.0,
                duration: 237.0,
            },
        );
        state.gcd = None;
    }
    env.exec(
        r#"
        local spell = C_Spell.GetSpellCooldown(19750)
        assert(not issecretvalue(spell.startTime))
        assert(spell.startTime == 312 and spell.duration == 237)
        assert(spell.modRate == 1 and spell.isEnabled == true)
        assert(spell.isActive == true)
        local book = C_SpellBook.GetSpellBookItemCooldown(5, 0)
        assert(not issecretvalue(book.startTime))
        assert(book.startTime == 0 and book.duration == 0)
        assert(book.modRate == 1 and book.isEnabled == false)
        assert(book.isActive == nil)
        local count = 0
        for _ in pairs(book) do count = count + 1 end
        assert(count == 4)
    "#,
    )
    .expect("inverse epoch/profile gate keeps existing provider behavior");
}
