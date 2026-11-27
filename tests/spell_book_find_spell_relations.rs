//! C_SpellBook override, base-spell and flyout lookups without native relation
//! links (R88m.4, R88j.10). Retail and Forever declare the three
//! SpellBookDocumentation.lua blocks byte-identically.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const LOOKUPS: &str = r##"
    local function capture(...) return { n = select("#", ...), ... } end
    local function describe(result)
        return result.n .. ":" .. tostring(result[1])
    end
    local out = {}
    for _, spellID in ipairs({ 19750, 642, 999999999 }) do
        out[#out + 1] = describe(capture(C_SpellBook.FindSpellOverrideByID(spellID)))
        out[#out + 1] = describe(capture(C_SpellBook.FindBaseSpellByID(spellID)))
        out[#out + 1] = describe(capture(C_SpellBook.FindFlyoutSlotBySpellID(spellID)))
    end
    return table.concat(out, " ")
"##;

#[test]
fn spell_relation_lookups_keep_known_ids_and_reject_unknown_ones() {
    let env = WowLuaEnv::new().unwrap();
    let observed: String = env.eval(LOOKUPS).unwrap();
    assert_eq!(
        observed,
        "1:19750 1:19750 1:0 1:642 1:642 1:0 1:nil 1:nil 1:0",
        "known spells keep their ID, unknown ones return nil, flyouts miss with 0"
    );
}

#[test]
fn spell_relation_lookups_require_a_spell_id() {
    let env = WowLuaEnv::new().unwrap();
    let rejected: (bool, bool, bool) = env
        .eval(
            "return not pcall(C_SpellBook.FindSpellOverrideByID), \
             not pcall(C_SpellBook.FindBaseSpellByID, {}), \
             not pcall(C_SpellBook.FindFlyoutSlotBySpellID, 'not-a-number')",
        )
        .unwrap();
    assert_eq!(rejected, (true, true, true));
}
