#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness;
use wow_ui_sim::c_api::store_catalog_model::{StoreProductInput, StoreProductSharedData};
use wow_ui_sim::c_api::store_vas_model::{
    StoreVasBoostInput, StoreVasCharacterInput, StoreVasCompletionInput, StoreVasEligibleRaceInput,
    StoreVasGuildFollowInfoInput, StoreVasGuildMasterInfoInput, StoreVasGuildMemberInput,
    StoreVasPriceInput, StoreVasPurchaseInput, StoreVasRaceEligibilityInput, StoreVasRealmInput,
    StoreVasRequest,
};
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

const LOOKUP_API_CONSUMER: &str = r#"
    local realms = C_StoreSecure.GetVASRealmList()
    assert(#realms == 3 and realms[1].realmName == "Azeroth" and realms[2].realmName == "Empty" and realms[3].realmName == "Kalimdor", "VAS realms should sort by name")
    assert(realms[1].characterCount == 0 and realms[1].pvp == false, "VAS realm rows should zero the count and preserve optional flags")
    local characters = C_StoreSecure.GetCharactersForRealm(101, true)
    assert(#characters == 1 and characters[1].name == "Simhero", "guild-only character query should match the supplied GUID set")
    local character = C_StoreSecure.GetCharacterInfoByGUID("Player-101-1-11")
    assert(character and character.name == "Simhero" and character.className == "Warrior" and character.guid == "Player-101-1-11", "character-object lookup should publish the verified row fields")
    assert(C_StoreSecure.GetCharacterInfoByGUID("Player-999-1-FF") == nil, "absent character objects should publish nil")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("ignored-local-name", true) == "WowAccount-1001-1", "local lookup should use the current account GUID")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("WoW2", false) == "WowAccount-2002-2", "remote lookup should match exact account-name bytes")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("wow2", false) == nil, "remote lookup should not fold account-name case")
    local races = C_StoreSecure.GetEligibleRacesForVASService(character.guid, 0)
    assert(#races == 2 and races[2].isAlliedRace and races[2].isHeritageArmorUnlocked, "race query should copy the supplied result rows")
    assert(C_StoreSecure.GetEligibleRacesForVASService("Player-999-1-FF", 0) == nil, "missing race data should publish nil")
    local guild = C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID(character.guid)
    assert(guild.guildName == "Simulator Guild" and guild.guildMemberInfos[1].memberName == "Simalt", "guild query should copy the supplied members")
    assert(C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID("Player-999-1-FF") == nil, "missing guild-master data should publish nil")
    assert(C_StoreSecure.GetVasServiceType(2003) == Enum.VasServiceType.NameChange, "product type should map to the verified VAS service")
    assert(C_StoreSecure.GetVasServiceType(99999) == nil, "unmapped product should publish nil")
    return true
"#;

const ACCOUNT_VALIDATION_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local observer = CreateFrame("Frame")
    observer:RegisterEvent("VAS_TRANSFER_VALIDATION_UPDATE")
    observer:SetScript("OnEvent", function(_, _, ...)
        local errorCode = ...
        local remoteGuid = C_StoreSecure.GetWoWAccountGUIDFromName("WoW2", false)
        table.insert(STORE_VAS_VALIDATION_EVENTS, {
            count = select('#', ...),
            errorType = type(errorCode),
            errorCode = errorCode,
            remoteGuid = remoteGuid,
        })
    end)
    STORE_VAS_VALIDATION_EVENTS = {}
    local selector = {}
    STORE_VAS_ACCOUNT_SELECTOR = selector
    lookup("AccountSelectorMixin").ValidateBnetTransfer(selector, " Transfer@Example.COM ")
    assert(selector.awaitingBnetTransferResponse == true, "original account selector should await validation")
    assert(#STORE_VAS_VALIDATION_EVENTS == 0, "validation request must not fire a response event")
"#;

const GUILD_MASTER_REQUEST_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local observer = CreateFrame("Frame")
    observer:RegisterEvent("STORE_GUILD_MASTER_INFO_RECEIVED")
    observer:SetScript("OnEvent", function(_, _, realmAddress)
        local info = C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID("Player-101-1-11")
        STORE_VAS_GUILD_MASTER_EVENT = { realmAddress = realmAddress, guildName = info and info.guildName }
    end)
    local panel = lookup("StoreVASValidationFrame")
    panel.productInfo = { sharedData = { vasServiceType = lookup("Enum").VasServiceType.GuildNameChange, name = "Fixture", instructions = "" } }
    local init = lookup("StoreVASValidationFrame_Init")
    assert(debug.getinfo(init, "S").source:find("Blizzard_Shared_StoreUISecure.lua", 1, true))
    init(panel)
"#;

const GUILD_FOLLOW_REQUEST_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local observer = CreateFrame("Frame")
    observer:RegisterEvent("STORE_GUILD_FOLLOW_INFO_RECEIVED")
    observer:SetScript("OnEvent", function(_, _, guid, info)
        STORE_VAS_GUILD_FOLLOW_EVENT = {
            guid = guid,
            transferredRealm = info.transferredRealm,
            factionChanged = info.factionChanged,
            count = select('#', guid, info),
        }
    end)
    local panel = lookup("StoreVASValidationFrame")
    panel.productInfo = { sharedData = { vasServiceType = lookup("Enum").VasServiceType.CharacterTransfer, name = "Fixture", instructions = "" } }
    lookup("StoreVASValidationFrame_Init")(panel)
    local realmCallback = lookup("VASCharacterSelectionRealmSelector_Callback")
    local characterCallback = lookup("VASCharacterSelectionCharacterSelector_Callback")
    assert(debug.getinfo(characterCallback, "S").source:find("Blizzard_Shared_StoreUISecure.lua", 1, true))
    realmCallback(C_StoreSecure.GetRealmList()[1])
    characterCallback(1)
"#;

const PURCHASE_COMPLETION_CONSUMER: &str = r#"
    local confirmation = { C_StoreSecure.GetConfirmationInfo() }
    assert(#confirmation == 6 and confirmation[1] == 2003 and confirmation[2] == "Balance")
    assert(confirmation[3] == 10 and confirmation[4] == 0 and confirmation[5] == 5 and confirmation[6] == 1)
    local boostProduct, boostCharacter, boostRealm = C_StoreSecure.GetUnrevokedBoostInfo()
    assert(boostProduct == "Apprentice Rider Bundle" and boostCharacter == "Simhero" and boostRealm == "Azeroth")
    C_StoreSecure.OpenNydusLink(1003)
    C_StoreSecure.PurchaseProduct(2003)
    C_StoreSecure.PurchaseVASProduct(2003, "Player-101-1-11", "Renamedhero", nil, nil, nil, nil, nil, false, false)
    C_StoreSecure.PurchaseProductConfirm(true, 10, 50)
    C_StoreSecure.OpenNydusLink(99999)
    C_StoreSecure.PurchaseVASProduct(99999, "Player-101-1-11", "Unused", nil, nil, nil, nil, nil, false, false)
    local product, guid, realm, shouldHandle = C_StoreSecure.GetVASCompletionInfo()
    assert(product == 2003 and guid == "Player-101-1-11" and realm == "Azeroth" and shouldHandle == true)
    local failureCode, failureReason = C_StoreSecure.GetFailureInfo()
    assert(failureCode == 9 and failureReason == nil)
    C_StoreSecure.AckFailure()
    assert(select(1, C_StoreSecure.GetFailureInfo()) == nil)
    return true
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
        realm_input(102, "Kalimdor", 1, true, false),
        realm_input(101, "Azeroth", 2, false, false),
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
        guid: Some("Player-101-1-11".into()),
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
        guid: Some("Player-101-1-12".into()),
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
        guid: Some("Player-102-1-13".into()),
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
        let characters = character_inputs();
        vas.characters = characters.clone();
        vas.character_objects = characters;
        vas.guild_character_guids.insert("Player-101-1-11".into());
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

fn seed_group3_lookup_inputs(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let vas = &mut sim.store_glue.store_secure.vas;
    vas.local_wow_account_guid = Some("WowAccount-1001-1".into());
    vas.wow_account_guids
        .insert(b"WoW2".to_vec(), "WowAccount-2002-2".into());
    vas.race_eligibility = vec![race_eligibility()];
    vas.guild_master_info = vec![guild_master_info()];
}

fn seed_purchase_inputs(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let vas = &mut sim.store_glue.store_secure.vas;
    vas.selected_purchase_product_id = Some(2003);
    vas.purchase_records = vec![purchase_record()];
    vas.product_prices.insert(
        2003,
        StoreVasPriceInput {
            original_amount: 100000,
            current_amount: 50100,
        },
    );
    vas.boost = Some(StoreVasBoostInput {
        product_id: 2003,
        character_guid: "Player-101-1-11".into(),
        realm_address: 101,
    });
    vas.completion = Some(StoreVasCompletionInput {
        product_id: 2003,
        guid: "Player-101-1-11".into(),
        first_auxiliary: 0,
        second_auxiliary: 0,
    });
    vas.failure_marker = 1;
    vas.received_failure_code = 1;
    vas.pre_generated_external_transaction_id = Some(b"external-1".to_vec());
}

fn seed_transfer_product(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .catalog
        .products
        .push(transfer_product());
}

fn transfer_product() -> StoreProductInput {
    StoreProductInput {
        product_id: 2010,
        supported: true,
        session_eligible: true,
        display_eligible: true,
        dynamic_bundle: false,
        product_type: Some(15),
        shared_data: StoreProductSharedData::default(),
    }
}

fn purchase_record() -> StoreVasPurchaseInput {
    StoreVasPurchaseInput {
        product_id: 2003,
        status: 9,
        secondary_status: 0,
        wallet_name: "Balance".into(),
        original_amount: 0,
        current_amount: 0,
    }
}

fn seed_transfer_completion(env: &WowLuaEnv, second_auxiliary: u32) {
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .vas
        .completion = Some(StoreVasCompletionInput {
        product_id: 2010,
        guid: "Player-999-1-FF".into(),
        first_auxiliary: 0,
        second_auxiliary,
    });
}

fn assert_transfer_completion(env: &WowLuaEnv, should_handle: bool) {
    let result: (i32, String, String, bool) = env
        .eval("return C_StoreSecure.GetVASCompletionInfo()")
        .expect("transfer completion should publish its populated tuple");
    assert_eq!(
        result,
        (2010, "Player-999-1-FF".into(), String::new(), should_handle)
    );
}

fn seed_missing_character_completion(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .vas
        .completion = Some(StoreVasCompletionInput {
        product_id: 2003,
        guid: "Player-999-1-FF".into(),
        first_auxiliary: 0,
        second_auxiliary: 0,
    });
    env.exec("C_StoreSecure.GetVASCompletionInfo()")
        .expect("missing character should prepare the follow-up lookup");
}

fn race_eligibility() -> StoreVasRaceEligibilityInput {
    StoreVasRaceEligibilityInput {
        guid: "Player-101-1-11".into(),
        service_type: 0,
        results: Some(vec![
            StoreVasEligibleRaceInput {
                race_name: "Human".into(),
                is_allied_race: false,
                is_heritage_armor_unlocked: true,
            },
            StoreVasEligibleRaceInput {
                race_name: "Void Elf".into(),
                is_allied_race: true,
                is_heritage_armor_unlocked: true,
            },
        ]),
    }
}

fn guild_master_info() -> StoreVasGuildMasterInfoInput {
    StoreVasGuildMasterInfoInput {
        guid: "Player-101-1-11".into(),
        guild_name: "Simulator Guild".into(),
        members: vec![StoreVasGuildMemberInput {
            guid: Some("Player-101-1-12".into()),
            member_name: "Simalt".into(),
        }],
    }
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

#[test]
fn retail_store_vas_lookup_apis_publish_explicit_host_rows() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        seed_group3_lookup_inputs(&env);
        let result: bool = env.eval(LOOKUP_API_CONSUMER)
            .expect("Retail VAS lookup results should reach their original call surface");
        assert!(result);
        let order: Vec<String> = env.state().borrow().store_glue.store_secure.vas.vas_realms
            .iter().map(|realm| realm.realm_name.clone()).collect();
        assert_eq!(order, ["Azeroth", "Empty", "Kalimdor"]);
    }
}

#[test]
fn retail_store_bnet_validation_uses_the_original_account_selector() {
    test_timeout! {
        let env = load_store_ui();
        env.state().borrow_mut().store_glue.connection_present = true;
        env.exec(ACCOUNT_VALIDATION_CONSUMER)
            .expect("original account selector should submit a validation request");
        assert_bnet_validation_request(&env);
        receive_bnet_success(&env);
        assert_stale_bnet_response_ignored(&env);
    }
}

fn assert_bnet_validation_request(env: &WowLuaEnv) {
    let sim = env.state();
    let store = sim.borrow();
    assert_eq!(store.store_glue.request_sequence, 1);
    assert_eq!(
        store.store_glue.store_secure.vas.submitted_requests,
        [StoreVasRequest::BnetTransferValidation {
            sequence: 1,
            account_name: b" Transfer@Example.COM ".to_vec(),
        }]
    );
}

fn receive_bnet_success(env: &WowLuaEnv) {
    wow_ui_sim::c_api::store_vas_model::receive_bnet_transfer_validation(
        env,
        1,
        0,
        "BNetAccount-55-1".into(),
        vec![
            (b"WoW2".to_vec(), "WowAccount-2002-2".into()),
            (b"WoW3".to_vec(), "WowAccount-3003-3".into()),
        ],
    )
    .expect("correlated BNet response should dispatch");
    let result: (i32, String, String, bool) = env.eval("return STORE_VAS_VALIDATION_EVENTS[1].count, STORE_VAS_VALIDATION_EVENTS[1].errorType, STORE_VAS_VALIDATION_EVENTS[1].remoteGuid, C_StoreSecure.GetWoWAccountGUIDFromName('WoW3', false) ~= nil")
        .expect("successful validation event should observe replaced account state");
    assert_eq!(result, (1, "nil".into(), "WowAccount-2002-2".into(), true));
}

fn assert_stale_bnet_response_ignored(env: &WowLuaEnv) {
    wow_ui_sim::c_api::store_vas_model::receive_bnet_transfer_validation(
        env,
        0,
        7,
        "ignored".into(),
        vec![(b"WoW4".to_vec(), "WowAccount-4004-4".into())],
    )
    .expect("stale BNet response should be ignored");
    let count: i32 = env
        .eval("return #STORE_VAS_VALIDATION_EVENTS")
        .expect("stale response should not dispatch another event");
    assert_eq!(count, 1);
}

#[test]
fn retail_store_vas_guild_master_request_uses_the_original_panel() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        env.exec(GUILD_MASTER_REQUEST_CONSUMER)
            .expect("original VAS panel should request guild-master data");
        assert_eq!(env.state().borrow().store_glue.store_secure.vas.submitted_requests.len(), 1);
        let request = env.state().borrow().store_glue.store_secure.vas.submitted_requests[0].clone();
        let StoreVasRequest::RealmGuildMasterInfo { realm_address, character_guids } = request else {
            panic!("original VAS panel should submit a realm guild-master request");
        };
        assert_eq!(realm_address, 101);
        assert_eq!(character_guids.len(), 2);
        wow_ui_sim::c_api::store_vas_model::receive_guild_master_info(
            &env,
            101,
            vec![guild_master_info()],
        )
        .expect("guild-master response should update state before notification");
        let event: (i32, String) = env.eval("return STORE_VAS_GUILD_MASTER_EVENT.realmAddress, STORE_VAS_GUILD_MASTER_EVENT.guildName")
            .expect("guild-master event should observe the updated cache");
        assert_eq!(event, (101, "Simulator Guild".into()));
    }
}

