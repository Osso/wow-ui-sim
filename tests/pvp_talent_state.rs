#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

type TalentStateSnapshot = (bool, bool, bool, i32, i32, i32, Option<i32>, Option<i32>);

#[cfg(feature = "client-retail")]
#[test]
fn retail_supplied_pvp_talent_state_reaches_open_player_spells_panel() {
    with_player_spells_panel(|env| {
        supply_selected_talent_state(env);
        assert_player_spells_slot_icons(env);
        assert_eq!(
            read_talent_state(env),
            (true, true, false, 2, 123, 456, Some(456), Some(30))
        );
        assert_pvp_talent_info_by_id(env);
        assert_inspect_pvp_slot_icon(env);
        assert_locked_talent_refresh_event(env);
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
fn forever_supplied_pvp_talent_state_reaches_legacy_lua_api() {
    let env = crate::common::panel_fixtures::setup_env();
    supply_selected_talent_state(&env);
    env.exec(
        r#"
            A_Admin.SetPvpTalentUnlockLevel(123, 25)
            assert(C_SpecializationInfo.GetPvpTalentUnlockLevel(123) == 25)
            assert(C_SpecializationInfo.GetPvpTalentUnlockLevel(999) == nil)
            assert(C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(99) == nil)
            local firstSlot = C_SpecializationInfo.GetPvpTalentSlotInfo(1)
            local secondSlot = C_SpecializationInfo.GetPvpTalentSlotInfo(2)
            assert(firstSlot.enabled and firstSlot.level == 60)
            assert(firstSlot.selectedTalentID == 123 and #firstSlot.availableTalentIDs == 2)
            assert(secondSlot.enabled and secondSlot.level == 60)
            assert(secondSlot.selectedTalentID == 456 and #secondSlot.availableTalentIDs == 2)
            "#,
    )
    .expect("supply Forever talent unlock and slot records");
    assert_eq!(
        read_talent_state(&env),
        (false, true, false, 2, 123, 456, Some(456), Some(30))
    );
    assert_pvp_talent_info_by_id(&env);
    assert_locked_talent_refresh_event(&env);
}

#[cfg(feature = "client-retail")]
fn assert_player_spells_slot_icons(env: &WowLuaEnv) {
    env.exec(
        r#"
        local tray = PlayerSpellsFrame.TalentsFrame.PvPTalentSlotTray
        assert(tray and tray:IsShown(), "real Player Spells PvP slot tray is visible")
        local first, second = tray.Slots[1], tray.Slots[2]
        local slotInfo = C_SpecializationInfo.GetPvpTalentSlotInfo(1)
        local selected = C_SpecializationInfo.GetAllSelectedPvpTalentIDs()
        local firstTalent, firstIcon = first:GetSelectedTalent(), first.Texture:GetTexture()
        local secondTalent, secondIcon = second:GetSelectedTalent(), second.Texture:GetTexture()
        assert(firstTalent == 123, "first slot talent: " .. tostring(firstTalent)
            .. ", slotInfo type: " .. type(slotInfo)
            .. ", slotInfo selected: " .. tostring(slotInfo and slotInfo.selectedTalentID)
            .. ", selected list: " .. tostring(selected[1])
            .. ", active group: " .. tostring(C_SpecializationInfo.GetActiveSpecGroup(true))
            .. ", inspect unit: " .. tostring(first:GetInspectUnit()))
        assert(firstIcon == 111, "first slot icon: " .. tostring(firstIcon))
        assert(secondTalent == 456, "second slot talent: " .. tostring(secondTalent))
        assert(secondIcon == 222, "second slot icon: " .. tostring(secondIcon))
        "#,
    )
    .expect("read supplied selected talents from the original Player Spells slot tray");
}

#[cfg(feature = "client-retail")]
fn assert_inspect_pvp_slot_icon(env: &WowLuaEnv) {
    env.exec("A_Admin.SetInspectSelectedPvpTalent('player', 2, 456)")
        .expect("supply the player inspect selection");
    let ui = crate::common::panel_fixtures::blizzard_ui_dir();
    crate::common::load_required_blizzard_addon(env, &ui, "Blizzard_InspectUI");
    env.exec(
        r#"
        InspectFrame_Show("player")
        assert(INSPECTED_UNIT == "player")
        ShowUIPanel(InspectFrame)
        FireEvent("INSPECT_READY", UnitGUID("player"))
        InspectSwitchTabs(2)
        assert(InspectFrame:IsShown() and InspectPVPFrame:IsShown())
        local slot = InspectPVPFrame.Slots[2]
        slot:Update()
        assert(slot.talentID == 456 and slot.Texture:GetTexture() == 222)
        "#,
    )
    .expect("read the inspected talent from the original Inspect PvP slot");
}

fn assert_locked_talent_refresh_event(env: &WowLuaEnv) {
    env.exec(
        r#"
        local updates = 0
        local observer = CreateFrame("Frame")
        observer:RegisterEvent("PLAYER_PVP_TALENT_UPDATE")
        observer:SetScript("OnEvent", function() updates = updates + 1 end)
        local before = updates
        assert(select('#', C_SpecializationInfo.SetPvpTalentLocked(123, true)) == 0)
        assert(updates == before + 1)
        before = updates
        C_SpecializationInfo.SetPvpTalentLocked(123, false)
        assert(updates == before + 1)
        "#,
    )
    .expect("lock changes dispatch the inferred PvP refresh event");
}

#[cfg(feature = "client-retail")]
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

const SUPPLY_SELECTED_TALENT_STATE_LUA: &str = r#"
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
        A_Admin.SetPvpTalentInfo(123, "First", 111, true, true, 1001, true, true, false, false, nil, 2, 3)
        A_Admin.SetPvpTalentInfo(456, "Second", 222, true, true, 1002, true, true, false, false, nil, 4, 5)
        A_Admin.SetPvpTalentInfo(789, "Nullable", nil, false, false, 1003, nil, false, false, false, nil)
        A_Admin.SetPvpTalentInfo(0, "Zero", 999, false, false, 1000, false, false, false, false, nil)
        A_Admin.SetPvpTalentSlotUnlockLevel(2, 30)
        A_Admin.SetPvpTalentAlertStatus(true, false)
        A_Admin.SetSelectedPvpTalentIDs({ 123, 456 })
        A_Admin.SetPvpTalentSlotInfo(1, true, 60, { 123, 456 })
        A_Admin.SetPvpTalentSlotInfo(2, true, 60, { 456, 123 })
        A_Admin.SetInspectSelectedPvpTalent("target", 2, 456)
        assert(select('#', C_SpecializationInfo.SetPvpTalentLocked(123, true)) == 0)
        C_SpecializationInfo.SetPvpTalentLocked(456, true)
        C_SpecializationInfo.SetPvpTalentLocked(456, false)
        assert(talentUpdates > 0)
        assert(inspectUpdates == 1)
        "#;

fn supply_selected_talent_state(env: &WowLuaEnv) {
    env.exec(SUPPLY_SELECTED_TALENT_STATE_LUA)
        .expect("supply selected, inspected and locked PvP talents");
}

fn assert_pvp_talent_info_by_id(env: &WowLuaEnv) {
    env.exec(
        r#"
        local count = select('#', GetPvpTalentInfoByID(456))
        local talentID, name, icon, selected, available, spellID, unlocked, tier, column, known, grantedByAura =
            GetPvpTalentInfoByID(456)
        assert(count == 11)
        assert(talentID == 456 and name == "Second" and icon == 222 and selected
            and available and spellID == 1002 and unlocked and tier == 4 and column == 5
            and known and not grantedByAura)
        assert(select('#', GetPvpTalentInfoByID(0)) == 0)
        assert(select('#', GetPvpTalentInfoByID(2147483647)) == 0)
        local ok, err = pcall(GetPvpTalentInfoByID, "456")
        assert(not ok and tostring(err):find("Usage: GetPvpTalentInfoByID", 1, true))
        local nullID, nullName, nullIcon, nullSelected, nullAvailable, nullSpellID,
            nullUnlocked, defaultTier, defaultColumn, nullKnown, nullGranted = GetPvpTalentInfoByID(789)
        assert(nullID == 789 and nullName == "Nullable" and nullIcon == nil
            and not nullSelected and not nullAvailable and nullSpellID == 1003
            and nullUnlocked == nil and defaultTier == 0 and defaultColumn == 0
            and not nullKnown and not nullGranted)
        assert(select('#', GetPvpTalentInfoByID(789)) == 11)
        "#,
    )
    .expect("read the R88eu PvP talent info tuple");
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
        return PlayerSpellsFrame and PlayerSpellsFrame:IsShown() or false,
            C_SpecializationInfo.IsPvpTalentLocked(123),
            C_SpecializationInfo.IsPvpTalentLocked(456), #selected, selected[1], selected[2],
            C_SpecializationInfo.GetInspectSelectedPvpTalent("target", 2),
            C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(2)
        "#,
    )
    .expect("read supplied PvP state through client APIs")
}
