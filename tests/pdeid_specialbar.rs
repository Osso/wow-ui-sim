#![cfg(feature = "retail-12-0-5")]
//! INFERRED host-state policies; no native AllowedWhenTainted permission credit.

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create host-state environment");
    env.exec(
        r#"
        function CheckPDEID(expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one PDEID result')
                local value = ...
                assert(type(value) == 'number' and not issecretvalue(value))
                assert(value == expected, 'live host PDEID')
            end
            check(C_DelvesUI.GetTieredEntrancePDEID())
        end
        function CheckSpecial(identifier, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one membership result')
                local value = ...
                assert(type(value) == 'boolean' and not issecretvalue(value))
                assert(value == expected, 'direct or explicit special membership')
            end
            check(C_ActionBar.IsOnBarOrSpecialBar(identifier))
        end
        "#,
    )
    .expect("install assertions without replacing APIs");
    env
}

fn empty_membership_env() -> WowLuaEnv {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.action_bars.clear();
        state.action_macros.clear();
        state.action_outfits.clear();
        state.spell_id_aliases.clear();
    }
    env
}

#[test]
fn pdeid_default_is_exactly_one_public_numeric_zero() {
    let env = fixture_env();
    assert_eq!(env.state().borrow().tiered_entrance_pde_id, 0);
    env.exec("CheckPDEID(0)").expect("INFERRED zero default");
}

#[test]
fn pdeid_reads_live_scalar_including_zero_and_storage_boundary() {
    let env = fixture_env();
    for pde_id in [77012, 88021, 0, u32::MAX] {
        env.state().borrow_mut().tiered_entrance_pde_id = pde_id;
        env.exec(&format!("CheckPDEID({pde_id})"))
            .expect("one numeric result for each live host value");
        assert_eq!(env.state().borrow().tiered_entrance_pde_id, pde_id);
    }
}

#[test]
fn pdeid_updates_are_environment_local() {
    let first = fixture_env();
    let second = fixture_env();
    first.state().borrow_mut().tiered_entrance_pde_id = 88021;
    first.exec("CheckPDEID(88021)").unwrap();
    second.exec("CheckPDEID(0)").unwrap();
    second.state().borrow_mut().tiered_entrance_pde_id = 77012;
    first.exec("CheckPDEID(88021)").unwrap();
    second.exec("CheckPDEID(77012)").unwrap();
}

