//! Integration tests for `src/lua_api/globals/combat_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── CastSpellByID ─────────────────────────────────────────────────────────────

#[test]
fn cast_spell_by_id_populates_unit_casting_info() {
    let env = env();
    let (name, spell_id): (String, f64) = env
        .eval(
            r#"
            CastSpellByID(12345)
            local name, _, _, _, _, _, _, _, spellId = UnitCastingInfo("player")
            return name, spellId
            "#,
        )
        .unwrap();
    assert_eq!(name, "Spell 12345");
    assert_eq!(spell_id as u32, 12345);
}

#[test]
fn cast_spell_by_id_no_arg_is_silent_noop() {
    let env = env();
    let name: Option<String> = env
        .eval("CastSpellByID(); return (UnitCastingInfo('player'))")
        .unwrap();
    assert!(
        name.is_none(),
        "no-arg CastSpellByID should leave casting nil"
    );
}

// ── CastSpellByName ───────────────────────────────────────────────────────────

#[test]
fn cast_spell_by_name_populates_unit_casting_info() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            CastSpellByName("Fireball")
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(name, "Fireball");
}

#[test]
fn cast_spell_by_name_empty_string_is_noop() {
    let env = env();
    let name: Option<String> = env
        .eval(
            r#"
            CastSpellByName("")
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert!(name.is_none(), "empty name must not start a cast");
}

// ── CastSpell (legacy) ────────────────────────────────────────────────────────

#[test]
fn cast_spell_forwards_to_cast_spell_by_id() {
    let env = env();
    let spell_id: f64 = env
        .eval(
            r#"
            CastSpell(77)
            local _, _, _, _, _, _, _, _, spellId = UnitCastingInfo("player")
            return spellId
            "#,
        )
        .unwrap();
    assert_eq!(spell_id as u32, 77);
}

// ── AttackTarget / StopAttack ─────────────────────────────────────────────────

#[test]
fn attack_target_starts_auto_attack_marker() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            AttackTarget()
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(name, "Auto Attack");
}

#[test]
fn stop_attack_clears_auto_attack_only() {
    let env = env();
    let (after_cast, after_stop): (String, String) = env
        .eval(
            r#"
            CastSpellByName("Fireball")
            StopAttack()
            local a = (UnitCastingInfo("player")) or ""
            AttackTarget()
            StopAttack()
            local b = (UnitCastingInfo("player")) or ""
            return a, b
            "#,
        )
        .unwrap();
    assert_eq!(
        after_cast, "Fireball",
        "StopAttack must not clear a non-auto-attack cast"
    );
    assert_eq!(
        after_stop, "",
        "StopAttack must clear the Auto Attack marker"
    );
}

// ── ClickSpecialAbility ───────────────────────────────────────────────────────

#[test]
fn click_special_ability_1_is_auto_attack() {
    let env = env();
    let name: String = env
        .eval("ClickSpecialAbility(1); return (UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(name, "Auto Attack");
}

#[test]
fn click_special_ability_2_is_extra_attack() {
    let env = env();
    let name: String = env
        .eval("ClickSpecialAbility(2); return (UnitCastingInfo('player'))")
        .unwrap();
    assert_eq!(name, "Extra Attack");
}

#[test]
fn click_special_ability_unknown_index_is_noop() {
    let env = env();
    let name: Option<String> = env
        .eval("ClickSpecialAbility(99); return (UnitCastingInfo('player'))")
        .unwrap();
    assert!(name.is_none(), "unknown index must not start a cast");
}

// ── SpellTargetUnit ───────────────────────────────────────────────────────────

#[test]
fn spell_target_unit_is_noop_without_pending_cast() {
    let env = env();
    // Must not error, must not populate casting.
    let name: Option<String> = env
        .eval("SpellTargetUnit('target'); return (UnitCastingInfo('player'))")
        .unwrap();
    assert!(name.is_none());
}

#[test]
fn spell_target_unit_preserves_pending_cast() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            CastSpellByName("Polymorph")
            SpellTargetUnit("target")
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(name, "Polymorph");
}

#[test]
fn spell_is_targeting_defaults_false() {
    let env = env();
    let targeting: bool = env.eval("return SpellIsTargeting()").unwrap();
    assert!(!targeting, "SpellIsTargeting should default false");
}

#[test]
fn spell_item_targeting_helpers_default_false() {
    let env = env();
    let (can_target_item, can_target_item_id): (bool, bool) = env
        .eval("return SpellCanTargetItem(), SpellCanTargetItemID()")
        .unwrap();
    assert!(!can_target_item, "SpellCanTargetItem should default false");
    assert!(
        !can_target_item_id,
        "SpellCanTargetItemID should default false"
    );
}

#[test]
fn spell_stop_casting_clears_active_cast_and_reports_result() {
    let env = env();
    let (stopped, active_cleared, stopped_again): (bool, bool, bool) = env
        .eval(
            r#"
            CastSpellByName("Fireball")
            local stopped = SpellStopCasting()
            local activeAfterStop = UnitCastingInfo("player")
            local stoppedAgain = SpellStopCasting()
            return stopped, activeAfterStop == nil, stoppedAgain
            "#,
        )
        .unwrap();

    assert!(stopped, "active cast should be interrupted");
    assert!(
        active_cleared,
        "SpellStopCasting should clear UnitCastingInfo"
    );
    assert!(
        !stopped_again,
        "SpellStopCasting should report false when no cast is active"
    );
}

// ── Monotonic cast ids ────────────────────────────────────────────────────────

#[test]
fn successive_casts_advance_cast_id() {
    let env = env();
    // Retail's cast GUID moved numeric identity to the castBarID slot.
    let identity_slot = if cfg!(feature = "retail-12-1-0") { 10 } else { 7 };
    let (id1, id2): (f64, f64) = env
        .eval(&format!(
            r#"
            CastSpellByID(1)
            local castId1 = select({identity_slot}, UnitCastingInfo("player"))
            CastSpellByID(2)
            local castId2 = select({identity_slot}, UnitCastingInfo("player"))
            return castId1, castId2
            "#,
        ))
        .unwrap();
    assert!(id2 > id1, "cast id must advance between casts");
}
