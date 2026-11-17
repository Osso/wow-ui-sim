#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness;
use wow_ui_sim::c_api::store_vas_model::{StoreVasCharacterInput, StoreVasRealmInput};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const REALM_AND_CHARACTER_CONSUMER: &str = r#"
    local source = debug.getinfo(StoreVASValidationFrame_Init, "S")
    assert(source.source:find("Blizzard_Shared_StoreUISecure.lua", 1, true))

    local panel = StoreVASValidationFrame
    panel.productInfo = {
        sharedData = {
            vasServiceType = Enum.VasServiceType.NameChange,
            name = "Fixture service",
            instructions = "",
        },
    }
    StoreVASValidationFrame_Init(panel)

    local selectors = panel.CharacterSelectionFrame
    assert(selectors.RealmSelector.Text:GetText() == "Azeroth")
    assert(selectors.CharacterSelector.Text:GetText():find("Simhero", 1, true))
    assert(not selectors.NoEligibleCharactersErrorMessage:IsShown())
    VASCharacterSelectionCharacterSelector_Callback(2)
    assert(selectors.SelectedCharacterName:GetText() == "Simalt")

    VASCharacterSelectionRealmSelector_Callback(TEST_VAS_REALMS[2])
    assert(selectors.RealmSelector.Text:GetText() == "Kalimdor")
    assert(selectors.CharacterSelector.Text:GetText():find("KalimdorHero", 1, true))
    VASCharacterSelectionCharacterSelector_Callback(1)
    assert(selectors.SelectedCharacterName:GetText() == "KalimdorHero")

    VASCharacterSelectionRealmSelector_Callback(TEST_VAS_REALMS[3])
    assert(selectors.RealmSelector.Text:GetText() == "Empty")
    assert(selectors.NoEligibleCharactersErrorMessage:IsShown())
"#;

fn load_store_ui() -> WowLuaEnv {
    let ui_dir = blizzard_addon_harness::blizzard_ui_dir();
    let env = blizzard_addon_harness::new_blizzard_addon_glue_env(&ui_dir);
    let loaded = blizzard_addon_harness::load_blizzard_addon_closure_for_screen_into_env(
        &env,
        &ui_dir,
        ScreenKind::CharacterSelect,
        &["Blizzard_StoreUI", "Blizzard_GlueXML"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_StoreUI"));
    assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
    env
}

const REALM_AND_CHARACTER_INPUTS: &str = r#"
    TEST_VAS_REALMS = {
        { virtualRealmAddress = 101, realmName = "Azeroth", characterCount = 2, pvp = false, rp = false },
        { virtualRealmAddress = 102, realmName = "Kalimdor", characterCount = 1, pvp = true, rp = false },
        { virtualRealmAddress = 303, realmName = "Empty", characterCount = 0, pvp = false, rp = true },
    }
    __wow_store_state.realms = TEST_VAS_REALMS
    __wow_store_realms = TEST_VAS_REALMS
    __wow_store_characters = {
        [101] = {
            { guid = 501001, name = "Simhero", realmName = "Azeroth", currentServer = 101,
              classFileName = "WARRIOR", className = "Warrior", level = 70,
              raceName = "Human", faction = 0, wowAccount = 1001, createScreenIconAtlas = "" },
            { guid = 501002, name = "Simalt", realmName = "Azeroth", currentServer = 101,
              classFileName = "MAGE", className = "Mage", level = 60,
              raceName = "Dwarf", faction = 0, wowAccount = 1001, createScreenIconAtlas = "" },
        },
        [102] = {
            { guid = 502001, name = "KalimdorHero", realmName = "Kalimdor", currentServer = 102,
              classFileName = "PRIEST", className = "Priest", level = 70,
              raceName = "Night Elf", faction = 1, wowAccount = 2001, createScreenIconAtlas = "" },
        },
    }
"#;

fn realm_inputs() -> Vec<StoreVasRealmInput> {
    vec![
        realm_input(101, "Azeroth", 2, false, false),
        realm_input(102, "Kalimdor", 1, true, false),
        realm_input(303, "Empty", 0, false, true),
    ]
}

fn realm_input(address: u32, name: &str, count: u32, pvp: bool, rp: bool) -> StoreVasRealmInput {
    StoreVasRealmInput {
        virtual_realm_address: address,
        realm_name: name.into(),
        character_count: count,
        pvp: Some(pvp),
        rp: Some(rp),
    }
}

fn character_inputs() -> Vec<StoreVasCharacterInput> {
    vec![simhero(), simalt(), kalimdor_mage()]
}

fn simhero() -> StoreVasCharacterInput {
    StoreVasCharacterInput {
        realm_address: 101,
        realm_name: Some("Azeroth".into()),
        guid: Some("Player-101-1-A".into()),
        guid_order: Some(1),
        name: Some("Simhero".into()),
        class_name: Some("Warrior".into()),
        race_name: Some("Human".into()),
        level: 70,
        class_file_name: Some("WARRIOR".into()),
        race_file_name: Some("Human".into()),
        wow_account: 1001,
        current_server: 101,
        faction: 0,
        sex: 1,
        create_screen_icon_atlas: Some("".into()),
    }
}

fn simalt() -> StoreVasCharacterInput {
    StoreVasCharacterInput {
        realm_address: 101,
        realm_name: Some("Azeroth".into()),
        guid: Some("Player-101-2-B".into()),
        guid_order: Some(2),
        name: Some("Simalt".into()),
        class_name: Some("Mage".into()),
        race_name: Some("Dwarf".into()),
        level: 60,
        class_file_name: Some("MAGE".into()),
        race_file_name: Some("Dwarf".into()),
        wow_account: 1001,
        current_server: 101,
        faction: 0,
        sex: 1,
        create_screen_icon_atlas: Some("".into()),
    }
}

fn kalimdor_mage() -> StoreVasCharacterInput {
    StoreVasCharacterInput {
        realm_address: 102,
        realm_name: Some("Kalimdor".into()),
        guid: Some("Player-102-1-C".into()),
        guid_order: Some(3),
        name: Some("KalimdorHero".into()),
        class_name: Some("Priest".into()),
        race_name: Some("Night Elf".into()),
        level: 70,
        class_file_name: Some("PRIEST".into()),
        race_file_name: Some("NightElf".into()),
        wow_account: 2001,
        current_server: 102,
        faction: 1,
        sex: 1,
        create_screen_icon_atlas: Some("".into()),
    }
}

fn seed_realm_and_character_inputs(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        let vas = &mut sim.store_glue.store_secure.vas;
        vas.realms = realm_inputs();
        vas.vas_realms = realm_inputs();
        vas.characters = character_inputs();
    }
    env.exec(REALM_AND_CHARACTER_INPUTS)
        .expect("explicit realm and character host rows should seed");
}

fn apply_callable_wrong_producer_overlay(env: &WowLuaEnv) {
    let Ok(path) = std::env::var("WOW_SIM_STORE_VAS_OVERLAY") else {
        return;
    };
    let overlay = std::fs::read_to_string(path).expect("wrong-producer overlay should be readable");
    env.exec(&overlay)
        .expect("wrong-producer overlay should leave the selected producer callable");
}

#[test]
fn retail_store_vas_realm_and_character_consumers_use_explicit_rows() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        apply_callable_wrong_producer_overlay(&env);
        env.exec(REALM_AND_CHARACTER_CONSUMER)
            .expect("the selected VAS panel should consume realm and character rows");
    }
}
