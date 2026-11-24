//! M5 original pet action consumers and generated service contracts.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]

use crate::cfactory_support::{documented_env, load_original};

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]
fn pet_env() -> wow_ui_sim::lua_api::WowLuaEnv {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    // Mode presentation is outside the pet-action queries; no C_* method is replaced.
    env.exec("UIModeUtil = { RegisterMode = function() end, CreateExtendedBlocklist = function() return {} end }; StaticPopupDialogs = {}")
        .unwrap();
    load_original(&env, "Blizzard_FrameXML/SharedPetBattleTemplates.lua");
    load_original(&env, "Blizzard_PetBattleUI/Shared/Blizzard_PetBattleUI.lua");
    env
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]
#[test]
fn cfactory_m5_original_pet_switch_disabled_without_legality_model() {
    let env = pet_env();
    env.exec(
        r#"
        local button = CreateFrame('Button')
        button.actionType = Enum.BattlePetAction.SwitchPet
        PetBattleActionButton_UpdateState(button)
        assert(not button:IsEnabled())
    "#,
    )
    .unwrap();
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]
#[test]
fn cfactory_m5_original_pet_skip_disabled_without_legality_model() {
    let env = pet_env();
    env.exec(
        r#"
        local button = CreateFrame('Button')
        button.actionType = Enum.BattlePetAction.Skip
        PetBattleActionButton_UpdateState(button)
        assert(not button:IsEnabled())
    "#,
    )
    .unwrap();
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]
#[test]
fn cfactory_m5_pet_icon_projects_species_catalog() {
    let env = documented_env("PetBattlesDocumentation.lua", "GetIcon");
    let (species, icon) = {
        let state = env.state();
        let sim = state.borrow();
        let pet = sim.world.pets.first().expect("SETUP: pet journal entry");
        (pet.species_id as i32, pet.icon)
    };
    env.state().borrow_mut().pet_battles.player_pets[0].species_id = species;
    let actual: i64 = env.eval("return C_PetBattles.GetIcon(1, 1)").unwrap();
    assert_eq!(actual, i64::from(icon));
    env.state().borrow_mut().pet_battles.player_pets.clear();
    env.exec("assert(C_PetBattles.GetIcon(1, 1) == nil)")
        .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn cfactory_m5_housing_bootstrap_delivers_empty_owned_house_list() {
    let env = documented_env("HousingUIDocumentation.lua", "GetPlayerOwnedHouses");
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('PLAYER_HOUSE_LIST_UPDATED')
        frame:SetScript('OnEvent', function(_, _, houses) OwnedHouses = houses end)
        C_Housing.GetPlayerOwnedHouses()
    "#,
    )
    .unwrap();
    env.process_timers().unwrap();
    env.exec("assert(type(OwnedHouses) == 'table' and #OwnedHouses == 0)")
        .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m5_forever_bank_tabs_documented_table() {
    let env = documented_env("BankDocumentation.lua", "FetchPurchasedBankTabData");
    env.exec("assert(type(C_Bank.FetchPurchasedBankTabData(Enum.BankType.Character)) == 'table')")
        .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m5_forever_bank_bag_capability_documented_boolean() {
    let env = documented_env("BankDocumentation.lua", "ShouldUsePlayerBagsInBank");
    env.exec("assert(type(C_Bank.ShouldUsePlayerBagsInBank()) == 'boolean')")
        .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cfactory_m5_forever_valid_sets_use_supplied_catalog() {
    use wow_ui_sim::c_api::c_transmog_sets::TransmogSetInfo;

    let env = documented_env("TransmogSetsDocumentation.lua", "HasAnyValidSets");
    env.state().borrow_mut().transmog_sets.entries.clear();
    env.exec("assert(C_TransmogSets.HasAnyValidSets() == false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .transmog_sets
        .entries
        .push(TransmogSetInfo {
            set_id: 812,
            name: "Fixture set".into(),
            valid_for_character: true,
            ..Default::default()
        });
    env.state()
        .borrow_mut()
        .transmog_set_filters
        .insert(1, false);
    env.exec("assert(C_TransmogSets.HasAnyValidSets() == true)")
        .unwrap();
    env.state().borrow_mut().transmog_sets.entries[0].valid_for_character = false;
    env.exec("assert(C_TransmogSets.HasAnyValidSets() == false)")
        .unwrap();
}
