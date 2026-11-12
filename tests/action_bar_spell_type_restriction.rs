use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn empty_restriction_map_returns_one_true_for_a_valid_slot() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let (spell_count, spell_match, zero_spell_count, zero_spell_match): (i64, bool, i64, bool) =
        env.eval(
            r##"
            local matchSpell = C_ActionBar.SpellMatchesActionSpellTypeRestriction
            local function pack(...) return { count = select("#", ...), ... } end
            local spell = pack(matchSpell(853, 1))
            local zero = pack(matchSpell(0, 1))
            return spell.count, spell[1], zero.count, zero[1]
            "##,
        )
        .expect("the PTR action-bar matcher should return its result");

    assert_eq!((spell_count, spell_match), (1, true));
    assert_eq!((zero_spell_count, zero_spell_match), (1, true));
}

#[cfg(feature = "client-ptr")]
#[test]
fn invalid_converted_slots_return_no_values() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let (zero_slot_count, slot_409_count): (i64, i64) = env
        .eval(
            r##"
            local matchSpell = C_ActionBar.SpellMatchesActionSpellTypeRestriction
            local function resultCount(...) return select("#", ...) end
            return resultCount(matchSpell(853, 0)), resultCount(matchSpell(853, 409))
            "##,
        )
        .expect("out-of-range slots should not raise an argument error");

    assert_eq!((zero_slot_count, slot_409_count), (0, 0));
}

#[cfg(feature = "client-ptr")]
#[test]
fn arguments_use_verified_numeric_conversions_and_range_errors() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let (
        fractional_count,
        fractional_match,
        negative_action_ok,
        negative_action_error,
        oversized_spell_ok,
        oversized_spell_error,
        missing_spell_ok,
        missing_spell_error,
    ): (i64, bool, bool, String, bool, String, bool, String) = env
        .eval(
            r##"
            local matchSpell = C_ActionBar.SpellMatchesActionSpellTypeRestriction
            local function pack(...) return { count = select("#", ...), ... } end
            local fractional = pack(matchSpell("853", 0.25))
            local negativeActionOk, negativeActionError = pcall(matchSpell, 853, -1)
            local oversizedSpellOk, oversizedSpellError = pcall(matchSpell, 2147483648, 1)
            local missingSpellOk, missingSpellError = pcall(matchSpell, nil, 1)
            return fractional.count, fractional[1],
                   negativeActionOk, tostring(negativeActionError),
                   oversizedSpellOk, tostring(oversizedSpellError),
                   missingSpellOk, tostring(missingSpellError)
            "##,
        )
        .expect("argument conversion probes should run");

    assert_eq!((fractional_count, fractional_match), (1, true));
    assert!(!negative_action_ok);
    assert!(negative_action_error.contains("outside of expected range 0 to 4294967295"));
    assert!(!oversized_spell_ok);
    assert!(oversized_spell_error.contains("outside of expected range -2147483648 to 2147483647"));
    assert!(!missing_spell_ok);
    assert!(missing_spell_error.contains(
        "Usage: local actionMatchesSpellTypeRestriction = C_ActionBar.SpellMatchesActionSpellTypeRestriction(spellID, actionID)"
    ));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn ptr_only_matcher_is_not_published_on_retail_or_forever() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let (has_namespace, method_is_absent): (bool, bool) = env
        .eval(
            r##"
            local namespace = rawget(_G, "C_ActionBar")
            return type(namespace) == "table",
                   type(namespace) == "table"
                       and rawget(namespace, "SpellMatchesActionSpellTypeRestriction") == nil
            "##,
        )
        .expect("C_ActionBar publication probe should run");

    assert!(has_namespace);
    assert!(method_is_absent);
}
