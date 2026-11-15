//! R88m.4 API boundaries. Existing catalog rows are simulator inputs, not an
//! exhaustive 70688 Spell dataset. No native override/base links are invented.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    assert!(
        wow_ui_sim::spells::get_spell(6603).is_some(),
        "existing catalog row"
    );
    assert!(
        wow_ui_sim::spells::get_spell(i32::MAX as u32).is_none(),
        "absent catalog row"
    );
    assert!(
        wow_ui_sim::spells::get_spell(u32::MAX).is_none(),
        "absent signed -1 row"
    );
    assert!(
        wow_ui_sim::spells::get_spell(133).is_some(),
        "second existing catalog row"
    );
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        R88FindMethods = {
            {'FindSpellOverrideByID', 'overrideSpellID'},
            {'FindBaseSpellByID', 'baseSpellID'},
            {'FindFlyoutSlotBySpellID', 'flyoutSlot'},
        }
        function R88FindUsage(method)
            return 'Usage: local ' .. method[2] .. ' = C_SpellBook.' .. method[1] .. '(spellID)'
        end
        function R88OneNumber(expected, ...)
            assert(select('#', ...) == 1, 'exactly one numeric result')
            local value = ...
            assert(type(value) == 'number', 'result must be numeric, not input string')
            assert(value == expected, 'result must use converted/truncated signed ID')
        end
        function R88OneNil(...)
            assert(select('#', ...) == 1, 'missing record returns one nil, not zero results')
            assert((...) == nil, 'missing record must not return the input ID')
        end
    "#,
    )
    .unwrap();
    env
}

#[test]
fn find_record_results_normalize_numeric_strings_and_fractions_red() {
    let env = env();
    env.exec(r#"
        for _, method in ipairs({C_SpellBook.FindSpellOverrideByID, C_SpellBook.FindBaseSpellByID}) do
            R88OneNumber(6603, method(6603))
            R88OneNumber(6603, method('6603'))
            R88OneNumber(6603, method(6603.875))
            R88OneNumber(6603, method('6603.875', 'ignored extra argument'))
            R88OneNumber(133, method(133))
            R88OneNumber(133, method('133.875'))
            R88OneNil(method(2147483647))
        end
    "#).unwrap();
}

#[test]
fn find_missing_spell_record_returns_exactly_one_nil_red() {
    let env = env();
    env.exec(r#"
        for _, method in ipairs({C_SpellBook.FindSpellOverrideByID, C_SpellBook.FindBaseSpellByID}) do
            R88OneNil(method(2147483647))
            R88OneNil(method('2147483647'))
            R88OneNil(method(-1))
            R88OneNil(method('-1.875'))
            R88OneNumber(6603, method('6603.875'))
            R88OneNumber(133, method('133.875'))
        end
    "#).unwrap();
}

#[test]
fn find_required_signed_range_and_exact_error_reasons_preservation() {
    let env = env();
    env.exec(r#"
        for _, method in ipairs(R88FindMethods) do
            local api = C_SpellBook[method[1]]
            local usage = R88FindUsage(method)
            local prefix = "bad argument #1 to 'C_SpellBook." .. method[1] .. "' ("
            local function rejects(expected, ...)
                local ok, message = pcall(api, ...)
                assert(not ok, method[1] .. ' must reject input')
                assert(type(message) == 'string' and message:find(prefix .. expected .. ')', 1, true),
                    method[1] .. ' wrong diagnostic: ' .. tostring(message))
            end
            rejects(usage)
            rejects(usage, nil)
            local range = 'outside of expected range -2147483648 to 2147483647 - ' .. usage
            for _, bad in ipairs({true, false, {}, 'not a number', 2147483648, -2147483649,
                                  2147483647.5, -2147483648.5, '2147483648', math.huge, -math.huge}) do
                rejects(range, bad)
            end
            rejects(range, 0/0)
            for _, valid in ipairs({-2147483648, 2147483647, '-2147483648', '2147483647',
                                    -1.875, '-1.875', 0, '0'}) do
                local ok, message = pcall(api, valid)
                assert(ok, 'valid signed input rejected: ' .. tostring(message))
            end
        end
    "#).unwrap();
}

#[test]
fn flyout_no_candidate_returns_one_numeric_zero_preservation() {
    let env = env();
    assert!(env.state().borrow().spell_flyouts.is_empty());
    env.exec(
        r#"
        for _, input in ipairs({0, '0', 6603, '6603.875', -1, '-1.875', 2147483647}) do
            R88OneNumber(0, C_SpellBook.FindFlyoutSlotBySpellID(input))
        end
    "#,
    )
    .unwrap();
}

#[test]
fn find_authentic_secret_untainted_acceptance_and_tainted_rejection() {
    let env = env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let secret = rilua::table_security::wrap_host_secret_number(lua.state_mut(), 6603.0);
        lua.state_mut().push(secret);
        let result = lua.set_global_val("R88SecretSpell", secret);
        lua.state_mut().pop();
        result.unwrap();
    }
    env.exec(r#"
        assert(issecretvalue(R88SecretSpell), 'authentic VM secret required')
        assert(issecure(), 'untainted caller')
        for _, method in ipairs(R88FindMethods) do
            assert(pcall(C_SpellBook[method[1]], R88SecretSpell), 'untainted accessible secret accepted')
        end
        local function addon()
            assert(debug.getstacktaint() == 'R88m4FindFixture')
            for _, method in ipairs(R88FindMethods) do
                local ok, message = pcall(C_SpellBook[method[1]], R88SecretSpell)
                local expected = "bad argument #1 to 'C_SpellBook." .. method[1] .. "' (" ..
                    R88FindUsage(method) .. '. Secret values are only allowed during untainted execution for this argument.)'
                assert(not ok and type(message) == 'string' and message:find(expected, 1, true),
                    'tainted secret diagnostic: ' .. tostring(message))
                assert(debug.getstacktaint() == 'R88m4FindFixture', 'caller taint preserved')
                assert(issecretvalue(R88SecretSpell), 'secret not declassified')
            end
        end
        debug.setobjecttaint(addon, 'R88m4FindFixture')
        addon()
        assert(issecure(), 'outer caller restored')
    "#).unwrap();
    // No configurable native mode-0 reader exists in this harness. This tests
    // genuine VM accessible/tainted secrets, not fabricated mode-0 enforcement.
}
