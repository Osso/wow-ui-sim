//! Tests for the `C_AzeriteEssence` SimState-backed surface.
//! Covers milestone/essence reads, pending-activation lifecycle,
//! activate/unlock event dispatch, the forge state, hyperlink format,
//! and combat gating.

use std::collections::HashMap;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{
    AzeriteEssenceInfo, AzeriteEssenceMilestoneInfo, AzeriteEssenceState,
};

const MAIN_SLOT: i32 = 0;
const PASSIVE_ONE_SLOT: i32 = 1;

fn major_milestone(id: i32, unlocked: bool) -> AzeriteEssenceMilestoneInfo {
    AzeriteEssenceMilestoneInfo {
        id,
        required_level: 50,
        slot: Some(MAIN_SLOT),
        unlocked,
        can_unlock: !unlocked,
        is_major_slot: true,
        swirl_scale: 1.0,
        requires_only_aura: false,
        spell_id: 100_000 + id,
        rank: None,
        active_essence_id: None,
    }
}

fn minor_milestone(id: i32, unlocked: bool) -> AzeriteEssenceMilestoneInfo {
    AzeriteEssenceMilestoneInfo {
        id,
        required_level: 55,
        slot: Some(PASSIVE_ONE_SLOT),
        unlocked,
        can_unlock: !unlocked,
        is_major_slot: false,
        swirl_scale: 1.0,
        requires_only_aura: false,
        spell_id: 200_000 + id,
        rank: None,
        active_essence_id: None,
    }
}

fn stamina_milestone(id: i32) -> AzeriteEssenceMilestoneInfo {
    AzeriteEssenceMilestoneInfo {
        id,
        required_level: 60,
        slot: None,
        unlocked: true,
        can_unlock: false,
        is_major_slot: false,
        swirl_scale: 0.5,
        requires_only_aura: true,
        spell_id: 300_000 + id,
        rank: Some(1),
        active_essence_id: None,
    }
}

fn essence(id: i32, name: &str, rank: i32, unlocked: bool, valid: bool) -> AzeriteEssenceInfo {
    AzeriteEssenceInfo {
        id,
        name: name.to_string(),
        rank,
        icon: 3_000_000 + id,
        unlocked,
        valid,
        access_rank: rank,
        has_never_activated: false,
    }
}

fn seeded_state() -> AzeriteEssenceState {
    let mut essences = HashMap::new();
    essences.insert(200, essence(200, "Anima of Life and Death", 3, true, true));
    essences.insert(201, essence(201, "Memory of Lucid Dreams", 2, true, true));
    essences.insert(202, essence(202, "Vitality Conduit", 1, false, true));
    AzeriteEssenceState {
        milestones: vec![
            major_milestone(100, true),
            minor_milestone(101, true),
            minor_milestone(102, false),
            stamina_milestone(103),
        ],
        essences,
        essence_order: vec![200, 201, 202],
        pending_activation_essence: None,
        num_unlocked: 2,
        is_at_forge: false,
        has_never_activated: false,
        has_neck_equipped: true,
        neck_power_level: 75,
        ..AzeriteEssenceState::default()
    }
}

const PENDING_LISTENER: &str = r#"
    pending_log = {}
    local f = CreateFrame("Frame")
    f:RegisterEvent("PENDING_AZERITE_ESSENCE_CHANGED")
    f:SetScript("OnEvent", function(_, _, prev, new)
        table.insert(pending_log, { prev = prev, new = new })
    end)
"#;

const ACTIVATED_LISTENER: &str = r#"
    activated_log = {}
    changed_log = {}
    local f = CreateFrame("Frame")
    f:RegisterEvent("AZERITE_ESSENCE_ACTIVATED")
    f:RegisterEvent("AZERITE_ESSENCE_CHANGED")
    f:SetScript("OnEvent", function(_, event, a, b)
        if event == "AZERITE_ESSENCE_ACTIVATED" then
            table.insert(activated_log, { essence = a, milestone = b })
        else
            table.insert(changed_log, { essence = a, rank = b })
        end
    end)
"#;

