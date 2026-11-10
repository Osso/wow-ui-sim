#![cfg(feature = "client-mists")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::engraving_inputs::{EngravingData, EngravingInputs};
use wow_ui_sim::lua_api::WowLuaEnv;

fn supplied_rune() -> EngravingData {
    EngravingData {
        skill_line_ability_id: 41001,
        item_enchantment_id: 42002,
        name: "Supplied test rune".into(),
        icon_texture: 134400,
        equipment_slot: 5,
        level: 12,
        learned_ability_spell_ids: vec![43003, 44004],
    }
}

fn supplied_inputs() -> EngravingInputs {
    let mut inputs = EngravingInputs {
        enabled: Some(true),
        mode_enabled: true,
        current_rune: Some(None),
        ..EngravingInputs::default()
    };
    for slot in 1..=36 {
        inputs.inventory_eligibility.insert((0, slot), true);
        inputs.inventory_runes.insert((0, slot), None);
        inputs.current_cast_eligibility.insert((0, slot), false);
    }
    inputs.inventory_runes.insert((0, 1), Some(supplied_rune()));
    inputs
}

fn supplied_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.state().borrow_mut().engraving = supplied_inputs();
    load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_UIParentPanelManager",
            "Blizzard_MainMenuBarBagButtons",
            "Blizzard_UIPanels_Game",
        ],
        &[],
    );
    open_original_bag(&env);
    assert_no_engraving_errors(&env);
    env
}

fn open_original_bag(env: &WowLuaEnv) {
    env.exec(r#"
        SetCVar("alwaysShowRuneIcons", "0")
        A_Admin.ClearBags()
        A_Admin.AddBagItem(0, 1, 6948, 1)
        OpenBag(0)
        assert(ContainerFrame1:GetID() == 0 and ContainerFrame1:IsShown())
        local source = debug.getinfo(ContainerFrame_RefreshRuneIcons, "S").source:gsub("\\", "/")
        assert(source:find("Blizzard_UIPanels_Game/Classic/ContainerFrame_Shared.lua", 1, true), source)
        EngravingBagButtons = {}
        for index = 1, ContainerFrame1.size do
            local button = _G["ContainerFrame1Item" .. index]
            EngravingBagButtons[button:GetID()] = button
        end
        assert(EngravingBagButtons[1] and EngravingBagButtons[2])
    "#).expect("load and open the original Mists bag with pre-startup supplied input");
}

fn assert_no_engraving_errors(env: &WowLuaEnv) {
    let engraving_errors: Vec<_> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("C_Engraving"))
        .cloned()
        .collect();
    assert!(engraving_errors.is_empty(), "{engraving_errors:#?}");
}