#[test]
fn retail_store_vas_guild_follow_request_uses_the_original_character_callback() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        env.exec(GUILD_FOLLOW_REQUEST_CONSUMER)
            .expect("original character callback should request guild-follow data");
        assert_eq!(
            env.state()
                .borrow()
                .store_glue
                .store_secure
                .vas
                .submitted_requests
                .len(),
            1,
            "original callback should submit one guild-follow request"
        );
        let request = env.state().borrow().store_glue.store_secure.vas.submitted_requests[0].clone();
        let StoreVasRequest::CharacterGuildFollowInfo { guid, realm_address } = request else {
            panic!("original character callback should submit a guild-follow request");
        };
        assert!(matches!(guid, wow_ui_sim::c_api::store_guid::Outcome::Known(_)));
        assert_eq!(realm_address, 101);
        wow_ui_sim::c_api::store_vas_model::receive_guild_follow_info(
            &env,
            StoreVasGuildFollowInfoInput {
                guid: "Player-101-1-11".into(),
                transferred_realm: Some("Kalimdor".into()),
                faction_changed: false,
            },
        )
        .expect("guild-follow response should update state before notification");
        let event: (String, String, bool, i32) = env.eval("return STORE_VAS_GUILD_FOLLOW_EVENT.guid, STORE_VAS_GUILD_FOLLOW_EVENT.transferredRealm, STORE_VAS_GUILD_FOLLOW_EVENT.factionChanged, STORE_VAS_GUILD_FOLLOW_EVENT.count")
            .expect("guild-follow event should publish its verified payload");
        assert_eq!(event, ("Player-101-1-11".into(), "Kalimdor".into(), false, 2));
    }
}