const FORGE_CLOSE_LISTENER: &str = r#"
    forge_close_count = 0
    local f = CreateFrame("Frame")
    f:RegisterEvent("AZERITE_ESSENCE_FORGE_CLOSE")
    f:SetScript("OnEvent", function(_, _) forge_close_count = forge_close_count + 1 end)
"#;

const UNLOCK_LISTENER: &str = r#"
    unlock_log = {}
    local f = CreateFrame("Frame")
    f:RegisterEvent("AZERITE_ESSENCE_MILESTONE_UNLOCKED")
    f:SetScript("OnEvent", function(_, _, id) table.insert(unlock_log, id) end)
"#;

#[test]
fn get_milestones_returns_canonical_table_shape() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let (count, first_id, first_unlocked, first_is_major, first_slot, stamina_slot_nil): (
        i32,
        i32,
        bool,
        bool,
        i32,
        bool,
    ) = env
        .eval(
            r#"
            local m = C_AzeriteEssence.GetMilestones()
            return #m,
                m[1].ID,
                m[1].unlocked,
                m[1].isMajorSlot,
                m[1].slot,
                m[4].slot == nil
            "#,
        )
        .unwrap();
    assert_eq!(count, 4);
    assert_eq!(first_id, 100);
    assert!(first_unlocked);
    assert!(first_is_major);
    assert_eq!(first_slot, MAIN_SLOT);
    assert!(stamina_slot_nil);
}

#[test]
fn get_milestones_default_state_is_empty() {
    let env = WowLuaEnv::new().unwrap();
    let count: i32 = env
        .eval("return #C_AzeriteEssence.GetMilestones()")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_essences_returns_in_essence_order() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let (count, first_id, first_name, first_unlocked, third_valid): (i32, i32, String, bool, bool) =
        env.eval(
            r#"
            local e = C_AzeriteEssence.GetEssences()
            return #e, e[1].ID, e[1].name, e[1].unlocked, e[3].valid
            "#,
        )
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(first_id, 200);
    assert_eq!(first_name, "Anima of Life and Death");
    assert!(first_unlocked);
    assert!(third_valid);
}

#[test]
fn get_essence_info_returns_nil_for_unknown_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let nil: bool = env
        .eval("return C_AzeriteEssence.GetEssenceInfo(9999) == nil")
        .unwrap();
    assert!(nil);
}

#[test]
fn get_essence_info_returns_table_for_known_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let (id, name, rank): (i32, String, i32) = env
        .eval(
            r#"
            local info = C_AzeriteEssence.GetEssenceInfo(201)
            return info.ID, info.name, info.rank
            "#,
        )
        .unwrap();
    assert_eq!(id, 201);
    assert_eq!(name, "Memory of Lucid Dreams");
    assert_eq!(rank, 2);
}

#[test]
fn get_milestone_info_returns_nil_for_unknown_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let nil: bool = env
        .eval("return C_AzeriteEssence.GetMilestoneInfo(9999) == nil")
        .unwrap();
    assert!(nil);
}

#[test]
fn get_milestone_spell_returns_spell_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let spell_id: i32 = env
        .eval("return C_AzeriteEssence.GetMilestoneSpell(100)")
        .unwrap();
    assert_eq!(spell_id, 100_100);
}

#[test]
fn get_milestone_spell_returns_nil_for_unknown_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let nil: bool = env
        .eval("return C_AzeriteEssence.GetMilestoneSpell(9999) == nil")
        .unwrap();
    assert!(nil);
}

#[test]
fn get_milestone_essence_reflects_active_slot() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.azerite_essence = seeded_state();
        sim.azerite_essence.milestones[0].active_essence_id = Some(201);
    }
    let active: i32 = env
        .eval("return C_AzeriteEssence.GetMilestoneEssence(100)")
        .unwrap();
    assert_eq!(active, 201);
    let empty_nil: bool = env
        .eval("return C_AzeriteEssence.GetMilestoneEssence(101) == nil")
        .unwrap();
    assert!(empty_nil);
}

