//! `C_Spell` flyout/spell-book probes consumed by Blizzard_ActionBar.
//!
//! - `GetSpellTradeSkillLink` reads `state.spell_trade_skill_links`.
//! - `GetSpellIDForSpellIdentifier` resolves through `state.spell_id_aliases`.
//! - `IsCurrentSpell` reads `state.casting.spell_id`.
//! - `GetSpellLossOfControlCooldownInfo` reads `state.spell_loss_of_control`.

use wow_ui_sim::lua_api::{LossOfControlInfo, WowLuaEnv, state::CastingState};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("env")
}

#[test]
fn trade_skill_link_defaults_nil() {
    let env = env();
    let result: Option<String> = env
        .eval("return C_Spell.GetSpellTradeSkillLink(2259)")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn trade_skill_link_returns_state_entry() {
    let env = env();
    env.state()
        .borrow_mut()
        .spell_trade_skill_links
        .insert(2259, "|cffffd000|Henchant:2259|h[Alchemy]|h|r".into());
    let result: String = env
        .eval("return C_Spell.GetSpellTradeSkillLink(2259)")
        .unwrap();
    assert_eq!(result, "|cffffd000|Henchant:2259|h[Alchemy]|h|r");
}

#[test]
fn trade_skill_link_invalid_input_returns_nil() {
    let env = env();
    let result: Option<String> = env
        .eval("return C_Spell.GetSpellTradeSkillLink({})")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn id_for_identifier_passes_numeric_through_when_no_alias() {
    let env = env();
    let result: i64 = env
        .eval("return C_Spell.GetSpellIDForSpellIdentifier(133)")
        .unwrap();
    assert_eq!(result, 133, "numeric input should pass through unchanged");
}

#[test]
fn id_for_identifier_resolves_numeric_alias() {
    let env = env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("100".into(), 200);
    let result: i64 = env
        .eval("return C_Spell.GetSpellIDForSpellIdentifier(100)")
        .unwrap();
    assert_eq!(result, 200, "registered alias should resolve");
}

#[test]
fn id_for_identifier_resolves_name_alias_case_insensitive() {
    let env = env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fireball".into(), 133);
    let from_lower: i64 = env
        .eval(r#"return C_Spell.GetSpellIDForSpellIdentifier("fireball")"#)
        .unwrap();
    let from_mixed: i64 = env
        .eval(r#"return C_Spell.GetSpellIDForSpellIdentifier("FireBall")"#)
        .unwrap();
    assert_eq!(from_lower, 133);
    assert_eq!(from_mixed, 133, "name lookup should be case-insensitive");
}

#[test]
fn id_for_identifier_unknown_name_is_nil() {
    let env = env();
    let result: Option<i64> = env
        .eval(r#"return C_Spell.GetSpellIDForSpellIdentifier("nonesuch")"#)
        .unwrap();
    assert!(
        result.is_none(),
        "string input without alias should return nil"
    );
}

#[test]
fn id_for_identifier_invalid_input_is_nil() {
    let env = env();
    let result: Option<i64> = env
        .eval("return C_Spell.GetSpellIDForSpellIdentifier({})")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn is_current_spell_defaults_false() {
    let env = env();
    let result: bool = env.eval("return C_Spell.IsCurrentSpell(133)").unwrap();
    assert!(!result);
}

#[test]
fn is_current_spell_matches_casting_state() {
    let env = env();
    env.state().borrow_mut().casting = Some(CastingState {
        spell_id: 133,
        spell_name: "Fireball".into(),
        icon_path: String::new(),
        start_time: 0.0,
        end_time: 1.0,
        cast_id: 1,
        empower: None,
        delay_time: 0.0,
    });
    let matches: bool = env.eval("return C_Spell.IsCurrentSpell(133)").unwrap();
    let other: bool = env.eval("return C_Spell.IsCurrentSpell(999)").unwrap();
    assert!(matches, "matching spell id should report true");
    assert!(!other, "non-matching spell id should report false");
}

#[test]
fn flyout_update_state_branches_on_current_spell() {
    // Mirrors SpellFlyoutPopupButtonMixin:UpdateState — checked when the
    // active cast matches the flyout entry, unchecked otherwise.
    let env = env();
    env.state().borrow_mut().casting = Some(CastingState {
        spell_id: 42,
        spell_name: "Fire Blast".into(),
        icon_path: String::new(),
        start_time: 0.0,
        end_time: 1.0,
        cast_id: 1,
        empower: None,
        delay_time: 0.0,
    });
    let (checked_match, checked_other): (bool, bool) = env
        .eval(
            r#"
            local function updateChecked(spellID)
                if C_Spell.IsCurrentSpell(spellID) then
                    return true
                else
                    return false
                end
            end
            return updateChecked(42), updateChecked(99)
            "#,
        )
        .unwrap();
    assert!(checked_match);
    assert!(!checked_other);
}

#[test]
fn loss_of_control_defaults_nil() {
    let env = env();
    let result: Option<bool> = env
        .eval(
            r#"
            local info = C_Spell.GetSpellLossOfControlCooldownInfo(133)
            if info == nil then return nil end
            return info.isActive
            "#,
        )
        .unwrap();
    assert!(
        result.is_none(),
        "missing entry should return nil so callers fall back to defaultLossOfControlInfo"
    );
}

#[test]
fn loss_of_control_returns_state_table() {
    let env = env();
    env.state().borrow_mut().spell_loss_of_control.insert(
        500,
        LossOfControlInfo {
            start_time: 12.5,
            duration: 4.0,
            mod_rate: 0.75,
            is_active: true,
            should_replace_normal_cooldown: true,
        },
    );
    let (start, duration, mod_rate, is_active, replace): (f64, f64, f64, bool, bool) = env
        .eval(
            r#"
            local info = C_Spell.GetSpellLossOfControlCooldownInfo(500)
            return info.startTime, info.duration, info.modRate,
                   info.isActive, info.shouldReplaceNormalCooldown
            "#,
        )
        .unwrap();
    assert_eq!(start, 12.5);
    assert_eq!(duration, 4.0);
    assert!((mod_rate - 0.75).abs() < 1e-6);
    assert!(is_active);
    assert!(replace);
}

#[test]
fn loss_of_control_invalid_input_returns_nil() {
    let env = env();
    let result: Option<bool> = env
        .eval(
            r#"
            local info = C_Spell.GetSpellLossOfControlCooldownInfo({})
            if info == nil then return nil end
            return true
            "#,
        )
        .unwrap();
    assert!(result.is_none());
}
