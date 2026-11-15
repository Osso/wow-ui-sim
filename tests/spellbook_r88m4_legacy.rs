//! R88m.4 legacy boundary contracts; no call-pet records or selection cache invented.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn call_pet_missing_record_accepts_numeric_strings_and_fractions() {
    let env = WowLuaEnv::new().unwrap();
    assert!(wow_ui_sim::spells::get_spell(u32::MAX).is_none());
    env.exec(
        r##"
        for _, value in ipairs({-1, '-1', -1.9, '-1.9'}) do
            assert(select('#', GetCallPetSpellInfo(value, 'extra')) == 0)
        end
    "##,
    )
    .unwrap();
}

#[test]
fn call_pet_usage_and_checked_signed_overflow() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, args in ipairs({{}, {false}, {'bad'}, {{}}}) do
            local ok, message = pcall(GetCallPetSpellInfo, unpack(args))
            assert(not ok and message == 'Usage: GetCallPetSpellInfo(spellID)', tostring(message))
        end
        for _, value in ipairs({2147483648, '2147483648', -2147483649, '-2147483649', 2147483647.5, '-2147483648.5'}) do
            local ok, message = pcall(GetCallPetSpellInfo, value)
            local expected = string.format('integer overflow attempting to store %f', tonumber(value))
            assert(not ok and message == expected, tostring(message))
        end
    "#).unwrap();
}

#[test]
fn selected_spellbook_item_rejects_unaccepted_forms() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, args in ipairs({{}, {false}, {{}}}) do
            local ok, message = pcall(IsSelectedSpellBookItem, unpack(args))
            assert(not ok and message == 'IsSelectedSpellBookItem(): Invalid spell slot', tostring(message))
        end
        local ok, message = pcall(IsSelectedSpellBookItem, 2147483648, 'player')
        assert(not ok and message == 'integer overflow attempting to store 2147483648.000000', tostring(message))
    "#).unwrap();
}

#[test]
fn selected_spellbook_item_numeric_string_missing_slots_return_no_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        for _, bank in ipairs({'player', 'pet', 0}) do
            for _, slot in ipairs({0, '0', -1, '-1', 2147483647, '2147483647'}) do
                assert(select('#', IsSelectedSpellBookItem(slot, bank)) == 0, tostring(slot))
            end
        end
    "##,
    )
    .unwrap();
}