#[test]
fn get_num_unlocked_essences_returns_state_value() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let count: i32 = env
        .eval("return C_AzeriteEssence.GetNumUnlockedEssences()")
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn pending_activation_lifecycle_fires_event_with_prev_and_new() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let script = format!(
        r#"
        {listener}
        C_AzeriteEssence.SetPendingActivationEssence(200)
        C_AzeriteEssence.ClearPendingActivationEssence()
        return #pending_log,
            pending_log[1].prev == nil,
            pending_log[1].new,
            pending_log[2].prev,
            pending_log[2].new == nil
        "#,
        listener = PENDING_LISTENER,
    );
    let (count, first_prev_nil, first_new, second_prev, second_new_nil): (
        i32,
        bool,
        i32,
        i32,
        bool,
    ) = env.eval(&script).unwrap();
    assert_eq!(count, 2);
    assert!(first_prev_nil);
    assert_eq!(first_new, 200);
    assert_eq!(second_prev, 200);
    assert!(second_new_nil);
}

#[test]
fn pending_activation_lifecycle_clears_pending_state() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    env.exec("C_AzeriteEssence.SetPendingActivationEssence(200)")
        .unwrap();
    env.exec("C_AzeriteEssence.ClearPendingActivationEssence()")
        .unwrap();
    let pending: bool = env
        .eval("return C_AzeriteEssence.HasPendingActivationEssence()")
        .unwrap();
    assert!(!pending);
}

#[test]
fn clear_pending_activation_is_silent_when_already_clear() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let script = format!(
        r#"
        {listener}
        C_AzeriteEssence.ClearPendingActivationEssence()
        return #pending_log
        "#,
        listener = PENDING_LISTENER,
    );
    let count: i32 = env.eval(&script).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn activate_essence_fires_activated_then_changed() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    env.exec("C_AzeriteEssence.SetPendingActivationEssence(200)")
        .unwrap();
    let script = format!(
        r#"
        {listener}
        local ok = C_AzeriteEssence.ActivateEssence(200, 100)
        return ok,
            #activated_log, activated_log[1].essence, activated_log[1].milestone,
            #changed_log, changed_log[1].essence, changed_log[1].rank
        "#,
        listener = ACTIVATED_LISTENER,
    );
    let event_results: (bool, i32, i32, i32, i32, i32, i32) = env.eval(&script).unwrap();
    assert_eq!(event_results, (true, 1, 200, 100, 1, 200, 3));
}

#[test]
fn activate_essence_updates_state_and_clears_pending() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    env.exec("C_AzeriteEssence.SetPendingActivationEssence(200)")
        .unwrap();
    let activated: bool = env
        .eval("return C_AzeriteEssence.ActivateEssence(200, 100)")
        .unwrap();
    assert!(activated);
    let sim = env.state().borrow();
    assert_eq!(
        sim.azerite_essence.milestones[0].active_essence_id,
        Some(200)
    );
    assert!(sim.azerite_essence.pending_activation_essence.is_none());
}

#[test]
fn activate_essence_returns_false_when_milestone_locked() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let ok: bool = env
        .eval("return C_AzeriteEssence.ActivateEssence(200, 102)")
        .unwrap();
    assert!(!ok);
}

#[test]
fn activate_essence_returns_false_for_unknown_essence() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let ok: bool = env
        .eval("return C_AzeriteEssence.ActivateEssence(9999, 100)")
        .unwrap();
    assert!(!ok);
}

#[test]
fn can_activate_essence_requires_unlocked_essence_and_milestone() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let (yes_unlocked, no_locked_essence, no_locked_milestone): (bool, bool, bool) = env
        .eval(
            r#"
            return C_AzeriteEssence.CanActivateEssence(200, 100),
                C_AzeriteEssence.CanActivateEssence(202, 100),
                C_AzeriteEssence.CanActivateEssence(200, 102)
            "#,
        )
        .unwrap();
    assert!(yes_unlocked);
    assert!(!no_locked_essence);
    assert!(!no_locked_milestone);
}

