//! Integration tests for the pet action-bar globals registered in
//! `src/lua_api/globals/real/pet_bar.rs`.
//!
//! Verifies `GetNumPetActions`, `GetPetActionInfo`, `GetPetActionCooldown`,
//! `CastPetAction`, `TogglePetAutocast`, `CancelPetPossess`, and
//! `PetHasActionBar`/`HasPetUI` against `state.pet_actions`.

use wow_ui_sim::lua_api::state::{PetActionSlot, SpellCooldownState};
use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_pet_slots() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    {
        let mut state = env.state().borrow_mut();
        state.pet_actions[0] = PetActionSlot {
            has_action: true,
            name: Some("Claw".to_string()),
            texture: Some("Interface/Icons/Ability_Druid_Rake".to_string()),
            is_token: false,
            is_active: false,
            auto_cast_allowed: true,
            auto_cast_enabled: false,
            spell_id: Some(16827),
            passive: false,
            cooldown: None,
        };
        state.pet_actions[1] = PetActionSlot {
            has_action: true,
            name: Some("PET_MODE_ASSIST".to_string()),
            texture: Some("PET_ASSIST_TEXTURE".to_string()),
            is_token: true,
            is_active: true,
            auto_cast_allowed: false,
            auto_cast_enabled: false,
            spell_id: None,
            passive: true,
            cooldown: None,
        };
    }
    env
}

#[test]
fn cancel_pet_possess_fires_pet_bar_update_even_when_clearing_state() {
    let env = env_with_pet_slots();
    env.state().borrow_mut().pet_actions[0].is_active = true;

    let fired: bool = env
        .eval(
            r#"
            local fired = false
            local frame = CreateFrame("Frame")
            frame:RegisterEvent("PET_BAR_UPDATE")
            frame:SetScript("OnEvent", function() fired = true end)
            CancelPetPossess()
            return fired
            "#,
        )
        .expect("CancelPetPossess fires PET_BAR_UPDATE");

    assert!(fired, "CancelPetPossess must fire PET_BAR_UPDATE");
    assert!(
        env.state()
            .borrow()
            .pet_actions
            .iter()
            .all(|slot| !slot.is_active),
        "CancelPetPossess must clear active pet slots"
    );
}

#[test]
fn get_num_pet_actions_reports_ten_slots_by_default() {
    let env = WowLuaEnv::new().unwrap();
    let count: i32 = env.eval("return GetNumPetActions()").unwrap();
    assert_eq!(count, 10);
}

#[test]
fn get_pet_action_info_returns_full_nine_tuple_for_active_slot() {
    let env = env_with_pet_slots();
    let (
        name,
        texture,
        is_token,
        is_active,
        auto_cast_allowed,
        auto_cast_enabled,
        spell_id,
        unused,
        passive,
    ): (String, String, bool, bool, bool, bool, i32, bool, bool) = env
        .eval("return GetPetActionInfo(1)")
        .expect("eval GetPetActionInfo");
    assert_eq!(name, "Claw");
    assert_eq!(texture, "Interface/Icons/Ability_Druid_Rake");
    assert!(!is_token);
    assert!(!is_active);
    assert!(auto_cast_allowed);
    assert!(!auto_cast_enabled);
    assert_eq!(spell_id, 16827);
    assert!(!unused, "8th return is reserved/unused");
    assert!(!passive);
}

#[test]
fn get_pet_action_info_empty_slot_returns_default_nine_tuple() {
    let env = env_with_pet_slots();
    let (name, texture, is_token, is_active, _aca, _ace, spell_id, _u, passive): (
        Option<String>,
        Option<String>,
        bool,
        bool,
        bool,
        bool,
        Option<i32>,
        bool,
        bool,
    ) = env.eval("return GetPetActionInfo(5)").unwrap();
    assert!(name.is_none());
    assert!(texture.is_none());
    assert!(!is_token);
    assert!(!is_active);
    assert!(spell_id.is_none());
    assert!(!passive);
}

#[test]
fn get_pet_action_info_out_of_range_returns_empty_tuple() {
    let env = env_with_pet_slots();
    let (name, texture): (Option<String>, Option<String>) =
        env.eval("return GetPetActionInfo(99)").unwrap();
    assert!(name.is_none());
    assert!(texture.is_none());
}

#[test]
fn get_pet_action_cooldown_defaults_to_zero_with_enable_one() {
    let env = env_with_pet_slots();
    let (start, duration, enable): (f64, f64, f64) =
        env.eval("return GetPetActionCooldown(1)").unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1.0);
}

#[test]
fn get_pet_action_cooldown_reads_slot_cooldown() {
    let env = env_with_pet_slots();
    {
        let mut state = env.state().borrow_mut();
        state.pet_actions[0].cooldown = Some(SpellCooldownState {
            start: 50.0,
            duration: 10.0,
        });
    }
    let (start, duration, enable): (f64, f64, f64) =
        env.eval("return GetPetActionCooldown(1)").unwrap();
    assert_eq!(start, 50.0);
    assert_eq!(duration, 10.0);
    assert_eq!(enable, 1.0);
}

