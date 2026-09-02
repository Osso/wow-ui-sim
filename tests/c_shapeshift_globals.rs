//! Integration tests for the shapeshift/stance globals registered in
//! `src/lua_api/globals/real/shapeshift.rs`.
//!
//! Verifies `GetShapeshiftFormInfo`, `GetShapeshiftFormCooldown`, and
//! `CastShapeshiftForm` against `state.shapeshift_forms` /
//! `state.shapeshift_cooldowns`. `GetNumShapeshiftForms` is exercised by
//! `tests/social_probes.rs`.

use wow_ui_sim::lua_api::state::{ShapeshiftForm, SpellCooldownState};
use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_druid_forms() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    {
        let mut state = env.state().borrow_mut();
        state.shapeshift_forms = vec![
            ShapeshiftForm {
                name: "Bear Form".to_string(),
                texture: "Interface/Icons/Ability_Racial_BearForm".to_string(),
                spell_id: 5487,
                is_active: false,
                is_castable: true,
            },
            ShapeshiftForm {
                name: "Cat Form".to_string(),
                texture: "Interface/Icons/Ability_Druid_CatForm".to_string(),
                spell_id: 768,
                is_active: false,
                is_castable: true,
            },
        ];
    }
    env
}

#[test]
fn seeded_paladin_exposes_three_aura_forms() {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");

    let (count, first_name, second_name, third_name): (
        i32,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = env
        .eval(
            r#"
            local names = {}
            for i = 1, GetNumShapeshiftForms() do
                local _texture, _active, _castable, spellID = GetShapeshiftFormInfo(i)
                local info = C_Spell.GetSpellInfo(spellID)
                names[i] = info and info.name or nil
            end
            return GetNumShapeshiftForms(), names[1], names[2], names[3]
            "#,
        )
        .expect("seeded Paladin shapeshift aura probe");

    assert_eq!(count, 3);
    assert_eq!(first_name.as_deref(), Some("Devotion Aura"));
    assert_eq!(second_name.as_deref(), Some("Crusader Aura"));
    assert_eq!(third_name.as_deref(), Some("Retribution Aura"));
}

#[test]
fn get_shapeshift_form_cooldown_invalid_index_returns_zero_state() {
    let env = env_with_druid_forms();
    let (start, duration, enable): (f64, f64, f64) = env
        .eval("return GetShapeshiftFormCooldown(0)")
        .expect("invalid shapeshift cooldown index");

    assert_eq!((start, duration, enable), (0.0, 0.0, 1.0));
}

#[test]
fn get_shapeshift_form_info_reports_all_four_fields() {
    let env = env_with_druid_forms();
    let (texture, is_active, is_castable, spell_id): (String, bool, bool, i32) = env
        .eval("return GetShapeshiftFormInfo(1)")
        .expect("eval GetShapeshiftFormInfo");
    assert_eq!(texture, "Interface/Icons/Ability_Racial_BearForm");
    assert!(!is_active);
    assert!(is_castable);
    assert_eq!(spell_id, 5487);
}

#[test]
fn get_shapeshift_form_info_out_of_range_returns_nil() {
    let env = env_with_druid_forms();
    let is_nil: bool = env.eval("return GetShapeshiftFormInfo(99) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn get_shapeshift_form_info_zero_index_returns_nil() {
    let env = env_with_druid_forms();
    let is_nil: bool = env.eval("return GetShapeshiftFormInfo(0) == nil").unwrap();
    assert!(is_nil, "1-based — index 0 is invalid");
}

#[test]
fn get_shapeshift_form_cooldown_defaults_to_zero_with_enable_one() {
    let env = env_with_druid_forms();
    let (start, duration, enable): (f64, f64, f64) = env
        .eval("return GetShapeshiftFormCooldown(1)")
        .expect("eval GetShapeshiftFormCooldown");
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1.0, "enable is always 1 when no cooldown");
}

#[test]
fn get_shapeshift_form_cooldown_reads_state_entry() {
    let env = env_with_druid_forms();
    {
        let mut state = env.state().borrow_mut();
        state.shapeshift_cooldowns.insert(
            2,
            SpellCooldownState {
                start: 100.0,
                duration: 30.0,
            },
        );
    }
    let (start, duration, enable): (f64, f64, f64) =
        env.eval("return GetShapeshiftFormCooldown(2)").unwrap();
    assert_eq!(start, 100.0);
    assert_eq!(duration, 30.0);
    assert_eq!(enable, 1.0);
}

#[test]
fn cast_shapeshift_form_activates_target_form() {
    let env = env_with_druid_forms();
    env.exec("CastShapeshiftForm(1)").unwrap();
    let state = env.state().borrow();
    assert!(state.shapeshift_forms[0].is_active);
    assert!(!state.shapeshift_forms[1].is_active);
}

#[test]
fn get_shapeshift_form_reports_active_form_index() {
    let env = env_with_druid_forms();
    {
        let mut state = env.state().borrow_mut();
        state.shapeshift_forms[1].is_active = true;
    }

    let active_index: i32 = env.eval("return GetShapeshiftForm()").unwrap();

    assert_eq!(active_index, 2);
}

#[test]
fn cast_shapeshift_form_clears_previously_active_form() {
    let env = env_with_druid_forms();
    {
        let mut state = env.state().borrow_mut();
        state.shapeshift_forms[1].is_active = true;
    }
    env.exec("CastShapeshiftForm(1)").unwrap();
    let state = env.state().borrow();
    assert!(state.shapeshift_forms[0].is_active);
    assert!(
        !state.shapeshift_forms[1].is_active,
        "previously active form should be cleared"
    );
}

#[test]
fn cast_shapeshift_form_toggles_off_when_already_active() {
    let env = env_with_druid_forms();
    {
        let mut state = env.state().borrow_mut();
        state.shapeshift_forms[0].is_active = true;
    }
    env.exec("CastShapeshiftForm(1)").unwrap();
    let state = env.state().borrow();
    assert!(
        !state.shapeshift_forms[0].is_active,
        "casting current form drops back to humanoid"
    );
}

#[test]
fn cast_shapeshift_form_fires_update_event() {
    let env = env_with_druid_forms();
    let fired: bool = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("UPDATE_SHAPESHIFT_FORM")
            f:SetScript("OnEvent", function() fired = true end)
            CastShapeshiftForm(1)
            return fired
            "#,
        )
        .unwrap();
    assert!(fired, "CastShapeshiftForm must fire UPDATE_SHAPESHIFT_FORM");
}

#[test]
fn cast_shapeshift_form_out_of_range_is_silent_noop() {
    let env = env_with_druid_forms();
    env.exec("CastShapeshiftForm(99)").unwrap();
    let state = env.state().borrow();
    assert!(state.shapeshift_forms.iter().all(|f| !f.is_active));
}
