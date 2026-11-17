#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness;
use wow_ui_sim::c_api::store_vas_model::{StoreVasCharacterInput, StoreVasRealmInput};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const REALM_AND_CHARACTER_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local init = lookup("StoreVASValidationFrame_Init")
    local panel = lookup("StoreVASValidationFrame")
    local openRealm = lookup("VASCharacterSelectionRealmSelector_OnClick")
    local openCharacter = lookup("VASCharacterSelectionCharacterSelector_OnClick")
    local chooseEntry = lookup("StoreDropdownMenuMenuButton_OnClick")
    local menuButtonOnLoad = lookup("StoreDropdownMenuMenuButton_OnLoad")
    local createFrame = lookup("CreateFrame")
    local source = debug.getinfo(init, "S")
    assert(source.source:find("Blizzard_Shared_StoreUISecure.lua", 1, true), "Retail StoreUI consumer must be selected")

    panel.productInfo = {
        sharedData = {
            vasServiceType = lookup("Enum").VasServiceType.NameChange,
            name = "Fixture service",
            instructions = "",
        },
    }
    init(panel)

    local selectors = panel.CharacterSelectionFrame
    -- Test-only menu slots avoid the intentionally unavailable forbidden-frame factory.
    local function seedMenuButtons(menu, count)
        for index = 2, count do
            if not menu.Buttons[index] then
                local button = createFrame("Button", nil, menu, "StoreDropdownMenuButtonTemplate")
                menu.Buttons[index] = button
                menuButtonOnLoad(button)
            end
        end
    end
    seedMenuButtons(selectors.RealmSelector.List, 3)
    seedMenuButtons(selectors.CharacterSelector.List, 2)

    openRealm(selectors.RealmSelector.Button)
    local realmMenu = selectors.RealmSelector.List
    assert(realmMenu:IsShown(), "realm selector should show its original dropdown")
    assert(realmMenu.Buttons[1]:GetText():find("Azeroth", 1, true), "first realm row should reach dropdown")
    assert(realmMenu.Buttons[2]:GetText():find("Kalimdor", 1, true), "second realm row should reach dropdown")
    assert(realmMenu.Buttons[3]:GetText():find("Empty", 1, true), "empty-roster realm row should reach dropdown")

    chooseEntry(realmMenu.Buttons[1], "LeftButton")
    openCharacter(selectors.CharacterSelector.Button)
    local characterMenu = selectors.CharacterSelector.List
    assert(characterMenu:IsShown(), "character selector should show its original dropdown")
    assert(characterMenu.Buttons[1] and characterMenu.Buttons[1]:IsShown()
        and characterMenu.Buttons[1]:GetText():find("Simhero", 1, true), "first character row should reach dropdown")
    assert(characterMenu.Buttons[2] and characterMenu.Buttons[2]:IsShown()
        and characterMenu.Buttons[2]:GetText():find("Simalt", 1, true), "second character row should reach dropdown")

    openRealm(selectors.RealmSelector.Button)
    chooseEntry(realmMenu.Buttons[2], "LeftButton")
    openCharacter(selectors.CharacterSelector.Button)
    assert(characterMenu.Buttons[1] and characterMenu.Buttons[1]:IsShown()
        and characterMenu.Buttons[1]:GetText():find("KalimdorHero", 1, true), "second realm row should reach dropdown")

    openRealm(selectors.RealmSelector.Button)
    chooseEntry(realmMenu.Buttons[3], "LeftButton")
    assert(selectors.RealmSelector.Text:GetText() == "Empty", "empty realm should be selected through its dropdown")
    assert(selectors.NoEligibleCharactersErrorMessage:IsShown(), "absent roster should show the original empty state")
    openCharacter(selectors.CharacterSelector.Button)
    assert(characterMenu:IsShown(), "empty roster should keep the original dropdown boundary")
    assert(not characterMenu.Buttons[1] or not characterMenu.Buttons[1]:IsShown(), "empty roster should have no selectable dropdown row")
"#;

const EMPTY_REALM_LIST_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local selectors = lookup("StoreVASValidationFrame").CharacterSelectionFrame
    lookup("VASCharacterSelectionRealmSelector_OnClick")(selectors.RealmSelector.Button)
    assert(selectors.RealmSelector.List:IsShown(), "empty realm list should use the original dropdown")
    assert(not selectors.RealmSelector.List.Buttons[1] or not selectors.RealmSelector.List.Buttons[1]:IsShown(), "empty realm list should have no dropdown rows")
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
    -- Empty class color strings let the original text formatter run; colors are not asserted.
    local classColors = rawget(_G, "RAID_CLASS_COLORS")
    if type(classColors) ~= "table" then classColors = {} end
    for _, classFile in ipairs({ "WARRIOR", "MAGE", "PRIEST" }) do
        if classColors[classFile] == nil then classColors[classFile] = { colorStr = "" } end
    end
    rawset(_G, "RAID_CLASS_COLORS", classColors)
    if type(__secureenv) == "table" and type(rawget(__secureenv, "RAID_CLASS_COLORS")) ~= "table" then
        rawset(__secureenv, "RAID_CLASS_COLORS", classColors)
    end
    __wow_store_state.realms = TEST_VAS_REALMS
    __wow_store_state.characters = {
        [101] = {
            { guid = 501001, name = "Simhero", realmName = "Azeroth", currentServer = 101,
              classFileName = "WARRIOR", className = "Warrior", level = 70,
              raceName = "Human", raceFileName = "Human", faction = 0, sex = 1,
              wowAccount = 1001, createScreenIconAtlas = "", guildMaster = true },
            { guid = 501002, name = "Simalt", realmName = "Azeroth", currentServer = 101,
              classFileName = "MAGE", className = "Mage", level = 60,
              raceName = "Dwarf", raceFileName = "Dwarf", faction = 0, sex = 1,
              wowAccount = 1001, createScreenIconAtlas = "", guildMaster = false },
        },
        [102] = {
            { guid = 502001, name = "KalimdorHero", realmName = "Kalimdor", currentServer = 102,
              classFileName = "PRIEST", className = "Priest", level = 70,
              raceName = "Night Elf", raceFileName = "NightElf", faction = 1, sex = 1,
              wowAccount = 2001, createScreenIconAtlas = "", guildMaster = false },
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

fn clear_realm_inputs(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .vas
        .realms
        .clear();
    env.exec("__wow_store_state.realms = {}")
        .expect("empty realm host input should seed");
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
        clear_realm_inputs(&env);
        env.exec(EMPTY_REALM_LIST_CONSUMER)
            .expect("the selected realm dropdown should consume an empty realm list");
    }
}
