#![cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]

use wow_ui_sim::lua_api::WowLuaEnv;

type TalentStateSnapshot = (bool, bool, bool, i32, i32, i32, Option<i32>, Option<i32>);

#[cfg(feature = "client-retail")]
#[test]
fn retail_supplied_pvp_talent_state_reaches_open_player_spells_panel() {
    with_player_spells_panel(|env| {
        supply_selected_talent_state(env);
        assert_eq!(
            read_talent_state(env),
            (true, true, false, 2, 123, 456, Some(456), Some(30))
        );
        let level: Option<i32> = env
            .eval("return C_SpecializationInfo.GetPvpTalentUnlockLevel(5573)")
            .expect("query the Retail catalog unlock level");
        assert_eq!(level, Some(25));
        let missing: Option<i32> = env
            .eval("return C_SpecializationInfo.GetPvpTalentUnlockLevel(2147483647)")
            .expect("query a missing Retail talent row");
        assert_eq!(missing, None);
    });
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_supplied_pvp_talent_state_reaches_open_player_spells_panel() {
    with_player_spells_panel(|env| {
        supply_selected_talent_state(env);
        env.exec(
            r#"
            A_Admin.SetPvpTalentUnlockLevel(123, 25)
            A_Admin.SetPvpTalentSlotInfo(1, true, 60, { 123, 456 })
            A_Admin.SetPvpTalentSlotInfo(2, true, 60, { 456, 123 })
            assert(C_SpecializationInfo.GetPvpTalentUnlockLevel(123) == 25)
            assert(C_SpecializationInfo.GetPvpTalentUnlockLevel(999) == nil)
            assert(C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(99) == nil)
            local firstSlot = C_SpecializationInfo.GetPvpTalentSlotInfo(1)
            local secondSlot = C_SpecializationInfo.GetPvpTalentSlotInfo(2)
            assert(firstSlot.selectedTalentID == 123 and #firstSlot.availableTalentIDs == 2)
            assert(secondSlot.selectedTalentID == 456 and #secondSlot.availableTalentIDs == 2)
            "#,
        )
        .expect("supply Forever talent unlock and slot records");
        assert_eq!(
            read_talent_state(env),
            (true, true, false, 2, 123, 456, Some(456), Some(30))
        );
    });
}

fn with_player_spells_panel(assert_state: impl FnOnce(&WowLuaEnv)) {
    let env = crate::common::panel_fixtures::setup_env();
    env.exec(
        r#"
        assert(PlayerSpellsUtil and PlayerSpellsUtil.ToggleClassTalentFrame)
        PlayerSpellsUtil.ToggleClassTalentFrame()
        assert(PlayerSpellsFrame and PlayerSpellsFrame:IsShown())
        assert(PlayerSpellsFrame.TalentsFrame:IsShown())
        "#,
    )
    .expect("open the original Player Spells talents panel");
    assert_state(&env);
}

fn supply_selected_talent_state(env: &WowLuaEnv) {
    env.exec(
        r#"
        local talentUpdates, inspectUpdates = 0, 0
        local observer = CreateFrame("Frame")
        observer:RegisterEvent("PLAYER_PVP_TALENT_UPDATE")
        observer:RegisterEvent("INSPECT_HONOR_UPDATE")
        observer:SetScript("OnEvent", function(_, event)
            if event == "PLAYER_PVP_TALENT_UPDATE" then
                talentUpdates = talentUpdates + 1
            else
                inspectUpdates = inspectUpdates + 1
            end
        end)
        A_Admin.SetPvpTalentInfo(123, "First", 111, true, true, 1001, true, true, false, false, nil)
        A_Admin.SetPvpTalentInfo(456, "Second", 222, true, true, 1002, true, true, false, false, nil)
        A_Admin.SetPvpTalentSlotUnlockLevel(2, 30)
        A_Admin.SetPvpTalentAlertStatus(true, false)
        A_Admin.SetSelectedPvpTalentIDs({ 123, 456 })
        A_Admin.SetInspectSelectedPvpTalent("target", 2, 456)
        assert(select('#', C_SpecializationInfo.SetPvpTalentLocked(123, true)) == 0)
        C_SpecializationInfo.SetPvpTalentLocked(456, true)
        C_SpecializationInfo.SetPvpTalentLocked(456, false)
        assert(talentUpdates > 0)
        assert(inspectUpdates == 1)
        "#,
    )
    .expect("supply selected, inspected and locked PvP talents");
}

fn read_talent_state(env: &WowLuaEnv) -> TalentStateSnapshot {
    env.eval(
        r#"
        local selected = C_SpecializationInfo.GetAllSelectedPvpTalentIDs()
        local info = C_SpecializationInfo.GetPvpTalentInfo(123)
        assert(info and info.talentID == 123 and info.name == "First" and info.icon == 111
            and info.selected and info.available and info.spellID == 1001 and info.unlocked
            and info.known and not info.grantedByAura and not info.dependenciesUnmet
            and info.dependenciesUnmetReason == nil)
        local alertOne, alertTwo = C_SpecializationInfo.GetPvpTalentAlertStatus()
        assert(select('#', C_SpecializationInfo.GetPvpTalentAlertStatus()) == 2)
        assert(type(alertOne) == "boolean" and type(alertTwo) == "boolean")
        assert(alertOne == true and alertTwo == false)
        assert(C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(99) == nil)
        return PlayerSpellsFrame:IsShown(), C_SpecializationInfo.IsPvpTalentLocked(123),
            C_SpecializationInfo.IsPvpTalentLocked(456), #selected, selected[1], selected[2],
            C_SpecializationInfo.GetInspectSelectedPvpTalent("target", 2),
            C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(2)
        "#,
    )
    .expect("read supplied PvP state through client APIs")
}
