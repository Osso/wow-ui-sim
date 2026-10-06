#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::c_api::c_housing::editor::complete_mode_change;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1127_house_editor_success_is_pending_until_host_reply() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.housing.editor.status_available = true;
        sim.housing.editor.availability = 0;
        sim.housing.editor.mode_availability[1] = 0;
        sim.housing.editor.mode_availability[2] = 2;
        sim.events.drain();
    }
    env.exec(r#"
        assert(C_HouseEditor.GetHouseEditorAvailability() == Enum.HousingResult.Success)
        assert(C_HouseEditor.IsHouseEditorStatusAvailable())
        assert(C_HouseEditor.EnterHouseEditor() == Enum.HousingResult.Success)
        assert(not C_HouseEditor.IsHouseEditorActive())
        assert(not pcall(C_HouseEditor.ActivateHouseEditorMode, 9))
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        assert_eq!(sim.housing.editor.pending_mode, Some(1));
        assert!(sim.events.is_empty());
        complete_mode_change(&mut sim, 0);
    }
    env.exec(r#"
        assert(C_HouseEditor.IsHouseEditorActive())
        assert(C_HouseEditor.IsHouseEditorModeActive(1))
        assert(C_HouseEditor.ActivateHouseEditorMode(2) == Enum.HousingResult.ActionLockedByCombat)
        assert(C_HouseEditor.IsHouseEditorModeActive(1))
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        assert_eq!(sim.housing.editor.pending_mode, None);
        let event = sim.events.drain().pop().unwrap();
        assert_eq!(event.name, "HOUSE_EDITOR_MODE_CHANGED");
        assert!(matches!(event.args.as_slice(), [wow_ui_sim::event::EventArg::Number(1.0)]));
        sim.housing.editor.mode_availability[2] = 0;
    }
    env.exec("assert(C_HouseEditor.ActivateHouseEditorMode(2) == 0)").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        complete_mode_change(&mut sim, 2);
        assert_eq!(sim.housing.active_house_editor_mode, 1);
        assert_eq!(sim.events.drain().pop().unwrap().name, "HOUSE_EDITOR_MODE_CHANGE_FAILURE");
    }
    env.exec("C_HouseEditor.LeaveHouseEditor(); assert(not C_HouseEditor.IsHouseEditorActive())").unwrap();
    assert_eq!(env.state().borrow().housing.editor.pending_mode, None);
}


#[test]
fn p1127_house_editor_does_not_grant_unconfigured_editing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_HouseEditor.IsHouseEditorActive() == false)
        assert(C_HouseEditor.IsHouseEditorModeActive(1) == false)
        assert(C_HouseEditor.IsHouseEditorStatusAvailable() == false)
        assert(C_HouseEditor.GetHouseEditorAvailability() == Enum.HousingResult.GenericFailure)
        assert(C_HouseEditor.GetHouseEditorModeAvailability(1) == Enum.HousingResult.GenericFailure)
        assert(C_HouseEditor.EnterHouseEditor() == Enum.HousingResult.GenericFailure)
        assert(C_HouseEditor.ActivateHouseEditorMode(1) == Enum.HousingResult.GenericFailure)
        C_HouseEditor.LeaveHouseEditor()
        assert(C_HouseEditor.IsHouseEditorActive() == false)
    "#).unwrap();
}