#[test]
fn original_bag_rune_icons_follow_supplied_mode_availability_and_slots() {
    let env = supplied_env();
    env.exec(
        r#"
        ContainerFrame_RefreshRuneIcons(ContainerFrame1, C_Engraving.GetEngravingModeEnabled())
        assert(EngravingBagButtons[1].subicon:IsShown())
        assert(EngravingBagButtons[1].subicon:GetTexture() == 134400)
        assert(EngravingBagButtons[2].subicon:GetTexture() == 136509)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().engraving.mode_enabled = false;
    env.exec(
        r#"
        ContainerFrame_RefreshRuneIcons(ContainerFrame1, C_Engraving.GetEngravingModeEnabled())
        assert(not EngravingBagButtons[1].subicon:IsShown())
        SetCVar("alwaysShowRuneIcons", "1")
        ContainerFrame_RefreshRuneIcons(ContainerFrame1, C_Engraving.GetEngravingModeEnabled())
        assert(EngravingBagButtons[1].subicon:IsShown())
        SetCVar("alwaysShowRuneIcons", "0")
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().engraving.refresh_requests, 1);
    {
        let mut state = env.state().borrow_mut();
        state.engraving.mode_enabled = true;
        state.engraving.inventory_eligibility.insert((0, 1), false);
    }
    env.exec(
        r#"
        ContainerFrame_RefreshRuneIcons(ContainerFrame1, C_Engraving.GetEngravingModeEnabled())
        assert(not EngravingBagButtons[1].subicon:IsShown())
        assert(EngravingBagButtons[2].subicon:IsShown())
    "#,
    )
    .unwrap();
    env.state().borrow_mut().engraving.enabled = Some(false);
    env.exec(
        r#"
        ContainerFrame_RefreshRuneIcons(ContainerFrame1, C_Engraving.GetEngravingModeEnabled())
        assert(not EngravingBagButtons[2].subicon:IsShown())
    "#,
    )
    .unwrap();
}

#[test]
fn original_bag_targeting_projects_supplied_current_cast_eligibility() {
    let env = supplied_env();
    env.exec(r#"
        ContainerFrame_EngravingTargetingModeChanged(ContainerFrame1, C_Engraving.GetCurrentRuneCast() ~= nil)
        assert(not EngravingBagButtons[1].icon:IsDesaturated())
    "#).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.engraving.current_rune = Some(Some(supplied_rune()));
        state
            .engraving
            .current_cast_eligibility
            .insert((0, 1), true);
    }
    env.exec(r#"
        ContainerFrame_EngravingTargetingModeChanged(ContainerFrame1, C_Engraving.GetCurrentRuneCast() ~= nil)
        assert(not EngravingBagButtons[1].icon:IsDesaturated())
        assert(EngravingBagButtons[2].icon:IsDesaturated())
    "#).unwrap();
    env.state().borrow_mut().engraving.current_rune = Some(None);
    env.exec(r#"
        ContainerFrame_EngravingTargetingModeChanged(ContainerFrame1, C_Engraving.GetCurrentRuneCast() ~= nil)
        assert(not EngravingBagButtons[2].icon:IsDesaturated())
    "#).unwrap();
}

#[test]
fn mists_r21_engraving_setter_dispatches_repeated_values_to_original_bag() {
    let env = supplied_env();
    env.exec(r#"
        local source = debug.getinfo(ContainerFrame_OnEvent, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_UIPanels_Game/Classic/ContainerFrame_Shared.lua', 1, true), source)
        local observations = {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('ENGRAVING_MODE_CHANGED')
        observer:SetScript('OnEvent', function(_, event, enabled, ...)
            assert(event == 'ENGRAVING_MODE_CHANGED' and select('#', ...) == 0)
            assert(C_Engraving.GetEngravingModeEnabled() == enabled, 'flag is set before event')
            observations[#observations + 1] = enabled
        end)
        C_Engraving.SetEngravingModeEnabled(false)
        assert(#observations == 1 and observations[1] == false, 'synchronous false event')
        assert(not EngravingBagButtons[1].subicon:IsShown(), 'original event consumer hides rune')
        C_Engraving.SetEngravingModeEnabled(true)
        assert(#observations == 2 and observations[2] == true)
        assert(EngravingBagButtons[1].subicon:IsShown(), 'original event consumer shows rune')
        C_Engraving.SetEngravingModeEnabled(true)
        assert(#observations == 3 and observations[3] == true, 'unchanged flag still fires')
        C_Engraving.SetEngravingModeEnabled(false)
        assert(#observations == 4 and observations[4] == false)
        assert(not EngravingBagButtons[1].subicon:IsShown())
    "#).unwrap();
    let other = WowLuaEnv::new().unwrap();
    assert!(
        !other
            .eval::<bool>("return C_Engraving.GetEngravingModeEnabled()")
            .unwrap()
    );
}

#[test]
fn direct_engraving_queries_project_records_and_reject_unsupplied_inputs() {
    // Native mode default is false: the only pinned producer is
    // Blizzard_EngravingUI OnShow/OnHide, which sets true/false.
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local function missing(call, name)
            local ok, message = pcall(call)
            assert(not ok and message:find("C_Engraving." .. name, 1, true), tostring(message))
            assert(message:find("unsupplied", 1, true), message)
        end
        missing(function() C_Engraving.IsInventorySlotEngravable(0, 1) end, "IsInventorySlotEngravable")
        missing(function() C_Engraving.GetRuneForInventorySlot(0, 1) end, "GetRuneForInventorySlot")
        missing(function() C_Engraving.IsInventorySlotEngravableByCurrentRuneCast(0, 1) end, "IsInventorySlotEngravableByCurrentRuneCast")
        assert(C_Engraving.IsEngravingEnabled() == false)
        assert(C_Engraving.GetCurrentRuneCast() == nil)
        assert(C_Engraving.GetEngravingModeEnabled() == false)
    "#).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.engraving.enabled = Some(false);
        state.engraving.mode_enabled = true;
        state.engraving.current_rune = Some(Some(supplied_rune()));
        state
            .engraving
            .inventory_runes
            .insert((0, 1), Some(supplied_rune()));
        state.engraving.inventory_runes.insert((0, 2), None);
        state.engraving.inventory_eligibility.insert((0, 1), false);
        state
            .engraving
            .current_cast_eligibility
            .insert((0, 1), true);
    }
    env.exec(
        r#"
        assert(C_Engraving.IsEngravingEnabled() == false)
        assert(C_Engraving.GetEngravingModeEnabled() == true)
        assert(C_Engraving.IsInventorySlotEngravable(0, 1) == false)
        assert(C_Engraving.IsInventorySlotEngravableByCurrentRuneCast(0, 1) == true)
        local rune = C_Engraving.GetRuneForInventorySlot(0, 1)
        assert(rune.skillLineAbilityID == 41001 and rune.itemEnchantmentID == 42002)
        assert(rune.name == "Supplied test rune" and rune.iconTexture == 134400)
        assert(rune.equipmentSlot == 5 and rune.level == 12)
        assert(#rune.learnedAbilitySpellIDs == 2 and rune.learnedAbilitySpellIDs[2] == 44004)
        assert(C_Engraving.GetCurrentRuneCast().skillLineAbilityID == 41001)
        rune.learnedAbilitySpellIDs[2] = 0
        assert(C_Engraving.GetRuneForInventorySlot(0, 1).learnedAbilitySpellIDs[2] == 44004)
        assert(select('#', C_Engraving.GetRuneForInventorySlot(0, 2)) == 1)
        assert(C_Engraving.GetRuneForInventorySlot(0, 2) == nil)
        assert(select('#', C_Engraving.RefreshRunesList()) == 0)
        C_Engraving.RefreshRunesList()
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().engraving.refresh_requests, 2);
    env.state().borrow_mut().engraving.mode_enabled = false;
    assert!(
        !env.eval::<bool>("return C_Engraving.GetEngravingModeEnabled()")
            .unwrap()
    );
    let other = WowLuaEnv::new().unwrap();
    assert_eq!(other.state().borrow().engraving.enabled, Some(false));
    assert_eq!(other.state().borrow().engraving.refresh_requests, 0);
    assert!(
        !other
            .eval::<bool>("return C_Engraving.GetEngravingModeEnabled()")
            .unwrap()
    );
    assert!(
        other
            .eval::<bool>("return C_Engraving.GetCurrentRuneCast() == nil")
            .unwrap()
    );
}
