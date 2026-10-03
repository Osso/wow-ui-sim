//! Batch69 exact rows313/326. Simulator policies marked INFERRED in the spec.
//! Written 2026-10-02 CDT; main owns compilation/RED. No native parity claim.
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod current {
    use rilua::{LuaApi, LuaApiMut, Val};
    use wow_ui_sim::lua_api::{LossOfControlInfo, WowLuaEnv};

    type Payload = (f64, f64, f32, bool, bool);
    const FIRST: Payload = (312.0, 237.0, 1.25, true, true);
    const SECOND: Payload = (11.0, 27.0, 0.5, true, false);
    const EMPTY: Payload = (0.0, 0.0, 1.0, false, false);
    const FIELDS: [&str; 3] = ["startTime", "duration", "modRate"];
    const SPELL: &str = "C_Spell.GetSpellLossOfControlCooldownInfo(19750)";
    const BOOK: &str = "C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(5, 0)";
    const QUERIES: [&str; 2] = [SPELL, BOOK];

    fn record(value: Payload) -> LossOfControlInfo {
        LossOfControlInfo {
            start_time: value.0,
            duration: value.1,
            mod_rate: value.2,
            is_active: value.3,
            should_replace_normal_cooldown: value.4,
        }
    }

    fn fixture(restricted: bool) -> WowLuaEnv {
        let env = WowLuaEnv::new().expect("rows313/326 environment");
        {
            let mut state = env.state().borrow_mut();
            state.cooldowns_restricted = restricted;
            state.spell_loss_of_control.clear();
            state.spell_loss_of_control.insert(19750, record(FIRST));
            state.spell_loss_of_control.insert(642, record(SECOND));
        }
        env.exec("assert(C_SpellBook.GetSpellBookItemInfo(5, 0).spellID == 19750)")
            .expect("actual player-bank slot identity");
        env
    }

    fn capture(env: &WowLuaEnv, query: &str) {
        env.exec(&format!(
            r#"
            assert(select('#', {query}) == 1)
            Info = {query}
            assert(type(Info) == 'table' and getmetatable(Info) == nil)
            assert(not issecretvalue(Info) and canaccessvalue(Info))
            Numbers = {{Info.startTime, Info.duration, Info.modRate}}
            "#
        ))
        .expect("one fresh ordinary rooted DTO");
    }

    fn assert_payload(env: &WowLuaEnv, expected: Payload, restricted: bool) {
        for (field, number) in
            FIELDS
                .into_iter()
                .zip([expected.0, expected.1, f64::from(expected.2)])
        {
            let value: Val = env.eval(&format!("return Info.{field}")).unwrap();
            let lua = env.lua();
            assert!(rilua::api::state_is_secure(lua.state()));
            let payload = rilua::table_security::unwrap_secret(lua.state(), value)
                .expect("trusted host authenticates without declassification");
            assert_eq!(payload, Val::Num(number), "{field} model payload");
            assert_eq!(
                rilua::table_security::is_secret_value(lua.state(), value),
                restricted
            );
            drop(lua);
            env.exec(&format!(
                "assert(issecretvalue(Info.{field}) == {restricted})\n\
                 if not {restricted} then assert(type(Info.{field}) == 'number') end"
            ))
            .unwrap();
        }
        env.exec(&format!(
            r#"
            assert(type(Info.isActive) == 'boolean')
            assert(not issecretvalue(Info.isActive))
            assert(Info.isActive == {})
            assert(type(Info.shouldReplaceNormalCooldown) == 'boolean')
            assert(not issecretvalue(Info.shouldReplaceNormalCooldown))
            assert(Info.shouldReplaceNormalCooldown == {})
            local count = 0
            for _ in pairs(Info) do count = count + 1 end
            assert(count == 5)
            "#,
            expected.3, expected.4
        ))
        .expect("exact five fields; NeverSecret booleans");
    }

    fn check(env: &WowLuaEnv, query: &str, expected: Payload, restricted: bool) {
        capture(env, query);
        assert_payload(env, expected, restricted);
    }

    fn check_both(env: &WowLuaEnv, expected: Payload, restricted: bool) {
        for query in QUERIES {
            check(env, query, expected, restricted);
        }
    }

    fn assert_nil(env: &WowLuaEnv, query: &str) {
        env.exec(&format!(
            "assert(select('#', {query}) == 1)\nassert({query} == nil)"
        ))
        .expect("one nil, not an inactive fabricated DTO");
    }

    fn book_miss(env: &WowLuaEnv, slot: &str, bank: &str) {
        assert_nil(
            env,
            &format!("C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo({slot}, {bank})"),
        );
    }

    fn publish_secret(env: &WowLuaEnv, name: &str, number: f64) {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        assert!(rilua::api::state_is_secure(lua.state_mut()));
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.set_global_val(name, value)
            .expect("root genuine numeric selector");
    }

    fn wrappers(env: &WowLuaEnv, root: &str) -> Vec<(Val, u64)> {
        (1..=3)
            .map(|index| {
                let value: Val = env.eval(&format!("return {root}[{index}]")).unwrap();
                let lua = env.lua();
                assert!(rilua::api::state_is_secure(lua.state()));
                assert!(rilua::table_security::is_secret_value(lua.state(), value));
                let Val::Userdata(reference) = value else {
                    panic!("genuine wrapper missing")
                };
                let userdata = lua
                    .state()
                    .gc
                    .userdata
                    .get(reference)
                    .expect("live rooted wrapper");
                assert!(matches!(
                    rilua::table_security::unwrap_secret(lua.state(), value).unwrap(),
                    Val::Num(_)
                ));
                (value, userdata.alloc_seq())
            })
            .collect()
    }

    fn addon(env: &WowLuaEnv, body: &str) {
        env.exec(&format!(
            r#"
            assert(issecure())
            local function probe()
                assert(debug.getstacktaint() == 'BookLoCProbe')
                {body}
                assert(debug.getstacktaint() == 'BookLoCProbe')
            end
            debug.setobjecttaint(probe, 'BookLoCProbe')
            probe()
            assert(issecure())
            "#
        ))
        .expect("actual tainted closure and secure recovery");
    }

    #[test]
    fn spell_public_first_record_is_verbatim() {
        check(&fixture(false), SPELL, FIRST, false);
    }

    #[test]
    fn spell_public_second_record_keeps_false_replacement() {
        check(
            &fixture(false),
            "C_Spell.GetSpellLossOfControlCooldownInfo(642)",
            SECOND,
            false,
        );
    }

    #[test]
    fn spell_explicit_inactive_record_is_not_absence() {
        let env = fixture(false);
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(19750, record(EMPTY));
        check(&env, SPELL, EMPTY, false);
    }

    #[test]
    fn spell_absent_map_returns_one_nil_under_both_flags() {
        for restricted in [false, true] {
            let env = fixture(restricted);
            env.state()
                .borrow_mut()
                .spell_loss_of_control
                .remove(&19750);
            assert_nil(&env, SPELL);
            assert_nil(
                &env,
                "C_Spell.GetSpellLossOfControlCooldownInfo(4294967295)",
            );
        }
    }

    #[test]
    fn spell_existing_permissive_public_identifier_behavior_is_preserved() {
        let env = fixture(false);
        for selector in ["19750.75", "'19750'", "'fLaSh Of LiGhT'"] {
            check(
                &env,
                &format!("C_Spell.GetSpellLossOfControlCooldownInfo({selector})"),
                FIRST,
                false,
            );
        }
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(u32::MAX, record(SECOND));
        check(
            &env,
            "C_Spell.GetSpellLossOfControlCooldownInfo(4294967296)",
            SECOND,
            false,
        );
        for selector in [
            "nil",
            "false",
            "{}",
            "function() end",
            "-1",
            "0/0",
            "math.huge",
            "'not-a-spell'",
            "0",
        ] {
            assert_nil(
                &env,
                &format!("C_Spell.GetSpellLossOfControlCooldownInfo({selector})"),
            );
        }
    }

    #[test]
    fn spell_secret_userdata_stays_unmodeled_nil_for_secure_and_addon_callers() {
        let env = fixture(true);
        publish_secret(&env, "SecretSpell", 19750.0);
        assert_nil(
            &env,
            "C_Spell.GetSpellLossOfControlCooldownInfo(SecretSpell)",
        );
        addon(
            &env,
            "assert(C_Spell.GetSpellLossOfControlCooldownInfo(SecretSpell) == nil)\nassert(issecretvalue(SecretSpell))",
        );
        check(&env, SPELL, FIRST, true);
    }

    #[test]
    fn book_public_slot_five_selects_spell_record() {
        check(&fixture(false), BOOK, FIRST, false);
    }

    #[test]
    fn book_discovered_alternate_slot_selects_actual_642_record() {
        for restricted in [false, true] {
            let env = fixture(restricted);
            env.exec(
                r#"
                AlternateSlot = C_SpellBook.FindSpellBookSlotForSpell(642)
                assert(type(AlternateSlot) == 'number')
                assert(C_SpellBook.GetSpellBookItemInfo(AlternateSlot, 0).spellID == 642)
            "#,
            )
            .unwrap();
            check(
                &env,
                "C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(AlternateSlot, 0)",
                SECOND,
                restricted,
            );
        }
    }

    #[test]
    fn book_explicit_inactive_record_is_not_absence() {
        let env = fixture(true);
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(19750, record(EMPTY));
        check(&env, BOOK, EMPTY, true);
    }

    #[test]
    fn book_valid_identity_absent_map_returns_inferred_nil() {
        for restricted in [false, true] {
            let env = fixture(restricted);
            env.state()
                .borrow_mut()
                .spell_loss_of_control
                .remove(&19750);
            assert_nil(&env, BOOK);
        }
    }

    #[test]
    fn book_actual_offspec_20473_is_inferred_miss_even_with_record() {
        let env = fixture(false);
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(20473, record(FIRST));
        env.exec(
            r#"
            OffSpecSlot = C_SpellBook.FindSpellBookSlotForSpell(20473)
            assert(type(OffSpecSlot) == 'number')
            local info = C_SpellBook.GetSpellBookItemInfo(OffSpecSlot, 0)
            assert(info.spellID == 20473 and info.isOffSpec == true)
        "#,
        )
        .unwrap();
        book_miss(&env, "OffSpecSlot", "0");
    }

    #[test]
    fn book_invalid_or_unresolved_public_slot_is_inferred_nil() {
        let env = fixture(false);
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
            book_miss(&env, slot, "0");
        }
    }

    #[test]
    fn book_nonplayer_or_missing_public_bank_is_inferred_nil() {
        let env = fixture(false);
        for bank in ["1", "2", "-1", "0.5", "nil", "0/0", "math.huge"] {
            book_miss(&env, "5", bank);
        }
    }

    #[test]
    fn book_wrong_public_selector_types_are_inferred_nil() {
        let env = fixture(false);
        for value in ["'5'", "false", "{}", "function() end"] {
            book_miss(&env, value, "0");
            book_miss(&env, "5", value);
        }
    }

    #[test]
    fn book_secure_genuine_numeric_slot_and_bank_resolve_before_type_checks() {
        for restricted in [false, true] {
            let env = fixture(restricted);
            publish_secret(&env, "SecretSlot", 5.0);
            publish_secret(&env, "SecretBank", 0.0);
            for args in ["SecretSlot, 0", "5, SecretBank", "SecretSlot, SecretBank"] {
                check(
                    &env,
                    &format!("C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo({args})"),
                    FIRST,
                    restricted,
                );
            }
        }
    }

    #[test]
    fn book_secure_authenticated_invalid_numeric_selectors_are_inferred_nil() {
        let env = fixture(false);
        for number in [0.0, -1.0, 5.5, f64::NAN, f64::INFINITY, 2147483648.0] {
            publish_secret(&env, "SecretSlot", number);
            book_miss(&env, "SecretSlot", "0");
        }
        for number in [1.0, -1.0, 0.5, f64::NAN, f64::INFINITY] {
            publish_secret(&env, "SecretBank", number);
            book_miss(&env, "5", "SecretBank");
        }
    }

    #[test]
    fn addon_public_selectors_are_allowed_without_changing_taint_or_flags() {
        for restricted in [false, true] {
            let env = fixture(restricted);
            for query in QUERIES {
                addon(
                    &env,
                    &format!(
                        r#"
                    Info = {query}
                    assert(type(Info) == 'table' and not issecretvalue(Info))
                    for _, field in ipairs({{'startTime', 'duration', 'modRate'}}) do
                        assert(issecretvalue(Info[field]) == {restricted})
                        assert(canaccessvalue(Info[field]) == (not {restricted}))
                    end
                    assert(Info.isActive == true and not issecretvalue(Info.isActive))
                    assert(Info.shouldReplaceNormalCooldown == true)
                    assert(not issecretvalue(Info.shouldReplaceNormalCooldown))
                "#
                    ),
                );
                assert_payload(&env, FIRST, restricted);
            }
        }
    }

    #[test]
    fn addon_genuine_book_secrets_deny_before_other_invalid_or_unresolved_selector() {
        let env = fixture(false);
        // Secret payload validity must not bypass authentication under addon taint.
        for (slot, bank) in [(5.0, 0.0), (0.0, 1.0)] {
            publish_secret(&env, "SecretSlot", slot);
            publish_secret(&env, "SecretBank", bank);
            for args in [
                "SecretSlot, 0",
                "5, SecretBank",
                "SecretSlot, SecretBank",
                "SecretSlot, 1",
                "SecretSlot, {}",
                "2147483647, SecretBank",
                "nil, SecretBank",
                "{}, SecretBank",
            ] {
                addon(
                    &env,
                    &format!(
                        r#"
                local ok, err = pcall(C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo, {args})
                assert(not ok and type(err) == 'string' and #err > 0)
                assert(string.find(err, 'untainted', 1, true))
                assert(string.find(err, 'C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo', 1, true))
                assert(issecretvalue(SecretSlot) and issecretvalue(SecretBank))
            "#
                    ),
                );
            }
        }
        check_both(&env, FIRST, false);
    }

    #[test]
    fn both_namespaces_copy_explicit_flags_not_interval_derived_activity() {
        for restricted in [false, true] {
            let env = fixture(restricted);
            for flags in [(false, true), (true, false), (false, false), (true, true)] {
                let input = (0.0, 0.0, 1.25, flags.0, flags.1);
                env.state()
                    .borrow_mut()
                    .spell_loss_of_control
                    .insert(19750, record(input));
                check_both(&env, input, restricted);
            }
        }
    }

    #[test]
    fn explicit_cooldown_flag_alone_controls_numeric_privacy() {
        let env = fixture(false);
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
            check_both(&env, FIRST, restricted);
            check(
                &env,
                "C_Spell.GetSpellLossOfControlCooldownInfo(642)",
                SECOND,
                restricted,
            );
        }
    }

    #[test]
    fn live_record_replacement_does_not_recompute_or_expire_payload() {
        let env = fixture(true);
        check_both(&env, FIRST, true);
        let changed = (1.0, 2.0, 0.5, true, false);
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(19750, record(changed));
        check_both(&env, changed, true);
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .remove(&19750);
        for query in QUERIES {
            assert_nil(&env, query);
        }
    }

    #[test]
    fn dto_mutation_and_replacement_leave_other_snapshots_unchanged() {
        let env = fixture(true);
        for query in QUERIES {
            check(&env, query, FIRST, true);
            env.exec(&format!(
                r#"
                Other = {query}
                assert(Other ~= Info)
                Info.startTime = -99
                Info.duration = 999
                Info.modRate = 9
                Info.isActive = false
                Info.shouldReplaceNormalCooldown = false
                Info = {{duration = 123}}
                Info = Other
            "#
            ))
            .unwrap();
            assert_payload(&env, FIRST, true);
            check(&env, query, FIRST, true);
        }
    }

    #[test]
    fn queries_do_not_change_existing_input_maps_or_clock() {
        let env = fixture(true);
        let (clock, actions, charges, gcd) = {
            let state = env.state().borrow();
            (
                state.start_time,
                state.action_bars.clone(),
                state.spell_charges.clone(),
                state.gcd,
            )
        };
        check_both(&env, FIRST, true);
        let state = env.state().borrow();
        assert_eq!(state.start_time, clock);
        assert_eq!(state.action_bars, actions);
        assert_eq!(state.spell_charges, charges);
        assert_eq!(state.gcd, gcd);
        assert_eq!(state.spell_loss_of_control.len(), 2);
        for (id, expected) in [(19750, FIRST), (642, SECOND)] {
            let input = state.spell_loss_of_control.get(&id).unwrap();
            assert_eq!(
                (
                    input.start_time,
                    input.duration,
                    input.mod_rate,
                    input.is_active,
                    input.should_replace_normal_cooldown
                ),
                expected
            );
        }
    }

    #[test]
    fn independent_environments_do_not_share_records_or_policy() {
        let first = fixture(true);
        let second = fixture(false);
        first
            .state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(19750, record(EMPTY));
        check_both(&first, EMPTY, true);
        check_both(&second, FIRST, false);
    }

    #[test]
    fn flag_off_returns_fresh_public_dto_without_declassifying_old_roots() {
        let env = fixture(true);
        for query in QUERIES {
            env.state().borrow_mut().cooldowns_restricted = true;
            check(&env, query, FIRST, true);
            let original = wrappers(&env, "Numbers");
            env.state().borrow_mut().cooldowns_restricted = false;
            env.exec(&format!("Fresh = {query}\nassert(Fresh ~= Info)"))
                .unwrap();
            assert_eq!(wrappers(&env, "Numbers"), original);
            assert_payload(&env, FIRST, true);
            addon(
                &env,
                "assert(issecretvalue(Info.duration) and not canaccessvalue(Info.duration))",
            );
            env.exec("Info = Fresh").unwrap();
            assert_payload(&env, FIRST, false);
        }
    }

    #[test]
    fn addon_arithmetic_denial_preserves_wrapper_identity_and_recovery() {
        let env = fixture(true);
        for query in QUERIES {
            check(&env, query, FIRST, true);
            let original = wrappers(&env, "Numbers");
            addon(
                &env,
                r#"
                for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                    local ok, err = pcall(function() return Info[field] + 1 end)
                    assert(not ok and type(err) == 'string' and #err > 0)
                    for _, payload in ipairs({'312', '237', '1.25'}) do
                        assert(not string.find(err, payload, 1, true))
                    end
                    assert(debug.getstacktaint() == 'BookLoCProbe')
                end
            "#,
            );
            assert_eq!(wrappers(&env, "Numbers"), original);
            assert_payload(&env, FIRST, true);
            check(&env, query, FIRST, true);
        }
    }

    #[test]
    fn addon_copies_and_forced_gc_retain_authentic_rooted_numeric_wrappers() {
        let env = fixture(true);
        for query in QUERIES {
            check(&env, query, FIRST, true);
            let original = wrappers(&env, "Numbers");
            addon(
                &env,
                r#"
                Copy = {}
                for key, value in pairs(Info) do Copy[key] = value end
                assert(not issecretvalue(Copy))
                for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                    assert(issecretvalue(Copy[field]) and not canaccessvalue(Copy[field]))
                end
                CopyNumbers = {Copy.startTime, Copy.duration, Copy.modRate}
            "#,
            );
            env.exec(&format!(
                r#"
                for i = 1, 80 do
                    Fresh = {query}
                    assert(Fresh ~= Info)
                    local garbage = {{}}
                    for j = 1, 40 do garbage[j] = {{i, j, tostring(i)}} end
                    if i % 8 == 0 then collectgarbage('collect') end
                end
                collectgarbage('collect')
            "#
            ))
            .unwrap();
            assert_eq!(wrappers(&env, "Numbers"), original);
            assert_eq!(wrappers(&env, "CopyNumbers"), original);
            assert_payload(&env, FIRST, true);
            env.exec("Info = Copy").unwrap();
            assert_payload(&env, FIRST, true);
            env.exec("Info = Fresh").unwrap();
            assert_payload(&env, FIRST, true);
        }
    }
}

#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
#[test]
fn earlier_profiles_keep_public_spell_and_disabled_book_loc_payloads() {
    use wow_ui_sim::lua_api::{LossOfControlInfo, WowLuaEnv};
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = true;
        state.spell_loss_of_control.insert(
            19750,
            LossOfControlInfo {
                start_time: 312.0,
                duration: 237.0,
                mod_rate: 1.25,
                is_active: true,
                should_replace_normal_cooldown: true,
            },
        );
    }
    env.exec(
        r#"
        local spell = C_Spell.GetSpellLossOfControlCooldownInfo(19750)
        assert(spell.startTime == 312 and spell.duration == 237 and spell.modRate == 1.25)
        assert(spell.isActive == true and spell.shouldReplaceNormalCooldown == true)
        local book = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(5, 0)
        assert(book.startTime == 0 and book.duration == 0 and book.modRate == 1)
        assert(book.isActive == false and book.shouldReplaceNormalCooldown == false)
        for _, info in ipairs({spell, book}) do
            local count = 0
            for _, value in pairs(info) do
                assert(not issecretvalue(value))
                count = count + 1
            end
            assert(count == 5)
        end
    "#,
    )
    .expect("inverse epoch/profile control preserves previous LoC providers");
}