#[test]
fn get_pet_action_slot_usable_reports_bound_slots_only() {
    let env = env_with_pet_slots();
    let (active_slot, passive_slot, empty_slot, out_of_range): (bool, bool, bool, bool) = env
        .eval(
            r#"
            return GetPetActionSlotUsable(1),
                   GetPetActionSlotUsable(2),
                   GetPetActionSlotUsable(7),
                   GetPetActionSlotUsable(99)
            "#,
        )
        .unwrap();

    assert!(active_slot);
    assert!(passive_slot);
    assert!(!empty_slot);
    assert!(!out_of_range);
}

#[test]
fn cast_pet_action_toggles_active_and_fires_event() {
    let env = env_with_pet_slots();
    let fired: bool = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("PET_BAR_UPDATE")
            f:SetScript("OnEvent", function() fired = true end)
            CastPetAction(1)
            return fired
            "#,
        )
        .unwrap();
    assert!(fired, "CastPetAction must fire PET_BAR_UPDATE");
    let state = env.state().borrow();
    assert!(state.pet_actions[0].is_active);
}

#[test]
fn cast_pet_action_toggles_off_when_already_active() {
    let env = env_with_pet_slots();
    {
        let mut state = env.state().borrow_mut();
        state.pet_actions[0].is_active = true;
    }
    env.exec("CastPetAction(1)").unwrap();
    let state = env.state().borrow();
    assert!(
        !state.pet_actions[0].is_active,
        "second cast should toggle the slot off"
    );
}

#[test]
fn cast_pet_action_passive_slot_is_noop() {
    let env = env_with_pet_slots();
    env.exec("CastPetAction(2)").unwrap();
    let state = env.state().borrow();
    assert!(
        state.pet_actions[1].is_active,
        "passive slot's is_active should not flip from cast"
    );
}

#[test]
fn cast_pet_action_empty_slot_is_silent_noop() {
    let env = env_with_pet_slots();
    env.exec("CastPetAction(7)").unwrap();
    let state = env.state().borrow();
    assert!(!state.pet_actions[6].is_active);
}

#[test]
fn toggle_pet_autocast_flips_enabled_when_allowed() {
    let env = env_with_pet_slots();
    env.exec("TogglePetAutocast(1)").unwrap();
    {
        let state = env.state().borrow();
        assert!(state.pet_actions[0].auto_cast_enabled);
    }
    env.exec("TogglePetAutocast(1)").unwrap();
    let state = env.state().borrow();
    assert!(!state.pet_actions[0].auto_cast_enabled);
}

#[test]
fn toggle_pet_autocast_noop_when_not_allowed() {
    let env = env_with_pet_slots();
    env.exec("TogglePetAutocast(2)").unwrap();
    let state = env.state().borrow();
    assert!(
        !state.pet_actions[1].auto_cast_enabled,
        "slot without auto_cast_allowed should not flip"
    );
}

#[test]
fn cancel_pet_possess_clears_all_active_flags() {
    let env = env_with_pet_slots();
    {
        let mut state = env.state().borrow_mut();
        state.pet_actions[0].is_active = true;
        state.pet_actions[1].is_active = true;
    }
    env.exec("CancelPetPossess()").unwrap();
    let state = env.state().borrow();
    assert!(state.pet_actions.iter().all(|s| !s.is_active));
}

#[test]
fn pet_has_action_bar_reports_true_when_any_slot_bound() {
    let env = env_with_pet_slots();
    let has: bool = env.eval("return PetHasActionBar()").unwrap();
    assert!(has);
}

#[test]
fn pet_has_action_bar_reports_false_for_default_state() {
    let env = WowLuaEnv::new().unwrap();
    let has: bool = env.eval("return PetHasActionBar()").unwrap();
    assert!(!has, "default 10 empty slots should report no pet bar");
}

#[test]
fn has_pet_ui_reports_true_when_any_slot_bound() {
    let env = env_with_pet_slots();
    let (has_pet_ui, can_gain_xp): (bool, bool) = env.eval("return HasPetUI()").unwrap();
    assert!(has_pet_ui);
    assert!(!can_gain_xp);
}

#[test]
fn has_pet_ui_reports_xp_capable_when_pet_xp_is_seeded() {
    let env = env_with_pet_slots();
    env.state().borrow_mut().pet.xp_max = 10_000;
    let (has_pet_ui, can_gain_xp): (bool, bool) = env.eval("return HasPetUI()").unwrap();
    assert!(has_pet_ui);
    assert!(can_gain_xp);
}

#[test]
fn has_pet_ui_reports_false_for_default_state() {
    let env = WowLuaEnv::new().unwrap();
    let (has_pet_ui, can_gain_xp): (bool, bool) = env.eval("return HasPetUI()").unwrap();
    assert!(!has_pet_ui);
    assert!(!can_gain_xp);
}

#[test]
fn admin_can_seed_pet_action_slot_for_panel_tests() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetPetActionSlot(
            1,
            "Claw",
            "Interface/Icons/Ability_Druid_Rake",
            16827
        )
        "#,
    )
    .unwrap();

    let (has_pet_ui, name, spell_id): (bool, String, i32) = env
        .eval(
            r#"
            local actionName, _, _, _, _, _, actionSpellID = GetPetActionInfo(1)
            return HasPetUI(), actionName, actionSpellID
            "#,
        )
        .unwrap();

    assert!(has_pet_ui);
    assert_eq!(name, "Claw");
    assert_eq!(spell_id, 16827);
}