#[test]
fn can_activate_essence_blocks_in_combat() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.azerite_essence = seeded_state();
        sim.player.in_combat = true;
    }
    let allowed: bool = env
        .eval("return C_AzeriteEssence.CanActivateEssence(200, 100)")
        .unwrap();
    assert!(!allowed);
}

#[test]
fn can_open_ui_reflects_neck_equipped_flag() {
    let env = WowLuaEnv::new().unwrap();
    let default_no: bool = env.eval("return C_AzeriteEssence.CanOpenUI()").unwrap();
    assert!(!default_no);
    env.state().borrow_mut().azerite_essence.has_neck_equipped = true;
    let after_yes: bool = env.eval("return C_AzeriteEssence.CanOpenUI()").unwrap();
    assert!(after_yes);
}

#[test]
fn close_forge_clears_state_and_fires_event() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence.is_at_forge = true;
    let script = format!(
        r#"
        {listener}
        C_AzeriteEssence.CloseForge()
        return forge_close_count, C_AzeriteEssence.IsAtForge()
        "#,
        listener = FORGE_CLOSE_LISTENER,
    );
    let (count, at_forge): (i32, bool) = env.eval(&script).unwrap();
    assert_eq!(count, 1);
    assert!(!at_forge);
}

#[test]
fn unlock_milestone_fires_event_and_marks_unlocked() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let script = format!(
        r#"
        {listener}
        local ok = C_AzeriteEssence.UnlockMilestone(102)
        return ok, #unlock_log, unlock_log[1]
        "#,
        listener = UNLOCK_LISTENER,
    );
    let (ok, count, first): (bool, i32, i32) = env.eval(&script).unwrap();
    assert!(ok);
    assert_eq!(count, 1);
    assert_eq!(first, 102);
    let sim = env.state().borrow();
    let m = sim
        .azerite_essence
        .milestones
        .iter()
        .find(|m| m.id == 102)
        .unwrap();
    assert!(m.unlocked);
    assert!(!m.can_unlock);
}

#[test]
fn unlock_milestone_returns_false_for_unknown_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let script = format!(
        r#"
        {listener}
        local ok = C_AzeriteEssence.UnlockMilestone(9999)
        return ok, #unlock_log
        "#,
        listener = UNLOCK_LISTENER,
    );
    let (ok, count): (bool, i32) = env.eval(&script).unwrap();
    assert!(!ok);
    assert_eq!(count, 0);
}

#[test]
fn has_never_activated_any_essences_returns_state_flag() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence.has_never_activated = true;
    let yes: bool = env
        .eval("return C_AzeriteEssence.HasNeverActivatedAnyEssences()")
        .unwrap();
    assert!(yes);
    env.state().borrow_mut().azerite_essence.has_never_activated = false;
    let no: bool = env
        .eval("return C_AzeriteEssence.HasNeverActivatedAnyEssences()")
        .unwrap();
    assert!(!no);
}

#[test]
fn get_essence_hyperlink_returns_canonical_format() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().azerite_essence = seeded_state();
    let link: String = env
        .eval("return C_AzeriteEssence.GetEssenceHyperlink(200, 3)")
        .unwrap();
    assert_eq!(
        link,
        "|cffa335ee|Hazessence:200:3|h[Anima of Life and Death]|h|r"
    );
}

#[test]
fn get_essence_hyperlink_returns_nil_for_unknown_id() {
    let env = WowLuaEnv::new().unwrap();
    let nil: bool = env
        .eval("return C_AzeriteEssence.GetEssenceHyperlink(9999, 1) == nil")
        .unwrap();
    assert!(nil);
}

#[test]
fn activate_essence_clears_has_never_activated_flag() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.azerite_essence = seeded_state();
        sim.azerite_essence.has_never_activated = true;
        if let Some(e) = sim.azerite_essence.essences.get_mut(&200) {
            e.has_never_activated = true;
        }
    }
    env.exec("C_AzeriteEssence.ActivateEssence(200, 100)")
        .unwrap();
    let sim = env.state().borrow();
    assert!(!sim.azerite_essence.has_never_activated);
    let entry = sim.azerite_essence.essences.get(&200).unwrap();
    assert!(!entry.has_never_activated);
}
