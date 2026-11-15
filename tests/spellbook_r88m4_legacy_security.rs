//! R88m.4 legacy boundaries over existing no-data seams, not native selection
//! predicates or a fabricated call-pet effect/record dataset.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;

fn authentic_secret_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    for (name, number) in [("R88CallPetSecret", -1.0), ("R88SelectionSecret", 1.0)] {
        let secret: Val = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(secret);
        let result = lua.set_global_val(name, secret);
        lua.state_mut().pop();
        result.unwrap();
    }
    drop(lua);
    drop(loader);
    env
}

#[test]
fn legacy_authentic_accessible_secret_numbers_use_existing_no_data_seams() {
    let env = authentic_secret_env();
    assert!(wow_ui_sim::spells::get_spell(u32::MAX).is_none());
    env.exec(
        r##"
        assert(issecure(), 'untainted access context')
        assert(issecretvalue(R88CallPetSecret) and issecretvalue(R88SelectionSecret))
        assert(select('#', GetCallPetSpellInfo(R88CallPetSecret)) == 0,
            'accessible secret negative ID has no supplied Spell record')
        local function oneFalse(...)
            assert(select('#', ...) == 1, 'represented player slot returns one result')
            assert(type((...)) == 'boolean' and (...) == false,
                'existing unselected seam only, not native selection policy')
        end
        oneFalse(IsSelectedSpellBookItem(R88SelectionSecret, 'player'))
        assert(issecretvalue(R88CallPetSecret) and issecretvalue(R88SelectionSecret),
            'arguments remain secret after readable access')
        assert(issecure(), 'caller remains untainted')
    "##,
    )
    .unwrap();
}

#[test]
fn legacy_authentic_tainted_secret_first_argument_uses_entry_errors() {
    let env = authentic_secret_env();
    env.exec(
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'R88m4LegacyFixture')
            assert(issecretvalue(R88CallPetSecret) and issecretvalue(R88SelectionSecret))
            local ok, message = pcall(GetCallPetSpellInfo, R88CallPetSecret)
            assert(not ok and message == 'Usage: GetCallPetSpellInfo(spellID)',
                'denied call-pet first secret must take entry Usage, not reader secret error')
            ok, message = pcall(IsSelectedSpellBookItem, R88SelectionSecret, 'player')
            assert(not ok and message == 'IsSelectedSpellBookItem(): Invalid spell slot',
                'denied selection first secret must take invalid-slot entry error')
            assert(debug.getstacktaint() == 'R88m4LegacyFixture', 'caller taint preserved')
            assert(issecretvalue(R88CallPetSecret) and issecretvalue(R88SelectionSecret),
                'rejected inputs not declassified')
        end
        assert(issecure())
        debug.setobjecttaint(addon, 'R88m4LegacyFixture')
        addon()
        assert(issecure(), 'outer caller restored')
    "#,
    )
    .unwrap();
    // This authentic VM harness exercises untainted/tainted accessibility, not
    // a configurable native mode-0 policy; no synthetic policy is substituted.
}

#[test]
fn selected_numeric_slot_existing_unselected_seam_preservation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        assert(C_SpellBook.GetSpellBookItemInfo(1, Enum.SpellBookSpellBank.Player),
            'actual modeled first player-book row required')
        local function oneFalse(...)
            assert(select('#', ...) == 1)
            assert(type((...)) == 'boolean' and (...) == false)
        end
        for _, slot in ipairs({1, '1', 1.875, '1.875'}) do
            for _, bank in ipairs({'player', 'not-pet', 0, 1}) do
                oneFalse(IsSelectedSpellBookItem(slot, bank))
            end
        end
    "##,
    )
    .unwrap();
}

#[test]
fn selected_failed_name_lookup_no_variant_store_returns_zero_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        local absent = '__R88m4_absent_name_variant__'
        assert(select('#', IsSelectedSpellBookItem(absent)) == 0)
        assert(select('#', IsSelectedSpellBookItem(absent, 'rank text')) == 0)
        assert(select('#', IsSelectedSpellBookItem(absent, '')) == 0)
        -- Numeric first input without a string-or-number second input takes
        -- name resolution, not a silently defaulted numeric player slot.
        assert(select('#', IsSelectedSpellBookItem(1)) == 0)
        assert(select('#', IsSelectedSpellBookItem('1', false)) == 0)
    "##,
    )
    .unwrap();
}
