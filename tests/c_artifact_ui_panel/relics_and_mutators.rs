use super::{sample_artifact, seed_artifact};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin, RelicSlotInfo};

#[test]
fn relic_slot_count_and_info_read_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.relic_slots.push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: None,
            name: "Iron Relic".to_string(),
            icon: "Interface/Icons/iron".to_string(),
            link: "item:111".to_string(),
        });
        st.viewed_artifact.relic_slots.push(RelicSlotInfo {
            slot_type: "Blood".to_string(),
            locked_reason: Some("Locked".to_string()),
            name: String::new(),
            icon: String::new(),
            link: String::new(),
        });
    }
    let total: f64 = env.eval("return C_ArtifactUI.GetNumRelicSlots()").unwrap();
    let unlocked_only: f64 = env
        .eval("return C_ArtifactUI.GetNumRelicSlots(true)")
        .unwrap();
    let slot_type: String = env.eval("return C_ArtifactUI.GetRelicSlotType(2)").unwrap();
    let locked_reason: String = env
        .eval("return C_ArtifactUI.GetRelicLockedReason(2)")
        .unwrap();
    assert!((total - 2.0).abs() < 1e-6);
    assert!((unlocked_only - 1.0).abs() < 1e-6);
    assert_eq!(slot_type, "Blood");
    assert_eq!(locked_reason, "Locked");
}

#[test]
fn get_relic_info_returns_nothing_for_unsocketed_slot() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .viewed_artifact
        .relic_slots
        .push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: None,
            name: String::new(),
            icon: String::new(),
            link: String::new(),
        });
    let nil: bool = env
        .eval("return C_ArtifactUI.GetRelicInfo(1) == nil")
        .unwrap();
    assert!(nil);
}

#[test]
fn get_relic_info_returns_four_values_for_socketed_slot() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .viewed_artifact
        .relic_slots
        .push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: None,
            name: "Hammer".to_string(),
            icon: "Interface/Icons/hammer".to_string(),
            link: "item:222".to_string(),
        });
    env.exec("name, icon, slotType, link = C_ArtifactUI.GetRelicInfo(1)")
        .unwrap();
    let name: String = env.eval("return name").unwrap();
    let slot_type: String = env.eval("return slotType").unwrap();
    let link: String = env.eval("return link").unwrap();
    assert_eq!(name, "Hammer");
    assert_eq!(slot_type, "Iron");
    assert_eq!(link, "item:222");
}

#[test]
fn get_relic_info_by_item_id_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .viewed_artifact
        .relic_info_by_item_id
        .insert(
            555,
            RelicSlotInfo {
                slot_type: "Fel".to_string(),
                locked_reason: None,
                name: "Fel Relic".to_string(),
                icon: "Interface/Icons/fel".to_string(),
                link: "item:555".to_string(),
            },
        );
    env.exec("name = C_ArtifactUI.GetRelicInfoByItemID(555)")
        .unwrap();
    let name: String = env.eval("return name").unwrap();
    assert_eq!(name, "Fel Relic");
}

#[test]
fn can_apply_cursor_relic_to_slot_requires_known_relic_and_unlocked_slot() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.relic_slots.push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: None,
            name: String::new(),
            icon: String::new(),
            link: String::new(),
        });
        st.artifact_relic_items.insert(777);
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 777,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
            applied_transmog: None,
        });
    }
    let allowed: bool = env
        .eval("return C_ArtifactUI.CanApplyCursorRelicToSlot(1)")
        .unwrap();
    assert!(allowed);
}

#[test]
fn can_apply_cursor_relic_to_slot_rejects_unknown_relic() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.relic_slots.push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: None,
            name: String::new(),
            icon: String::new(),
            link: String::new(),
        });
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 1,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
            applied_transmog: None,
        });
    }
    let allowed: bool = env
        .eval("return C_ArtifactUI.CanApplyCursorRelicToSlot(1)")
        .unwrap();
    assert!(!allowed);
}

#[test]
fn can_apply_relic_item_id_to_slot_checks_lock_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.relic_slots.push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: Some("Locked".to_string()),
            name: String::new(),
            icon: String::new(),
            link: String::new(),
        });
        st.artifact_relic_items.insert(888);
    }
    let allowed: bool = env
        .eval("return C_ArtifactUI.CanApplyRelicItemIDToSlot(888, 1)")
        .unwrap();
    assert!(!allowed);
}

#[test]
fn forge_rotation_round_trips_through_state() {
    let env = WowLuaEnv::new().expect("env");
    env.exec("C_ArtifactUI.SetForgeRotation(0.5, 1.5, 2.5)")
        .unwrap();
    let stored = env.state().borrow().viewed_artifact.forge_rotation;
    assert!((stored.0 - 0.5).abs() < 1e-6);
    env.exec("a, b, c = C_ArtifactUI.GetForgeRotation()")
        .unwrap();
    let a: f64 = env.eval("return a").unwrap();
    let c: f64 = env.eval("return c").unwrap();
    assert!((a - 0.5).abs() < 1e-6);
    assert!((c - 2.5).abs() < 1e-6);
}

