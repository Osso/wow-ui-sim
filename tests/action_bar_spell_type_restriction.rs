#[cfg(feature = "client-ptr")]
use wow_ui_sim::lua_api::WowLuaEnv;

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
            local fractional = pack(matchSpell("853", 1.25))
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

    env.exec(
        r##"
        local matchSpell = C_ActionBar.SpellMatchesActionSpellTypeRestriction
        local invalid = {
            {-2147483649, 1},
            {853, 4294967296},
            {math.huge, 1},
            {-math.huge, 1},
            {0 / 0, 1},
            {853, math.huge},
            {853, -math.huge},
            {853, 0 / 0},
        }
        for _, args in ipairs(invalid) do
            local ok, err = pcall(matchSpell, args[1], args[2])
            assert(not ok)
            assert(string.find(tostring(err), 'outside of expected range', 1, true))
        end
        local missingActionOk, missingActionError = pcall(matchSpell, 853, nil)
        assert(not missingActionOk)
        assert(string.find(tostring(missingActionError), 'Usage: ', 1, true))
        assert(matchSpell(-2147483648, 1) == true)
        assert(matchSpell(2147483647, 1) == true)
        "##,
    )
    .expect("finite values and inclusive conversion bounds should match R69 readers");
}