#[test]
fn special_membership_is_live_union_not_direct_slot_assignment() {
    let env = empty_membership_env();
    assert!(env.state().borrow().special_bar_spells.is_empty());
    env.exec("CheckSpecial(7001, false)").unwrap();
    env.state().borrow_mut().special_bar_spells.insert(7001);
    env.exec(
        r#"
        CheckSpecial(7001, true)
        CheckSpecial(7002, false)
        assert(C_ActionBar.HasSpellActionButtons(7001) == false)
        assert(#C_ActionBar.FindSpellActionButtons(7001) == 0)
        "#,
    )
    .expect("special-only membership leaves direct queries unchanged");
    env.state().borrow_mut().action_bars.insert(3, 7001);
    env.exec("CheckSpecial(7001, true); assert(C_ActionBar.HasSpellActionButtons(7001))")
        .unwrap();
    env.state().borrow_mut().special_bar_spells.remove(&7001);
    env.exec("CheckSpecial(7001, true)").unwrap();
    env.state().borrow_mut().action_bars.remove(&3);
    env.exec("CheckSpecial(7001, false)").unwrap();
    env.state().borrow_mut().special_bar_spells.insert(7002);
    env.exec("CheckSpecial(7001, false); CheckSpecial(7002, true)")
        .unwrap();
    env.state().borrow_mut().special_bar_spells.clear();
    env.exec("CheckSpecial(7002, false)").unwrap();
}

#[test]
fn special_membership_uses_live_alias_resolution_not_link_grammar() {
    let env = empty_membership_env();
    {
        let mut state = env.state().borrow_mut();
        state.special_bar_spells.insert(7001);
        state
            .spell_id_aliases
            .insert("fixture special".into(), 7001);
        state.spell_id_aliases.insert("9001".into(), 7001);
        state
            .spell_id_aliases
            .insert("|hspell:9001|h[special]|h".into(), 7001);
    }
    env.exec(
        r#"
        CheckSpecial('FIXTURE SPECIAL', true)
        CheckSpecial(9001, true)
        CheckSpecial('9001', true)
        CheckSpecial('|Hspell:9001|h[Special]|h', true)
        CheckSpecial('|Hspell:7001|h[Unregistered]|h', false)
        CheckSpecial('unknown', false)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture special".into(), 7002);
    env.exec("CheckSpecial('fixture special', false)").unwrap();
    env.state().borrow_mut().special_bar_spells.insert(7002);
    env.exec("CheckSpecial('fixture special', true)").unwrap();
    env.state().borrow_mut().spell_id_aliases.clear();
    env.exec("CheckSpecial('fixture special', false); CheckSpecial(9001, false); CheckSpecial(7001, true)")
        .unwrap();
}

#[test]
fn special_membership_survives_direct_macro_and_outfit_shadows() {
    let env = empty_membership_env();
    {
        let mut state = env.state().borrow_mut();
        state.action_bars.extend([(3, 7001), (5, 7002), (0, 7003)]);
        state.action_macros.insert(3, 78);
        state.action_outfits.insert(3, 900);
        state.special_bar_spells.insert(7001);
    }
    env.exec(
        r#"
        assert(GetActionInfo(3) == 'outfit')
        CheckSpecial(7001, true)
        CheckSpecial(7002, true)
        CheckSpecial(7003, false)
        assert(not C_ActionBar.HasSpellActionButtons(7001))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().special_bar_spells.clear();
    env.exec("CheckSpecial(7001, false)").unwrap();
    env.state().borrow_mut().action_outfits.remove(&3);
    env.exec("assert(GetActionInfo(3) == 'macro'); CheckSpecial(7001, false)")
        .unwrap();
    env.state().borrow_mut().action_macros.remove(&3);
    env.exec("assert(GetActionInfo(3) == 'spell'); CheckSpecial(7001, true)")
        .unwrap();
}

#[test]
fn special_membership_queries_are_read_only_and_environment_local() {
    let first = empty_membership_env();
    let second = empty_membership_env();
    first.state().borrow_mut().special_bar_spells.insert(7001);
    first
        .exec("CheckSpecial(7001, true); CheckSpecial('unknown', false)")
        .unwrap();
    second.exec("CheckSpecial(7001, false)").unwrap();
    {
        let state = first.state().borrow();
        assert_eq!(
            state.special_bar_spells.iter().copied().collect::<Vec<_>>(),
            vec![7001]
        );
        assert!(state.action_bars.is_empty());
        assert!(state.action_macros.is_empty());
        assert!(state.action_outfits.is_empty());
        assert!(state.spell_id_aliases.is_empty());
    }
    second.state().borrow_mut().special_bar_spells.insert(7002);
    first.state().borrow_mut().special_bar_spells.clear();
    first
        .exec("CheckSpecial(7001, false); CheckSpecial(7002, false)")
        .unwrap();
    second
        .exec("CheckSpecial(7001, false); CheckSpecial(7002, true)")
        .unwrap();
}

#[test]
fn special_membership_preserves_public_validation_and_unmodeled_secret_boundary() {
    let env = empty_membership_env();
    env.state().borrow_mut().special_bar_spells.insert(7001);
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let number = wrap_host_secret_number(lua.state_mut(), 7001.0);
        lua.state_mut().push(number);
        let inserted = lua.set_global_val("SecretSpecialNumber", number);
        lua.state_mut().pop();
        inserted.unwrap();
        let text = wrap_host_secret_string(lua.state_mut(), "fixture special");
        lua.state_mut().push(text);
        let inserted = lua.set_global_val("SecretSpecialString", text);
        lua.state_mut().pop();
        inserted.unwrap();
    }
    env.exec(
        r#"
        -- Conservative existing boundary, NOT native AllowedWhenTainted behavior.
        local query = C_ActionBar.IsOnBarOrSpecialBar
        assert(not pcall(query))
        for _, value in ipairs({false, {}, -1, 1.5, math.huge, 4294967296}) do
            assert(not pcall(query, value))
        end
        assert(not pcall(query, 0/0))
        local function secretsRemainRejected()
            assert(not pcall(query, SecretSpecialNumber))
            assert(not pcall(query, SecretSpecialString))
            assert(issecretvalue(SecretSpecialNumber) and issecretvalue(SecretSpecialString))
            CheckSpecial(7001, true)
        end
        assert(debug.getstacktaint() == nil)
        secretsRemainRejected()
        local function taintedProbe()
            assert(debug.getstacktaint() == 'SpecialMembershipProbe')
            secretsRemainRejected()
            assert(not pcall(secretunwrap, SecretSpecialNumber))
            assert(debug.getstacktaint() == 'SpecialMembershipProbe')
        end
        debug.setobjecttaint(taintedProbe, 'SpecialMembershipProbe')
        taintedProbe()
        assert(debug.getstacktaint() == nil)
        collectgarbage('collect')
        secretsRemainRejected()
        assert(secretunwrap(SecretSpecialNumber) == 7001)
        "#,
    )
    .expect("unchanged public boundary, rooted secret wrappers and caller taint");
    assert!(env.state().borrow().special_bar_spells.contains(&7001));
}
