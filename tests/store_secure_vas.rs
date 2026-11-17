#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness;
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
    assert(selectors.CharacterSelector.Text:GetText():find("KalimdorMage", 1, true))
    VASCharacterSelectionCharacterSelector_Callback(1)
    assert(selectors.SelectedCharacterName:GetText() == "KalimdorMage")

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
        &["Blizzard_StoreUI"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_StoreUI"));
    assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
    env
}

const REALM_AND_CHARACTER_INPUTS: &str = r#"
    TEST_VAS_REALMS = {
        { virtualRealmAddress = 101, realmName = "Azeroth", characterCount = 2, pvp = false, rp = false },
        { virtualRealmAddress = 202, realmName = "Kalimdor", characterCount = 1, pvp = true, rp = false },
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
        [202] = {
            { guid = 602001, name = "KalimdorMage", realmName = "Kalimdor", currentServer = 202,
              classFileName = "MAGE", className = "Mage", level = 80,
              raceName = "Night Elf", faction = 1, wowAccount = 2002, createScreenIconAtlas = "" },
        },
    }
"#;

fn seed_realm_and_character_inputs(env: &WowLuaEnv) {
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
