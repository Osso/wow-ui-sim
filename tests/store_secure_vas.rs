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

const NAME_CHANGE_SERVICE_TYPE: u8 = 0;
const CHARACTER_TRANSFER_PRODUCT_TYPE: u32 = 15;
const FIRST_BNET_TRANSFER_GUID: &str = "BNetAccount-0-0000002A";
const SECOND_BNET_TRANSFER_GUID: &str = "BNetAccount-7-000000010000002A";
const FIRST_BNET_TRANSFER_ACCOUNTS: &[(&[u8], &str)] = &[
    (b"WoW2", "WowAccount-2002-2"),
    (b"WoW3", "WowAccount-3003-3"),
];
const SECOND_BNET_TRANSFER_ACCOUNTS: &[(&[u8], &str)] = &[(b"WoW3", "WowAccount-3003-3")];
const TEST_SPARSE_CHARACTER_REALM_ADDRESS: u32 = 404;
const TEST_SPARSE_CHARACTER_GUID: &str = "Player-404-1-1";

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
    local sparseRealmAddress = 404
    local sparseGuid = "Player-404-1-1"
    local nameChangeServiceType = Enum.VasServiceType.NameChange

    local realmList = C_StoreSecure.GetRealmList()
    assert(#realmList == 3, "realm list should include supplied rows")
    assert(realmList[1].realmName == "Azeroth", "largest realm should sort first")
    assert(realmList[2].realmName == "Kalimdor", "second-largest realm should sort second")
    assert(realmList[3].realmName == "Empty", "empty realm should sort last")
    assert(realmList[1].virtualRealmAddress == 101 and realmList[1].characterCount == 2, "first realm should copy its address and count")
    assert(realmList[1].pvp == false and realmList[1].rp == false, "first realm should copy its flags")
    assert(realmList[2].virtualRealmAddress == 102 and realmList[2].characterCount == 1, "second realm should copy its address and count")
    assert(realmList[2].pvp == true and realmList[2].rp == false, "second realm should copy its flags")
    assert(realmList[3].virtualRealmAddress == 303 and realmList[3].characterCount == 0, "empty realm should copy its address and count")
    assert(realmList[3].pvp == nil and realmList[3].rp == nil, "missing optional realm flags should stay nil")

    local vasRealms = C_StoreSecure.GetVASRealmList()
    assert(#vasRealms == 3, "VAS realm list should include supplied rows")
    assert(vasRealms[1].realmName == "Azeroth", "VAS realms should sort by name")
    assert(vasRealms[2].realmName == "Empty", "second VAS realm should sort by name")
    assert(vasRealms[3].realmName == "Kalimdor", "third VAS realm should sort by name")
    assert(vasRealms[1].characterCount == 0 and vasRealms[1].pvp == false and vasRealms[1].rp == false, "first VAS row should zero count and copy flags")
    assert(vasRealms[2].virtualRealmAddress == 303 and vasRealms[2].characterCount == 0, "empty VAS realm should copy its address and zero count")
    assert(vasRealms[2].pvp == nil and vasRealms[2].rp == nil, "empty VAS realm should preserve absent flags")
    assert(vasRealms[3].virtualRealmAddress == 102 and vasRealms[3].characterCount == 0, "remaining VAS realm should publish zero characters")

    local characters = C_StoreSecure.GetCharactersForRealm(101, false)
    assert(#characters == 2, "character query should return both supplied rows")
    assert(characters[1].name == "Simhero" and characters[2].name == "Simalt", "equal-level characters should sort by GUID order")
    local first = characters[1]
    assert(first.className == "Warrior" and first.raceName == "Human", "character list row should copy names")
    assert(first.level == 70 and first.classFileName == "WARRIOR" and first.raceFileName == "Human", "character list row should copy class and level")
    assert(first.guid == "Player-101-1-11" and first.wowAccount == 1001 and first.currentServer == 101, "character list row should copy identifiers")
    assert(first.faction == 0 and first.sex == 1 and first.createScreenIconAtlas == "", "character list row should copy numeric and atlas fields")

    local character = C_StoreSecure.GetCharacterInfoByGUID(first.guid)
    assert(character ~= nil, "character object should be present")
    assert(character.name == "Simhero" and character.className == "Warrior" and character.raceName == "Human", "character lookup should copy names")
    assert(character.level == 70 and character.classFileName == "WARRIOR" and character.raceFileName == "Human", "character lookup should copy class and level")
    assert(character.guid == "Player-101-1-11" and character.wowAccount == 1001 and character.currentServer == 101, "character lookup should copy identifiers")
    assert(character.faction == 0 and character.sex == 1 and character.createScreenIconAtlas == "", "character lookup should copy numeric and atlas fields")
    local guildCharacters = C_StoreSecure.GetCharactersForRealm(101, true)
    assert(#guildCharacters == 1 and guildCharacters[1].guid == character.guid, "guild-only query should apply the supplied GUID set")
    assert(#C_StoreSecure.GetCharactersForRealm(303, false) == 0, "empty realm should return an empty character array")
    assert(#C_StoreSecure.GetCharactersForRealm(999, false) == 0, "unmatched realm should return an empty character array")

    local sparseCharacters = C_StoreSecure.GetCharactersForRealm(sparseRealmAddress, false)
    assert(#sparseCharacters == 2, "sparse realm query should return both supplied rows")
    local sparseFound = false
    local emptyGuidFound = false
    for _, row in ipairs(sparseCharacters) do
        if row.name == "Sparse" then sparseFound = row.guid == sparseGuid end
        if row.name == "EmptyGuid" then emptyGuidFound = row.guid == nil end
    end
    assert(sparseFound and emptyGuidFound, "empty character GUID should publish nil")
    local sparse = C_StoreSecure.GetCharacterInfoByGUID(sparseGuid)
    assert(sparse and sparse.name == "Sparse", "sparse character object should be present")
    assert(sparse.className == nil and sparse.raceName == nil, "missing optional character names should stay nil")
    assert(sparse.classFileName == nil and sparse.raceFileName == nil, "missing optional character files should stay nil")
    assert(sparse.createScreenIconAtlas == nil, "missing optional character atlas should stay nil")
    assert(C_StoreSecure.GetCharacterInfoByGUID("Player-999-1-FF") == nil, "absent character objects should publish nil")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("ignored-local-name", true) == "WowAccount-1001-1", "local lookup should use the current account GUID")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("WoW2", false) == "WowAccount-2002-2", "remote lookup should match exact account-name bytes")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("wow2", false) == nil, "remote lookup should not fold account-name case")
    assert(C_StoreSecure.GetWoWAccountGUIDFromName("Empty", false) == nil, "empty remote account GUID should publish nil")

    local races = C_StoreSecure.GetEligibleRacesForVASService(character.guid, nameChangeServiceType)
    assert(#races == 2, "race query should return supplied rows")
    assert(races[1].raceName == "Human" and not races[1].isAlliedRace, "first race should copy its name and status")
    assert(races[2].raceName == "Void Elf" and races[2].isAlliedRace and races[2].isHeritageArmorUnlocked, "second race should copy its fields")
    assert(#C_StoreSecure.GetEligibleRacesForVASService(sparse.guid, nameChangeServiceType) == 0, "supplied empty race data should return an empty array")
    assert(C_StoreSecure.GetEligibleRacesForVASService("Player-999-1-FF", nameChangeServiceType) == nil, "missing race data should publish nil")

    local guild = C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID(character.guid)
    assert(guild.guildName == "Simulator Guild", "guild query should copy its name")
    assert(guild.guildMemberInfos[1].memberName == "Simalt" and guild.guildMemberInfos[1].guid == "Player-101-1-12", "guild query should copy member fields")
    local emptyGuild = C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID(sparse.guid)
    assert(emptyGuild and emptyGuild.guildName == "Empty Guild", "present empty guild data should keep its name")
    assert(#emptyGuild.guildMemberInfos == 0, "present empty guild data should return an empty member array")
    assert(C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID("Player-999-1-FF") == nil, "missing guild-master data should publish nil")
    assert(C_StoreSecure.GetVasServiceType(2003) == Enum.VasServiceType.NameChange, "product type should map to the verified VAS service")
    assert(C_StoreSecure.GetVasServiceType(99999) == nil, "missing product should publish nil")
    return true
"#;

const VAS_PRODUCT_TYPE_MAPPING_CONSUMER: &str = r#"
    local rows = {
        {2100, 0}, {2101, 1}, {2102, 2}, {2103, 3}, {2104, 4}, {2105, 5},
        {2106, 6}, {2107, 7}, {2108, 8}, {2109, 9}, {2110, 4},
    }
    for _, row in ipairs(rows) do
        assert(C_StoreSecure.GetVasServiceType(row[1]) == row[2], "product type should map to its service")
    end
    assert(C_StoreSecure.GetVasServiceType(2111) == nil, "unmapped product type should return nil")
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
    local validate = lookup("AccountSelectorMixin").ValidateBnetTransfer
    local source = debug.getinfo(validate, "S").source
    assert(source:find("CharacterServicesAccountSelector.lua", 1, true), "Retail account selector must be selected")
    validate(selector, " Transfer@Example.COM ")
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

const VAS_PURCHASE_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local panel = lookup("StoreVASValidationFrame")
    lookup("StoreFrame_SetSelectedEntryID")(1003)
    lookup("StoreVASValidationFrame_GetProductInfo")(panel)
    local continue = lookup("VASCharacterSelectionContinueButton_OnClick")
    local continueSource = debug.getinfo(continue, "S").source
    assert(continueSource:find("Blizzard_Shared_StoreUISecure.lua", 1, true), "Retail VAS purchase consumer must be selected")
    lookup("StoreVASValidationFrame_Init")(panel)
    panel:Show()
    local selectors = panel.CharacterSelectionFrame
    local realms = C_StoreSecure.GetRealmList()
    lookup("VASCharacterSelectionRealmSelector_Callback")(realms[1])
    lookup("VASCharacterSelectionCharacterSelector_Callback")(1)
    selectors.NewCharacterName:SetText("Renamedhero")
    continue(selectors.ContinueButton)
    assert(not selectors.ContinueButton:IsShown() and selectors.Spinner:IsShown(), "accepted purchase should enter the original waiting state")
"#;

const BNET_TRANSFER_INFO_CONSUMER_SETUP: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local panel = lookup("StoreVASValidationFrame")
    lookup("StoreFrame_SetSelectedEntryID")(1003)
    lookup("StoreVASValidationFrame_GetProductInfo")(panel)
    panel.productInfo.sharedData.vasServiceType = lookup("Enum").VasServiceType.CharacterTransfer
    panel.productInfo.sharedData.canChangeBNetAccount = true
    lookup("StoreVASValidationFrame_Init")(panel)
    panel:Show()
    local selectors = panel.CharacterSelectionFrame
    local store = lookup("C_StoreSecure")
    lookup("VASCharacterSelectionRealmSelector_Callback")(store.GetRealmList()[1])
    lookup("VASCharacterSelectionCharacterSelector_Callback")(1)
    selectors.TransferAccountCheckbox:SetChecked(true)
    lookup("TransferAccountCheckbox_OnClick")(selectors.TransferAccountCheckbox)
    local differentBnet = lookup("BLIZZARD_STORE_VAS_DIFFERENT_BNET")
    assert(differentBnet ~= nil, "BNet transfer selector should be available")
    lookup("VASCharacterSelectionTransferAccountDropdown_Callback")(differentBnet)
    STORE_VAS_BNET_SELECTORS = selectors
    STORE_VAS_BNET_CONTINUE = lookup("VASCharacterSelectionContinueButton_OnClick")
    STORE_VAS_BNET_TEXT_CHANGED = lookup("VASCharacterSelectionTransferBattlenetAccountEditbox_OnTextChanged")
    STORE_VAS_BNET_ACCOUNT_SELECTED = lookup("VASCharacterSelectionTransferBnetWoWAccountDropdown_Callback")
    local source = debug.getinfo(STORE_VAS_BNET_CONTINUE, "S").source
    assert(source:find("Blizzard_Shared_StoreUISecure.lua", 1, true), "Retail transfer consumer must be selected")
    STORE_VAS_BNET_TRANSFER_EVENTS = {}
    local observer = CreateFrame("Frame")
    observer:RegisterEvent("VAS_TRANSFER_VALIDATION_UPDATE")
    observer:SetScript("OnEvent", function(_, _, ...)
        local errorCode = ...
        local guid, accounts = store.GetBnetTransferInfo()
        table.insert(STORE_VAS_BNET_TRANSFER_EVENTS, {
            count = select('#', ...),
            errorType = type(errorCode),
            errorCode = errorCode,
            guid = guid,
            accountNames = table.concat(accounts, "|"),
            accountCount = #accounts,
        })
    end)
    assert(select('#', store.GetBnetTransferInfo("ignored", false)) == 2, "getter should ignore supplied arguments")
    local guid, accounts = store.GetBnetTransferInfo()
    assert(guid == nil and type(accounts) == "table" and #accounts == 0, "empty host inputs should publish nil and an empty account array")
    local function installPurchaseProbe(namespace)
        local original = namespace.PurchaseVASProduct
        namespace.PurchaseVASProduct = function(productID, characterGuid, newName, oldGuildName, newGuildMasterGuid, destinationRealm, wowAccountGuid, bnetAccountGuid, transferFactionBundle, guildFollow)
            assert(bnetAccountGuid == STORE_VAS_EXPECTED_BNET_GUID, "original transfer consumer should pass the saved BNet GUID")
            return original(productID, characterGuid, newName, oldGuildName, newGuildMasterGuid, destinationRealm, wowAccountGuid, bnetAccountGuid, transferFactionBundle, guildFollow)
        end
    end
    local secureStore = __secureenv and rawget(__secureenv, "C_StoreSecure")
    installPurchaseProbe(store)
    if secureStore and secureStore ~= store then installPurchaseProbe(secureStore) end
    STORE_VAS_BNET_START_VALIDATION = function(email)
        selectors.TransferBattlenetAccountEditbox:SetText(email)
        STORE_VAS_BNET_TEXT_CHANGED(selectors.TransferBattlenetAccountEditbox)
        STORE_VAS_BNET_CONTINUE(selectors.ContinueButton)
        assert(selectors.Spinner:IsShown(), "original transfer consumer should wait for validation")
    end
    STORE_VAS_BNET_PURCHASE = function(accountName, expectedGuid)
        STORE_VAS_EXPECTED_BNET_GUID = expectedGuid
        STORE_VAS_BNET_ACCOUNT_SELECTED(accountName)
        STORE_VAS_BNET_CONTINUE(selectors.ContinueButton)
    end
"#;

const VAS_COMPLETION_CONSUMER: &str = r#"
    local function lookup(name)
        return _G[name] or (__secureenv and rawget(__secureenv, name))
    end
    local complete = lookup("StoreVASValidationFrame_OnVasProductComplete")
    local completeSource = debug.getinfo(complete, "S").source
    assert(completeSource:find("Blizzard_Shared_StoreUISecure.lua", 1, true), "Retail VAS completion consumer must be selected")
    local parent = CreateFrame("Frame")
    parent:Show()
    complete(CreateFrame("Frame", nil, parent))
    assert(not parent:IsShown(), "resolved completion should close the original consumer parent")
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
        StoreVasRealmInput {
            virtual_realm_address: 102,
            realm_name: "Kalimdor".into(),
            character_count: 1,
            pvp: Some(true),
            rp: Some(false),
        },
        StoreVasRealmInput {
            virtual_realm_address: 101,
            realm_name: "Azeroth".into(),
            character_count: 2,
            pvp: Some(false),
            rp: Some(false),
        },
        StoreVasRealmInput {
            virtual_realm_address: 303,
            realm_name: "Empty".into(),
            character_count: 0,
            pvp: None,
            rp: None,
        },
    ]
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
        level: 70,
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

fn sparse_character() -> StoreVasCharacterInput {
    StoreVasCharacterInput {
        realm_address: TEST_SPARSE_CHARACTER_REALM_ADDRESS,
        guid: Some(TEST_SPARSE_CHARACTER_GUID.into()),
        guid_order: Some(1),
        name: Some("Sparse".into()),
        level: 1,
        ..Default::default()
    }
}

fn empty_guid_character() -> StoreVasCharacterInput {
    StoreVasCharacterInput {
        realm_address: TEST_SPARSE_CHARACTER_REALM_ADDRESS,
        guid: Some(String::new()),
        guid_order: Some(2),
        name: Some("EmptyGuid".into()),
        level: 1,
        ..Default::default()
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
    vas.wow_account_guids
        .insert(b"Empty".to_vec(), String::new());
    let sparse = sparse_character();
    let empty_guid = empty_guid_character();
    vas.characters.extend([sparse.clone(), empty_guid.clone()]);
    vas.character_objects.extend([sparse, empty_guid]);
    vas.race_eligibility = vec![race_eligibility(), empty_race_eligibility()];
    vas.guild_master_info = vec![guild_master_info(), empty_guild_master_info()];
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
        service_type: NAME_CHANGE_SERVICE_TYPE,
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

fn empty_race_eligibility() -> StoreVasRaceEligibilityInput {
    StoreVasRaceEligibilityInput {
        guid: TEST_SPARSE_CHARACTER_GUID.into(),
        service_type: NAME_CHANGE_SERVICE_TYPE,
        results: Some(Vec::new()),
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

fn empty_guild_master_info() -> StoreVasGuildMasterInfoInput {
    StoreVasGuildMasterInfoInput {
        guid: TEST_SPARSE_CHARACTER_GUID.into(),
        guild_name: "Empty Guild".into(),
        members: Vec::new(),
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
        env.state().borrow_mut().store_glue.store_secure.vas.local_wow_account_guid =
            Some(String::new());
        let empty_local_guid: bool = env
            .eval("return C_StoreSecure.GetWoWAccountGUIDFromName('ignored', true) == nil")
            .expect("empty local account GUID should publish nil");
        assert!(empty_local_guid);
        let order: Vec<String> = env.state().borrow().store_glue.store_secure.vas.vas_realms
            .iter().map(|realm| realm.realm_name.clone()).collect();
        assert_eq!(order, ["Azeroth", "Empty", "Kalimdor"]);
        env.state().borrow_mut().store_glue.store_secure.vas.vas_realms.clear();
        let empty_list: bool = env.eval("return #C_StoreSecure.GetVASRealmList() == 0")
            .expect("empty VAS realm input should publish an empty array");
        assert!(empty_list);
    }
}

#[test]
fn retail_store_vas_service_type_mapping_matches_product_types() {
    test_timeout! {
        let env = load_store_ui();
        seed_vas_service_type_products(&env);
        let result: bool = env.eval(VAS_PRODUCT_TYPE_MAPPING_CONSUMER)
            .expect("Retail VAS product types should map through C_StoreSecure");
        assert!(result);
    }
}

fn seed_vas_service_type_products(env: &WowLuaEnv) {
    let products = build_vas_service_type_products()
        .into_iter()
        .map(|(product_id, product_type)| build_vas_service_type_product(product_id, product_type))
        .collect::<Vec<_>>();
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .catalog
        .products
        .extend(products);
}

fn build_vas_service_type_products() -> [(i32, u32); 12] {
    [
        (2100, 7),
        (2101, 8),
        (2102, 9),
        (2103, 10),
        (2104, 15),
        (2105, 16),
        (2106, 19),
        (2107, 20),
        (2108, 21),
        (2109, 22),
        (2110, 23),
        (2111, 99),
    ]
}

fn build_vas_service_type_product(product_id: i32, product_type: u32) -> StoreProductInput {
    StoreProductInput {
        product_id,
        supported: true,
        session_eligible: true,
        display_eligible: true,
        dynamic_bundle: false,
        product_type: Some(product_type),
        shared_data: StoreProductSharedData::default(),
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

#[test]
fn retail_store_bnet_error_preserves_account_map_and_notifies_original_selector() {
    test_timeout! {
        let env = load_store_ui();
        seed_group3_lookup_inputs(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        env.exec(ACCOUNT_VALIDATION_CONSUMER)
            .expect("original account selector should submit the validation request");
        receive_bnet_error(&env);
    }
}

#[test]
fn retail_store_bnet_transfer_info_uses_original_transfer_consumer() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        set_character_transfer_product_type(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        apply_callable_wrong_producer_overlay(&env);
        env.exec(BNET_TRANSFER_INFO_CONSUMER_SETUP)
            .expect("original character-transfer consumer should initialize");
        assert_bnet_transfer_snapshot(&env, None, &[]);
        complete_first_bnet_transfer(&env);
        complete_second_bnet_transfer(&env);
        verify_bnet_transfer_error_stale_and_empty_snapshot(&env);
    }
}

#[test]
fn retail_store_bnet_validation_request_preserves_the_transfer_snapshot() {
    test_timeout! {
        let env = load_store_ui();
        env.state().borrow_mut().store_glue.connection_present = true;
        {
            let mut sim = env.state().borrow_mut();
            let vas = &mut sim.store_glue.store_secure.vas;
            vas.bnet_transfer_guid = Some(FIRST_BNET_TRANSFER_GUID.into());
            vas.bnet_game_accounts = vec![b"WoW2".to_vec()];
        }
        env.exec("C_StoreSecure.ValidateBnetTransfer('new@example.com')")
            .expect("new validation request should be accepted");
        assert_bnet_validation_request_at(&env, 1, "new@example.com");
        assert_bnet_transfer_snapshot(&env, Some(FIRST_BNET_TRANSFER_GUID), &["WoW2"]);
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

fn receive_bnet_error(env: &WowLuaEnv) {
    wow_ui_sim::c_api::store_vas_model::receive_bnet_transfer_validation(
        env,
        1,
        7,
        "BNetAccount-ignored".into(),
        vec![(b"WoW3".to_vec(), "WowAccount-3003-3".into())],
    )
    .expect("correlated error response should dispatch without replacing accounts");
    let result: (i32, String, i32, String, bool) = env
        .eval("return STORE_VAS_VALIDATION_EVENTS[1].count, STORE_VAS_VALIDATION_EVENTS[1].errorType, STORE_VAS_VALIDATION_EVENTS[1].errorCode, STORE_VAS_VALIDATION_EVENTS[1].remoteGuid, C_StoreSecure.GetWoWAccountGUIDFromName('WoW3', false) ~= nil")
        .expect("error event should retain its number and current account map");
    assert_eq!(
        result,
        (1, "number".into(), 7, "WowAccount-2002-2".into(), false)
    );
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

fn set_character_transfer_product_type(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let product = sim
        .store_glue
        .store_secure
        .catalog
        .products
        .iter_mut()
        .find(|product| product.product_id == 2003)
        .expect("transfer fixture product should exist");
    product.product_type = Some(CHARACTER_TRANSFER_PRODUCT_TYPE);
}

fn complete_first_bnet_transfer(env: &WowLuaEnv) {
    start_original_bnet_validation(env, "first@example.com", 1);
    receive_bnet_validation_snapshot(
        env,
        1,
        0,
        FIRST_BNET_TRANSFER_GUID,
        FIRST_BNET_TRANSFER_ACCOUNTS,
    );
    assert_bnet_transfer_event_matches_current_snapshot(env, 1, None);
}

fn complete_second_bnet_transfer(env: &WowLuaEnv) {
    start_original_bnet_validation(env, "second@example.com", 2);
    receive_bnet_validation_snapshot(
        env,
        2,
        0,
        SECOND_BNET_TRANSFER_GUID,
        SECOND_BNET_TRANSFER_ACCOUNTS,
    );
    assert_bnet_transfer_event_matches_current_snapshot(env, 2, None);
    purchase_original_bnet_transfer(env, "WoW3", SECOND_BNET_TRANSFER_GUID);
    assert_bnet_transfer_snapshot(env, Some(SECOND_BNET_TRANSFER_GUID), &["WoW3"]);
    assert_bnet_transfer_event_snapshot(env, 1, FIRST_BNET_TRANSFER_GUID, &["WoW2", "WoW3"]);
    assert_bnet_transfer_event_snapshot(env, 2, SECOND_BNET_TRANSFER_GUID, &["WoW3"]);
}

fn verify_bnet_transfer_error_stale_and_empty_snapshot(env: &WowLuaEnv) {
    start_bnet_validation_api(env, "third@example.com", 4);
    receive_bnet_validation_snapshot(
        env,
        4,
        7,
        "BNetAccount-ignored",
        &[(b"WoW2", "WowAccount-ignored")],
    );
    assert_bnet_transfer_event_matches_current_snapshot(env, 3, Some(7));
    assert_bnet_transfer_snapshot(env, Some(SECOND_BNET_TRANSFER_GUID), &["WoW3"]);
    receive_bnet_validation_snapshot(
        env,
        2,
        0,
        "BNetAccount-stale",
        &[(b"WoW2", "WowAccount-stale")],
    );
    let event_count: i32 = env
        .eval("return #STORE_VAS_BNET_TRANSFER_EVENTS")
        .expect("stale response should not dispatch an event");
    assert_eq!(event_count, 3);
    assert_bnet_transfer_snapshot(env, Some(SECOND_BNET_TRANSFER_GUID), &["WoW3"]);
    start_bnet_validation_api(env, "empty@example.com", 5);
    receive_bnet_validation_snapshot(env, 5, 0, "", &[]);
    assert_bnet_transfer_event_matches_current_snapshot(env, 4, None);
    assert_bnet_transfer_snapshot(env, None, &[]);
}

fn start_original_bnet_validation(env: &WowLuaEnv, email: &str, sequence: u64) {
    let script = format!("STORE_VAS_BNET_START_VALIDATION({email:?})");
    env.exec(&script)
        .expect("original transfer consumer should submit BNet validation");
    assert_bnet_validation_request_at(env, sequence, email);
}

fn start_bnet_validation_api(env: &WowLuaEnv, email: &str, sequence: u64) {
    let script = format!("C_StoreSecure.ValidateBnetTransfer({email:?})");
    env.exec(&script)
        .expect("ValidateBnetTransfer should start the host request");
    assert_bnet_validation_request_at(env, sequence, email);
}

fn assert_bnet_validation_request_at(env: &WowLuaEnv, sequence: u64, email: &str) {
    let sim = env.state();
    let sim = sim.borrow();
    assert_eq!(sim.store_glue.request_sequence, sequence);
    let expected = StoreVasRequest::BnetTransferValidation {
        sequence,
        account_name: email.as_bytes().to_vec(),
    };
    assert!(
        sim.store_glue
            .store_secure
            .vas
            .submitted_requests
            .contains(&expected)
    );
}

fn receive_bnet_validation_snapshot(
    env: &WowLuaEnv,
    sequence: u64,
    error_code: i32,
    bnet_guid: &str,
    accounts: &[(&[u8], &str)],
) {
    let accounts = accounts
        .iter()
        .map(|(name, guid)| (name.to_vec(), (*guid).to_owned()))
        .collect();
    wow_ui_sim::c_api::store_vas_model::receive_bnet_transfer_validation(
        env,
        sequence,
        error_code,
        bnet_guid.to_owned(),
        accounts,
    )
    .expect("BNet validation response should dispatch through the existing seam");
}

fn bnet_transfer_snapshot(env: &WowLuaEnv) -> (Option<String>, String, i32) {
    env.eval("local guid, accounts = C_StoreSecure.GetBnetTransferInfo(); return guid, table.concat(accounts, '|'), #accounts")
        .expect("GetBnetTransferInfo should return a GUID and account array")
}

fn assert_bnet_transfer_snapshot(
    env: &WowLuaEnv,
    expected_guid: Option<&str>,
    expected_accounts: &[&str],
) {
    let actual = bnet_transfer_snapshot(env);
    assert_eq!(actual.0.as_deref(), expected_guid);
    assert_eq!(actual.1, expected_accounts.join("|"));
    assert_eq!(actual.2, expected_accounts.len() as i32);
}

fn assert_bnet_transfer_event_snapshot(
    env: &WowLuaEnv,
    event_index: usize,
    expected_guid: &str,
    expected_accounts: &[&str],
) {
    let script = format!(
        "local event = STORE_VAS_BNET_TRANSFER_EVENTS[{event_index}]; return event.guid, event.accountNames, event.accountCount"
    );
    let event: (Option<String>, String, i32) = env
        .eval(&script)
        .expect("validation event should snapshot the BNet transfer info");
    assert_eq!(event.0.as_deref(), Some(expected_guid));
    assert_eq!(event.1, expected_accounts.join("|"));
    assert_eq!(event.2, expected_accounts.len() as i32);
}

fn assert_bnet_transfer_event_matches_current_snapshot(
    env: &WowLuaEnv,
    event_index: usize,
    expected_error: Option<i32>,
) {
    let script = format!(
        "local event = STORE_VAS_BNET_TRANSFER_EVENTS[{event_index}]; return event.count, event.errorType, event.errorCode, event.guid, event.accountNames, event.accountCount"
    );
    let event: (i32, String, Option<i32>, Option<String>, String, i32) = env
        .eval(&script)
        .expect("validation event observer should record the published snapshot");
    assert_eq!(event.0, 1);
    assert_eq!(
        event.1,
        if expected_error.is_some() {
            "number"
        } else {
            "nil"
        }
    );
    assert_eq!(event.2, expected_error);
    let snapshot = bnet_transfer_snapshot(env);
    assert_eq!(event.3, snapshot.0);
    assert_eq!(event.4, snapshot.1);
    assert_eq!(event.5, snapshot.2);
}

fn purchase_original_bnet_transfer(env: &WowLuaEnv, account_name: &str, expected_guid: &str) {
    let script = format!("STORE_VAS_BNET_PURCHASE({account_name:?}, {expected_guid:?})");
    env.exec(&script)
        .expect("original transfer consumer should forward the saved BNet GUID");
    let sim = env.state();
    let sim = sim.borrow();
    let request = sim
        .store_glue
        .store_secure
        .vas
        .submitted_requests
        .iter()
        .rev()
        .find(|request| matches!(request, StoreVasRequest::VasPurchase { .. }))
        .expect("original transfer consumer should submit a VAS purchase");
    let StoreVasRequest::VasPurchase {
        bnet_account_guid, ..
    } = request
    else {
        unreachable!("the last VAS request was filtered by its variant");
    };
    let expected = Some(crate::c_api::store_guid::parse(expected_guid.as_bytes()));
    assert_eq!(bnet_account_guid, &expected);
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

#[test]
fn retail_store_vas_purchase_uses_original_continue_button() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        env.state().borrow_mut().store_glue.connection_present = true;
        apply_callable_wrong_producer_overlay(&env);
        env.exec(VAS_PURCHASE_CONSUMER)
            .expect("original VAS continue button should submit its selected purchase");
        assert_original_vas_purchase_request(&env);
    }
}

#[test]
fn retail_store_vas_completion_uses_original_store_ui_handler() {
    test_timeout! {
        let env = load_store_ui();
        seed_realm_and_character_inputs(&env);
        crate::common::store_catalog_fixture::seed_vas_catalog(&env);
        seed_purchase_inputs(&env);
        apply_callable_wrong_producer_overlay(&env);
        env.exec(VAS_COMPLETION_CONSUMER)
            .expect("original completion handler should consume the populated completion row");
    }
}

fn assert_original_vas_purchase_request(env: &WowLuaEnv) {
    let requests = env
        .state()
        .borrow()
        .store_glue
        .store_secure
        .vas
        .submitted_requests
        .clone();
    assert_eq!(requests.len(), 1);
    let StoreVasRequest::VasPurchase {
        sequence,
        product_id,
        character_guid,
        new_name,
        ..
    } = &requests[0]
    else {
        panic!("original VAS continue button should submit a VAS purchase");
    };
    assert_eq!(
        (*sequence, *product_id, new_name.as_deref()),
        (1, 2003, Some(b"Renamedhero".as_slice()))
    );
    assert!(matches!(
        character_guid,
        wow_ui_sim::c_api::store_guid::Outcome::Known(_)
    ));
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
