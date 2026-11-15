//! R88m.4 VERIFIED public arity/validation, over existing simulator book state.
//! Supplied decoded cooldown DTOs and off-spec rejection are PRESERVATION;
//! neither is evidence of native acquisition, row-flag identity or event order.
//! Original runtime-selected consumer coverage is separate and labelled PRESERVATION.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use wow_ui_sim::lua_api::globals::spellbook_data;
use wow_ui_sim::lua_api::{LossOfControlInfo, WowLuaEnv};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("R88m.4 item API environment")
}

fn supported_slot() -> (i32, u32) {
    supported_records()[0]
}

fn supported_records() -> [(i32, u32); 2] {
    let mut records = Vec::new();
    for slot in 1.. {
        let Some((_, entry, line)) = spellbook_data::get_spell_at_slot(slot) else {
            break;
        };
        let supported = line.off_spec_id.is_none()
            && entry.spell_id != 0
            && wow_ui_sim::spells::get_spell(entry.spell_id).is_some();
        if supported && records.iter().all(|(_, spell)| *spell != entry.spell_id) {
            records.push((slot, entry.spell_id));
        }
        if records.len() == 2 {
            break;
        }
    }
    let records: [(i32, u32); 2] = records
        .try_into()
        .expect("two existing supported player-book records with distinct Spell membership");
    assert_ne!(records[0].0, records[1].0);
    assert_ne!(records[0].1, records[1].1);
    records
}

#[test]
fn autocast_missing_item_returns_zero_values() {
    let env = env();
    assert_autocast_supported(&env);
    assert_autocast_missing(&env);
}

fn assert_autocast_missing(env: &WowLuaEnv) {
    // No pet snapshot is supplied. Bank 1 is empty, not bank 0 or action slot 1.
    assert!(env.state().borrow().pet_spells.is_empty());
    for args in ["0, 0", "4294967295, 0", "1, 1", "'0.9', '0.9'"] {
        env.exec(&format!(
            "assert(select('#', C_SpellBook.GetSpellBookItemAutoCast({args})) == 0)"
        ))
        .expect("missing item publishes zero results, not two fabricated booleans");
    }
}

#[test]
fn autocast_supported_record_without_candidate_flags_returns_two_false_booleans() {
    let env = env();
    assert_autocast_missing(&env);
    assert_autocast_supported(&env);
}

fn assert_autocast_supported(env: &WowLuaEnv) {
    for (slot, _) in supported_records() {
        assert_autocast_record(env, slot);
    }
}

fn assert_autocast_record(env: &WowLuaEnv, slot: i32) {
    assert!(
        env.state()
            .borrow()
            .pet_actions
            .iter()
            .all(|a| !a.has_action)
    );
    env.exec(&format!(
        r#"
        assert(select('#', C_SpellBook.GetSpellBookItemAutoCast({slot}, 0)) == 2)
        local allowed, enabled = C_SpellBook.GetSpellBookItemAutoCast({slot}, 0)
        assert(type(allowed) == 'boolean' and allowed == false)
        assert(type(enabled) == 'boolean' and enabled == false)
        "#
    ))
    .expect("supported record with absent candidate state returns two booleans");
}

#[test]
fn loss_of_control_missing_item_and_failed_query_return_zero_values() {
    let env = env();
    assert_loss_of_control_missing(&env);
    assert_loss_of_control_success(&env);
}

fn assert_loss_of_control_missing(env: &WowLuaEnv) {
    let (slot, _) = supported_slot();
    env.state().borrow_mut().spell_loss_of_control.clear();
    for args in [
        "0, 0".to_string(),
        "4294967295, 0".into(),
        "1, 1".into(),
        format!("{slot}, 0"),
    ] {
        env.exec(&format!(
            "assert(select('#', C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo({args})) == 0)"
        ))
        .expect("missing item or failed cooldown query has no public result");
    }
}

#[test]
fn loss_of_control_existing_offspec_rejection_preserves_zero_result() {
    let env = env();
    let slot = (1..)
        .map_while(|slot| spellbook_data::get_spell_at_slot(slot).map(|row| (slot, row)))
        .find_map(|(slot, (_, _, line))| line.off_spec_id.is_some().then_some(slot))
        .expect("existing off-spec row");
    // This adapter rejects off-spec rows already. R88m.4 verifies rejection
    // arity, not the equivalence of off_spec_id with the native rejected flag.
    env.exec(&format!(
        "assert(select('#', C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo({slot}, 0)) == 0)"
    ))
    .expect("inherited rejected-row adapter produces zero values");
}

#[test]
fn loss_of_control_supplied_query_dto_preserves_one_fresh_table() {
    let env = env();
    assert_loss_of_control_success(&env);
    assert_loss_of_control_missing(&env);
}

fn assert_loss_of_control_success(env: &WowLuaEnv) {
    let [first, second] = supported_records();
    let cases = [
        (first, 3.0, 5.0, 1.25, true),
        (second, 11.0, 2.0, 0.75, false),
    ];
    for &((_, spell), start, duration, rate, replaces) in &cases {
        seed_loss_of_control_record(env, spell, start, duration, rate, replaces);
    }
    for ((slot, _), start, duration, rate, replaces) in cases {
        assert_loss_of_control_record(env, slot, start, duration, rate, replaces);
    }
}

fn seed_loss_of_control_record(
    env: &WowLuaEnv,
    spell_id: u32,
    start: f64,
    duration: f64,
    rate: f32,
    replaces: bool,
) {
    env.state().borrow_mut().spell_loss_of_control.insert(
        spell_id,
        LossOfControlInfo {
            start_time: start,
            duration,
            mod_rate: rate,
            is_active: true,
            should_replace_normal_cooldown: replaces,
        },
    );
}

fn assert_loss_of_control_record(
    env: &WowLuaEnv,
    slot: i32,
    start: f64,
    duration: f64,
    rate: f32,
    replaces: bool,
) {
    env.exec(&format!(r#"
        local function query()
            return C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo({slot}, 0)
        end
        assert(select('#', query()) == 1)
        local first, second = query(), query()
        assert(type(first) == 'table' and type(second) == 'table' and first ~= second)
        assert(first.startTime == {start} and first.duration == {duration} and first.modRate == {rate})
        assert(first.isActive == true and first.shouldReplaceNormalCooldown == {replaces})
        local count = 0
        for _ in pairs(first) do count = count + 1 end
        assert(count == 5)
        first.duration = 99
        assert(second.duration == {duration} and query().duration == {duration})
        "#)).expect("successful supplied decoded DTO serializes as one fresh five-field table");
}

#[test]
fn item_queries_report_verified_invalid_numeric_bank_reason() {
    let env = env();
    env.exec(
        r#"
        for _, method in ipairs({
            {'GetSpellBookItemAutoCast', 'Usage: local autoCastAllowed, autoCastEnabled = C_SpellBook.GetSpellBookItemAutoCast(spellBookItem)'},
            {'GetSpellBookItemLossOfControlCooldownInfo', 'Usage: local lossOfControlInfo = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(spellBookItem)'},
        }) do
            for _, bank in ipairs({2, 2.9, '2.9'}) do
                local ok, err = pcall(C_SpellBook[method[1]], 1, bank)
                assert(not ok and type(err) == 'string')
                local reason = 'Invalid spellBank value 2. Must be an Enum.SpellBookSpellBank value. - ' .. method[2]
                assert(string.find(err, reason, 1, true), err)
            end
        end
        "#,
    )
    .expect("numeric bank is truncated then rejected with enum reason and Usage");
}