#[test]
fn retail_store_vas_transfer_completion_and_missing_character_request_are_modeled() {
    test_timeout! {
        let env = load_store_ui();
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        seed_transfer_product(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        seed_transfer_completion(&env, 0);
        assert_transfer_completion(&env, true);
        seed_transfer_completion(&env, 1);
        assert_transfer_completion(&env, false);
        seed_missing_character_completion(&env);
        assert!(env.state().borrow().store_glue.store_secure.vas.completion_request_pending);
        let requests = env
            .state()
            .borrow()
            .store_glue
            .store_secure
            .vas
            .submitted_requests
            .clone();
        assert_eq!(requests.len(), 1);
        assert!(matches!(
            &requests[0],
            StoreVasRequest::CompletionInfo { product_id: 2003, .. }
        ));
    }
}

#[test]
fn retail_store_purchase_confirmation_checks_selected_state_and_cents() {
    test_timeout! {
        let env = load_store_ui();
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        seed_purchase_inputs(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        env.exec("C_StoreSecure.PurchaseProductConfirm(true, 10, 100)")
            .expect("100 narrowed cents should reject confirmation preparation");
        assert!(env.state().borrow().store_glue.store_secure.vas.submitted_requests.is_empty());
        env.exec("C_StoreSecure.PurchaseProductConfirm(false)")
            .expect("eligible cancellation should prepare a zero amount");
        assert_eq!(
            env.state().borrow().store_glue.store_secure.vas.submitted_requests,
            [StoreVasRequest::PurchaseConfirmation {
                correlation: 0,
                confirmed: false,
                dollars: 0,
                cents: 0,
            }]
        );
        env.state().borrow_mut().store_glue.store_secure.vas.purchase_records[0].status = 8;
        env.exec("C_StoreSecure.PurchaseProductConfirm(false)")
            .expect("unsuitable purchase state should not prepare a request");
        assert_eq!(env.state().borrow().store_glue.store_secure.vas.submitted_requests.len(), 1);
    }
}

#[test]
fn retail_store_vas_purchase_and_completion_use_explicit_host_inputs() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        seed_group3_lookup_inputs(&env);
        seed_purchase_inputs(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        let result: bool = env.eval(PURCHASE_COMPLETION_CONSUMER)
            .expect("explicit VAS purchase and completion inputs should be observable");
        assert!(result);
        assert_purchase_requests(&env);
        assert_eq!(
            env.state().borrow().store_glue.store_secure.vas.opened_nydus_links,
            ["fixture-link".to_string()]
        );
        clear_external_transaction_id(&env);
    }
}

fn assert_purchase_requests(env: &WowLuaEnv) {
    let state = env.state();
    let sim = state.borrow();
    let requests = &sim.store_glue.store_secure.vas.submitted_requests;
    assert_eq!(requests.len(), 4);
    assert_eq!(
        requests[0],
        StoreVasRequest::ProductPurchase {
            sequence: 1,
            product_id: 2003,
            external_transaction_id: Some(b"external-1".to_vec()),
        }
    );
    assert_purchase_confirmation(&requests[1]);
    assert_eq!(
        requests[2],
        StoreVasRequest::PurchaseConfirmation {
            correlation: 2,
            confirmed: true,
            dollars: 10,
            cents: 50,
        }
    );
    assert_eq!(
        requests[3],
        StoreVasRequest::AcknowledgeFailure { marker: 1 }
    );
}

fn assert_purchase_confirmation(request: &StoreVasRequest) {
    let StoreVasRequest::VasPurchase {
        sequence,
        product_id,
        new_name,
        guild_follow,
        ..
    } = request
    else {
        panic!("VAS purchase should prepare its service request");
    };
    assert_eq!(
        (*sequence, *product_id, new_name.as_deref()),
        (2, 2003, Some(b"Renamedhero".as_slice()))
    );
    assert!(!guild_follow);
}

fn clear_external_transaction_id(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .vas
        .pre_generated_external_transaction_id = Some(b"clear-me".to_vec());
    let results: i32 = env
        .eval("return select('#', C_StoreSecure.ClearPreGeneratedExternalTransactionID())")
        .expect("transaction clear should return zero values");
    assert_eq!(results, 0);
    assert!(
        env.state()
            .borrow()
            .store_glue
            .store_secure
            .vas
            .pre_generated_external_transaction_id
            .is_none()
    );
}