#[test]
fn should_suppress_forge_rotation_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .viewed_artifact
        .suppress_forge_rotation = true;
    let suppress: bool = env
        .eval("return C_ArtifactUI.ShouldSuppressForgeRotation()")
        .unwrap();
    assert!(suppress);
}

#[test]
fn add_power_marks_known_and_fires_artifact_update() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.info = Some(sample_artifact());
        st.viewed_artifact.points_remaining = 1;
    }
    env.exec(
        r#"
        EVENTS_SEEN = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("ARTIFACT_UPDATE")
        listener:SetScript("OnEvent", function(_, event, newItem)
            table.insert(EVENTS_SEEN, event .. ":" .. tostring(newItem))
        end)
        result = C_ArtifactUI.AddPower(42)
        "#,
    )
    .unwrap();
    let result: bool = env.eval("return result").unwrap();
    let first: String = env.eval("return EVENTS_SEEN[1]").unwrap();
    assert!(result);
    assert_eq!(first, "ARTIFACT_UPDATE:false");
    assert!(
        env.state()
            .borrow()
            .viewed_artifact
            .power_known
            .contains(&42)
    );
}

#[test]
fn clear_resets_state_and_fires_artifact_close_when_present() {
    let env = WowLuaEnv::new().expect("env");
    seed_artifact(&env);
    env.exec(
        r#"
        EVENTS_SEEN = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("ARTIFACT_CLOSE")
        listener:SetScript("OnEvent", function(_, event)
            table.insert(EVENTS_SEEN, event)
        end)
        C_ArtifactUI.Clear()
        "#,
    )
    .unwrap();
    let first: String = env.eval("return EVENTS_SEEN[1]").unwrap();
    assert_eq!(first, "ARTIFACT_CLOSE");
    assert!(env.state().borrow().viewed_artifact.info.is_none());
}

#[test]
fn clear_is_silent_when_no_artifact_present() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
        EVENTS_SEEN = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("ARTIFACT_CLOSE")
        listener:SetScript("OnEvent", function(_, event)
            table.insert(EVENTS_SEEN, event)
        end)
        C_ArtifactUI.Clear()
        "#,
    )
    .unwrap();
    let count: f64 = env.eval("return #EVENTS_SEEN").unwrap();
    assert!(count.abs() < 1e-6);
}

#[test]
fn confirm_respec_refunds_points_and_clears_known_powers() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.info = Some(sample_artifact());
        st.viewed_artifact.points_remaining = 0;
        st.viewed_artifact.total_purchased_ranks = 5;
        st.viewed_artifact.power_known.insert(7);
    }
    env.exec("C_ArtifactUI.ConfirmRespec()").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.viewed_artifact.points_remaining, 5);
    assert_eq!(st.viewed_artifact.total_purchased_ranks, 0);
    assert!(st.viewed_artifact.power_known.is_empty());
}

#[test]
fn set_appearance_writes_appearance_id_to_info() {
    let env = WowLuaEnv::new().expect("env");
    seed_artifact(&env);
    env.exec("C_ArtifactUI.SetAppearance(99)").unwrap();
    let id = env
        .state()
        .borrow()
        .viewed_artifact
        .info
        .as_ref()
        .map(|info| info.artifact_appearance_id)
        .unwrap_or(-1);
    assert_eq!(id, 99);
}

#[test]
fn set_preview_appearance_clears_when_passed_nil_or_zero() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().viewed_artifact.preview_appearance = Some(5);
    env.exec("C_ArtifactUI.SetPreviewAppearance(nil)").unwrap();
    assert!(
        env.state()
            .borrow()
            .viewed_artifact
            .preview_appearance
            .is_none()
    );
    env.exec("C_ArtifactUI.SetPreviewAppearance(7)").unwrap();
    assert_eq!(
        env.state().borrow().viewed_artifact.preview_appearance,
        Some(7)
    );
    env.exec("C_ArtifactUI.SetPreviewAppearance(0)").unwrap();
    assert!(
        env.state()
            .borrow()
            .viewed_artifact
            .preview_appearance
            .is_none()
    );
}

#[test]
fn apply_cursor_relic_to_slot_writes_into_slot() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut st = env.state().borrow_mut();
        st.viewed_artifact.relic_slots.push(RelicSlotInfo {
            slot_type: "Iron".to_string(),
            locked_reason: None,
            name: String::new(),
            icon: String::new(),
            link: String::new(),
        });
        st.artifact_relic_items.insert(444);
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 444,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
            applied_transmog: None,
        });
    }
    env.exec("C_ArtifactUI.ApplyCursorRelicToSlot(1)").unwrap();
    let st = env.state().borrow();
    let slot = &st.viewed_artifact.relic_slots[0];
    assert_eq!(slot.link, "item:444");
    assert!(slot.icon.contains("inv_relic_444"));
    assert!(slot.name.contains("444"));
}
