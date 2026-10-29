//! Temporary character creation default-state workaround.
//!
//! The character creation UI expects selected race/class/faction fields and a
//! few frame surfaces to be ready by the time creation mixins run. Seed them
//! here until the simulator models the full character-create state flow.

use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::{LuaApiMut, LuaResult, vm::state::LuaState};

const CHARACTER_CREATION_NAMESPACE_DEFAULTS_LUA: &str = r#"
local function noop()
end

if type(C_CharacterCreation) ~= "table" then
    C_CharacterCreation = {}
end

local races = rawget(_G, "__wow_character_create_races")
if type(races) ~= "table" then
    races = {
        { raceID = 1, name = "Human", fileName = "Human", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Versatile and determined.", createScreenIconAtlas = "charactercreate-humans" },
        { raceID = 2, name = "Orc", fileName = "Orc", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Fierce warriors from Draenor.", createScreenIconAtlas = "charactercreate-orcs" },
        { raceID = 3, name = "Dwarf", fileName = "Dwarf", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Stout defenders of Khaz Modan.", createScreenIconAtlas = "charactercreate-dwarves" },
        { raceID = 4, name = "Night Elf", fileName = "NightElf", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Ancient guardians of nature.", createScreenIconAtlas = "charactercreate-nightelves" },
        { raceID = 5, name = "Undead", fileName = "Scourge", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Forsaken who fight for their future.", createScreenIconAtlas = "charactercreate-undead" },
        { raceID = 6, name = "Tauren", fileName = "Tauren", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Noble protectors of the plains.", createScreenIconAtlas = "charactercreate-tauren" },
        { raceID = 7, name = "Gnome", fileName = "Gnome", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Inventive and resilient.", createScreenIconAtlas = "charactercreate-gnomes" },
        { raceID = 8, name = "Troll", fileName = "Troll", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Regenerating jungle fighters.", createScreenIconAtlas = "charactercreate-trolls" },
        { raceID = 9, name = "Goblin", fileName = "Goblin", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Clever survivors of Kezan.", createScreenIconAtlas = "charactercreate-goblins" },
        { raceID = 10, name = "Blood Elf", fileName = "BloodElf", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Arcane guardians of Quel'Thalas.", createScreenIconAtlas = "charactercreate-bloodelf" },
        { raceID = 11, name = "Draenei", fileName = "Draenei", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Exiled servants of the Light.", createScreenIconAtlas = "charactercreate-draenei" },
        { raceID = 22, name = "Worgen", fileName = "Worgen", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = false, loreDescription = "Cursed defenders of Gilneas.", createScreenIconAtlas = "charactercreate-worgen" },
        { raceID = 24, name = "Pandaren", fileName = "Pandaren", factionInternalName = "Neutral", enabled = true, isNeutralRace = true, isAlliedRace = false, loreDescription = "Wanderers from the Wandering Isle.", createScreenIconAtlas = "charactercreate-pandaren" },
        { raceID = 27, name = "Nightborne", fileName = "Nightborne", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Ancient arcanists of Suramar.", createScreenIconAtlas = "charactercreate-nightborne" },
        { raceID = 28, name = "Highmountain Tauren", fileName = "HighmountainTauren", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Protectors of Highmountain.", createScreenIconAtlas = "charactercreate-highmountain" },
        { raceID = 29, name = "Void Elf", fileName = "VoidElf", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Elves touched by the Void.", createScreenIconAtlas = "charactercreate-voidelf" },
        { raceID = 30, name = "Lightforged Draenei", fileName = "LightforgedDraenei", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Veterans of the Army of the Light.", createScreenIconAtlas = "charactercreate-lightforged" },
        { raceID = 31, name = "Zandalari Troll", fileName = "ZandalariTroll", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Proud trolls of Zandalar.", createScreenIconAtlas = "charactercreate-zandalari" },
        { raceID = 32, name = "Kul Tiran", fileName = "KulTiran", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Seafarers of Kul Tiras.", createScreenIconAtlas = "charactercreate-kultiran" },
        { raceID = 34, name = "Dark Iron Dwarf", fileName = "DarkIronDwarf", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Fiery dwarves of Blackrock.", createScreenIconAtlas = "charactercreate-darkirondwarf" },
        { raceID = 35, name = "Vulpera", fileName = "Vulpera", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Resourceful nomads of Vol'dun.", createScreenIconAtlas = "charactercreate-vulpera" },
        { raceID = 36, name = "Mag'har Orc", fileName = "MagharOrc", factionInternalName = "Horde", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Uncorrupted orcs of Draenor.", createScreenIconAtlas = "charactercreate-maghar" },
        { raceID = 37, name = "Mechagnome", fileName = "Mechagnome", factionInternalName = "Alliance", enabled = true, isNeutralRace = false, isAlliedRace = true, loreDescription = "Mechanically enhanced gnomes.", createScreenIconAtlas = "charactercreate-mechagnome" },
        { raceID = 52, name = "Dracthyr", fileName = "Dracthyr", factionInternalName = "Neutral", enabled = true, isNeutralRace = true, isAlliedRace = false, loreDescription = "Dragonkin soldiers of the dracthyr.", createScreenIconAtlas = "charactercreate-dracthyr" },
    }
    rawset(_G, "__wow_character_create_races", races)
end

local classes = rawget(_G, "__wow_character_create_classes")
if type(classes) ~= "table" then
    classes = {
        { classID = 1, fileName = "WARRIOR", name = "Warrior", description = "Armored melee fighter.", roleInfo = "Tank, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 2, fileName = "PALADIN", name = "Paladin", description = "Holy warrior.", roleInfo = "Tank, Healer, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 3, fileName = "HUNTER", name = "Hunter", description = "Ranged tracker with pets.", roleInfo = "Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 4, fileName = "ROGUE", name = "Rogue", description = "Stealthy opportunist.", roleInfo = "Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 5, fileName = "PRIEST", name = "Priest", description = "Light and shadow caster.", roleInfo = "Healer, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 6, fileName = "DEATHKNIGHT", name = "Death Knight", description = "Runeblade champion.", roleInfo = "Tank, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 7, fileName = "SHAMAN", name = "Shaman", description = "Elemental spiritualist.", roleInfo = "Healer, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 8, fileName = "MAGE", name = "Mage", description = "Master of arcane power.", roleInfo = "Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 9, fileName = "WARLOCK", name = "Warlock", description = "Fel caster with demonic allies.", roleInfo = "Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 10, fileName = "MONK", name = "Monk", description = "Martial artist with mystic focus.", roleInfo = "Tank, Healer, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 11, fileName = "DRUID", name = "Druid", description = "Shapeshifter of the wilds.", roleInfo = "Tank, Healer, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 12, fileName = "DEMONHUNTER", name = "Demon Hunter", description = "Agile hunter of the Legion.", roleInfo = "Tank, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
        { classID = 13, fileName = "EVOKER", name = "Evoker", description = "Dracthyr spellcaster wielding dragonflights.", roleInfo = "Healer, Damage", enabled = true, animLoopWaitTimeSeconds = 0.5 },
    }
    rawset(_G, "__wow_character_create_classes", classes)
end

local function clone_table(value)
    local copy = {}
    for key, item in pairs(value or {}) do
        copy[key] = item
    end
    return copy
end

local function customization_option_type(kind)
    if Enum ~= nil and Enum.ChrCustomizationOptionType ~= nil and Enum.ChrCustomizationOptionType[kind] ~= nil then
        return Enum.ChrCustomizationOptionType[kind]
    end
    if kind == "Checkbox" then
        return 1
    elseif kind == "Slider" then
        return 2
    end
    return 0
end

local function find_race(raceID)
    for _, raceData in ipairs(races) do
        if raceData.raceID == raceID then
            return clone_table(raceData)
        end
    end
    return nil
end

local function find_class(classID)
    for _, classData in ipairs(classes) do
        if classData.classID == classID then
            return clone_table(classData)
        end
    end
    return nil
end

local function character_create_categories()
    local function choices(baseID, names)
        local out = {}
        for index, name in ipairs(names) do
            out[index] = {
                id = baseID + index - 1,
                choiceIndex = index,
                name = name,
            }
        end
        return out
    end

    return {
        {
            id = 1,
            name = "Face",
            options = {
                { id = 101, orderIndex = 1, name = "Face Shape", optionType = customization_option_type("Dropdown"), currentChoiceIndex = 1, choices = choices(1001, { "Face 1", "Face 2", "Face 3" }) },
                { id = 102, orderIndex = 2, name = "Skin Tone", optionType = customization_option_type("Slider"), currentChoiceIndex = 2, choices = choices(1011, { "Tone 1", "Tone 2", "Tone 3" }) },
            },
        },
        {
            id = 2,
            name = "Hair",
            options = {
                { id = 201, orderIndex = 1, name = "Hair Style", optionType = customization_option_type("Dropdown"), currentChoiceIndex = 1, choices = choices(2001, { "Style 1", "Style 2", "Style 3" }) },
                { id = 202, orderIndex = 2, name = "Hair Color", optionType = customization_option_type("Dropdown"), currentChoiceIndex = 2, choices = choices(2011, { "Color 1", "Color 2", "Color 3" }) },
            },
        },
        {
            id = 3,
            name = "Details",
            options = {
                { id = 301, orderIndex = 1, name = "Accessories", optionType = customization_option_type("Checkbox"), currentChoiceIndex = 1, choices = choices(3001, { "Off", "On" }) },
                { id = 302, orderIndex = 2, name = "Markings", optionType = customization_option_type("Dropdown"), currentChoiceIndex = 1, choices = choices(3011, { "Marking 1", "Marking 2" }) },
            },
        },
    }
end

rawset(_G, "__wow_selected_race_id", rawget(_G, "__wow_selected_race_id") or races[1].raceID)
rawset(_G, "__wow_selected_class_id", rawget(_G, "__wow_selected_class_id") or classes[1].classID)
rawset(_G, "__wow_selected_sex_id", rawget(_G, "__wow_selected_sex_id") or 0)
rawset(_G, "__wow_character_create_type", rawget(_G, "__wow_character_create_type") or (Enum ~= nil and Enum.CharacterCreateType ~= nil and Enum.CharacterCreateType.Normal or 0))

if rawget(C_CharacterCreation, "GetNumCharacterTemplates") == nil then
    function C_CharacterCreation.GetNumCharacterTemplates()
        return 0
    end
end
if rawget(C_CharacterCreation, "GetBlockedRaces") == nil then
    function C_CharacterCreation.GetBlockedRaces()
        return {}
    end
end
if rawget(C_CharacterCreation, "GetSelectedRace") == nil then
    function C_CharacterCreation.GetSelectedRace()
        return rawget(_G, "__wow_selected_race_id") or races[1].raceID
    end
end
if rawget(C_CharacterCreation, "SetSelectedRace") == nil then
    function C_CharacterCreation.SetSelectedRace(raceID)
        local selectedRace = find_race(raceID)
        rawset(_G, "__wow_selected_race_id", selectedRace and selectedRace.raceID or races[1].raceID)
    end
end
if rawget(C_CharacterCreation, "GetAvailableRaces") == nil then
    function C_CharacterCreation.GetAvailableRaces()
        local out = {}
        for index, raceData in ipairs(races) do
            out[index] = clone_table(raceData)
        end
        return out
    end
end
if rawget(C_CharacterCreation, "GetRaceDataByID") == nil then
    function C_CharacterCreation.GetRaceDataByID(raceID)
        return raceID ~= nil and find_race(raceID) or nil
    end
end
if rawget(C_CharacterCreation, "SetSelectedClass") == nil then
    function C_CharacterCreation.SetSelectedClass(classID)
        local selectedClass = find_class(classID)
        rawset(_G, "__wow_selected_class_id", selectedClass and selectedClass.classID or classes[1].classID)
    end
end
if rawget(C_CharacterCreation, "GetAvailableClasses") == nil then
    function C_CharacterCreation.GetAvailableClasses()
        local out = {}
        for index, classData in ipairs(classes) do
            out[index] = clone_table(classData)
        end
        return out
    end
end
if rawget(C_CharacterCreation, "GetSelectedClass") == nil then
    function C_CharacterCreation.GetSelectedClass()
        return find_class(rawget(_G, "__wow_selected_class_id")) or find_class(classes[1].classID)
    end
end
if rawget(C_CharacterCreation, "SetSelectedSex") == nil then
    function C_CharacterCreation.SetSelectedSex(sexID)
        rawset(_G, "__wow_selected_sex_id", sexID or 0)
    end
end
if rawget(C_CharacterCreation, "GetSelectedSex") == nil then
    function C_CharacterCreation.GetSelectedSex()
        return rawget(_G, "__wow_selected_sex_id") or 0
    end
end
if rawget(C_CharacterCreation, "GetFactionForRace") == nil then
    function C_CharacterCreation.GetFactionForRace(raceID)
        local raceData = find_race(raceID)
        return raceData and raceData.factionInternalName or "Alliance"
    end
end
if rawget(C_CharacterCreation, "GetNameForRace") == nil then
    function C_CharacterCreation.GetNameForRace(raceID)
        local raceData = find_race(raceID)
        return raceData and raceData.name or "Human"
    end
end
if rawget(C_CharacterCreation, "GetClassAchievementRequirements") == nil then
    function C_CharacterCreation.GetClassAchievementRequirements(_raceID, _classID)
        return {}
    end
end
if rawget(C_CharacterCreation, "GetValidRacesForClass") == nil then
    function C_CharacterCreation.GetValidRacesForClass(_classID)
        return C_CharacterCreation.GetAvailableRaces()
    end
end
if rawget(C_CharacterCreation, "GetAlliedRaceAchievementRequirements") == nil then
    function C_CharacterCreation.GetAlliedRaceAchievementRequirements(_raceID)
        return {}
    end
end
if rawget(C_CharacterCreation, "UseBeginnerMode") == nil then
    function C_CharacterCreation.UseBeginnerMode()
        return false
    end
end
if rawget(C_CharacterCreation, "IsViewingAlteredForm") == nil then
    function C_CharacterCreation.IsViewingAlteredForm()
        return false
    end
end
if rawget(C_CharacterCreation, "IsUsingCharacterTemplate") == nil then
    function C_CharacterCreation.IsUsingCharacterTemplate()
        return false
    end
end
if rawget(C_CharacterCreation, "IsForcingCharacterTemplate") == nil then
    function C_CharacterCreation.IsForcingCharacterTemplate()
        return false
    end
end
if rawget(C_CharacterCreation, "IsTimerunningEnabled") == nil then
    function C_CharacterCreation.IsTimerunningEnabled()
        return rawget(_G, "__wow_timerunning_season_id") ~= nil
    end
end
if rawget(C_CharacterCreation, "IsNewPlayerRestricted") == nil then
    function C_CharacterCreation.IsNewPlayerRestricted()
        return false
    end
end
if rawget(C_CharacterCreation, "IsTrialAccountRestricted") == nil then
    function C_CharacterCreation.IsTrialAccountRestricted()
        return false
    end
end
if rawget(C_CharacterCreation, "GetCharacterCreateType") == nil then
    function C_CharacterCreation.GetCharacterCreateType()
        return rawget(_G, "__wow_character_create_type") or (Enum ~= nil and Enum.CharacterCreateType ~= nil and Enum.CharacterCreateType.Normal or 0)
    end
end
if rawget(C_CharacterCreation, "SetCharacterCreateType") == nil then
    function C_CharacterCreation.SetCharacterCreateType(characterCreateType)
        rawset(_G, "__wow_character_create_type", characterCreateType)
    end
end
if rawget(C_CharacterCreation, "SetTimerunningSeasonID") == nil then
    function C_CharacterCreation.SetTimerunningSeasonID(seasonID)
        rawset(_G, "__wow_timerunning_season_id", seasonID)
    end
end
for _, name in ipairs({
    "ClearCharacterTemplate",
    "ResetCharCustomize",
    "SetCharCustomizeFrame",
    "SetCharCustomizeBackground",
    "SetModelAlpha",
    "PlayClassIdleAnimationOnCharacter",
    "PlayCustomizationIdleAnimationOnCharacter",
    "DestroyAuxModel",
}) do
    if rawget(C_CharacterCreation, name) == nil then
        C_CharacterCreation[name] = noop
    end
end
if rawget(C_CharacterCreation, "GetCreateBackgroundModel") == nil then
    function C_CharacterCreation.GetCreateBackgroundModel()
        return 0
    end
end
if rawget(C_CharacterCreation, "GetAvailableCustomizations") == nil then
    function C_CharacterCreation.GetAvailableCustomizations()
        return character_create_categories()
    end
end
if rawget(C_CharacterCreation, "IsCharacterNameValid") == nil then
    function C_CharacterCreation.IsCharacterNameValid(_name)
        return true, ""
    end
end
if rawget(C_CharacterCreation, "IsGuildNameValid") == nil then
    function C_CharacterCreation.IsGuildNameValid(_name)
        return true, ""
    end
end
if rawget(C_CharacterCreation, "CreateCharacter") == nil then
    function C_CharacterCreation.CreateCharacter(name)
        if A_Admin and A_Admin.SetPlayerName then
            A_Admin.SetPlayerName(name)
        end
    end
end

function C_CharacterCreation.GetSelectedRace()
    return rawget(_G, "__wow_selected_race_id") or races[1].raceID
end
function C_CharacterCreation.SetSelectedRace(raceID)
    local selectedRace = find_race(raceID)
    rawset(_G, "__wow_selected_race_id", selectedRace and selectedRace.raceID or races[1].raceID)
end
function C_CharacterCreation.GetAvailableRaces()
    local out = {}
    for index, raceData in ipairs(races) do
        out[index] = clone_table(raceData)
    end
    return out
end
function C_CharacterCreation.GetRaceDataByID(raceID)
    return raceID ~= nil and find_race(raceID) or nil
end
function C_CharacterCreation.SetSelectedClass(classID)
    local selectedClass = find_class(classID)
    rawset(_G, "__wow_selected_class_id", selectedClass and selectedClass.classID or classes[1].classID)
end
function C_CharacterCreation.GetAvailableClasses()
    local out = {}
    for index, classData in ipairs(classes) do
        out[index] = clone_table(classData)
    end
    return out
end
function C_CharacterCreation.GetSelectedClass()
    return find_class(rawget(_G, "__wow_selected_class_id")) or find_class(classes[1].classID)
end
function C_CharacterCreation.SetSelectedSex(sexID)
    rawset(_G, "__wow_selected_sex_id", sexID or 0)
end
function C_CharacterCreation.GetSelectedSex()
    return rawget(_G, "__wow_selected_sex_id") or 0
end
function C_CharacterCreation.GetFactionForRace(raceID)
    local raceData = find_race(raceID)
    return raceData and raceData.factionInternalName or "Alliance"
end
function C_CharacterCreation.GetNameForRace(raceID)
    local raceData = find_race(raceID)
    return raceData and raceData.name or "Human"
end
function C_CharacterCreation.GetAvailableCustomizations()
    return character_create_categories()
end
"#;

const CHARACTER_CREATE_DEFAULTS_WORKAROUND_LUA: &str = r#"
local function __wow_character_create_defaults_frame()
    if type(CharacterCreateFrame) ~= "table" then
        return nil
    end
    return CharacterCreateFrame.RaceAndClassFrame
end

local function __wow_seed_character_create_defaults(frame)
    if type(frame) ~= "table" then
        return
    end

    local raceID = C_CharacterCreation and C_CharacterCreation.GetSelectedRace and C_CharacterCreation.GetSelectedRace() or 1
    if type(frame.selectedRaceData) ~= "table" then
        frame.selectedRaceData = C_CharacterCreation and C_CharacterCreation.GetRaceDataByID and C_CharacterCreation.GetRaceDataByID(raceID) or { enabled = true, isNeutralRace = false, factionInternalName = "Alliance" }
    end
    if type(frame.selectedClassData) ~= "table" then
        frame.selectedClassData = C_CharacterCreation and C_CharacterCreation.GetSelectedClass and C_CharacterCreation.GetSelectedClass() or { classID = 2, earlyFactionChoice = false }
    end
    if frame.selectedFaction == nil and C_CharacterCreation and C_CharacterCreation.GetFactionForRace then
        frame.selectedFaction = C_CharacterCreation.GetFactionForRace(raceID)
    end
end

local function __wow_seed_character_create_frame(frame)
    if type(frame) ~= "table" then
        return
    end

    if type(frame.BGTex) ~= "table" then
        frame.BGTex = {}
    end

    if type(frame.BackButton) == "table"
        and type(frame.BackButton.UpdateText) == "function"
        and type(frame.BackButton.GetText) == "function"
        and (frame.BackButton:GetText() == nil or frame.BackButton:GetText() == "")
    then
        frame.BackButton:UpdateText(BACK, BACKWARD_ARROW)
    end

    if type(frame.UpdateForwardButton) == "function" then
        frame:UpdateForwardButton()
    end
end

local characterCreateFrame = type(CharacterCreateFrame) == "table" and CharacterCreateFrame or nil
local raceAndClassFrame = characterCreateFrame and characterCreateFrame.RaceAndClassFrame or nil
if raceAndClassFrame ~= nil then
    __wow_seed_character_create_defaults(raceAndClassFrame)
end
if characterCreateFrame ~= nil then
    __wow_seed_character_create_frame(characterCreateFrame)
end

if type(CharacterCreateMixin) == "table" and type(CharacterCreateMixin.CreateCharacter) == "function" and not rawget(_G, "__wow_character_create_defaults_patched") then
    local originalCreateCharacter = CharacterCreateMixin.CreateCharacter
    function CharacterCreateMixin:CreateCharacter(...)
        __wow_seed_character_create_defaults(__wow_character_create_defaults_frame())
        __wow_seed_character_create_frame(self)
        if A_Admin and type(A_Admin.SetPlayerName) == "function" and type(self.GetSelectedName) == "function" then
            A_Admin.SetPlayerName(self:GetSelectedName())
        end
        return originalCreateCharacter(self, ...)
    end
    rawset(_G, "__wow_character_create_defaults_patched", true)
end

if type(CharacterCreateRaceAndClassMixin) == "table" and type(CharacterCreateRaceAndClassMixin.GetCreateCharacterFaction) == "function" and not rawget(_G, "__wow_character_create_faction_patched") then
    local originalGetCreateCharacterFaction = CharacterCreateRaceAndClassMixin.GetCreateCharacterFaction
    function CharacterCreateRaceAndClassMixin:GetCreateCharacterFaction()
        __wow_seed_character_create_defaults(self)
        return originalGetCreateCharacterFaction(self)
    end
    rawset(_G, "__wow_character_create_faction_patched", true)
end

if type(CharacterCreateRaceAndClassMixin) == "table" and type(CharacterCreateRaceAndClassMixin.UpdateState) == "function" and not rawget(_G, "__wow_character_create_update_patched") then
    local originalUpdateState = CharacterCreateRaceAndClassMixin.UpdateState
    function CharacterCreateRaceAndClassMixin:UpdateState(selectedFaction)
        __wow_seed_character_create_defaults(self)
        local result = originalUpdateState(self, selectedFaction)
        __wow_seed_character_create_frame(CharacterCreateFrame)
        return result
    end
    rawset(_G, "__wow_character_create_update_patched", true)
end

if type(CharacterCreateMixin) == "table" and type(CharacterCreateMixin.UpdateBackgroundOverlays) == "function" and not rawget(_G, "__wow_character_create_background_overlay_patched") then
    local originalUpdateBackgroundOverlays = CharacterCreateMixin.UpdateBackgroundOverlays
    function CharacterCreateMixin:UpdateBackgroundOverlays(selectedClassData, selectedRaceData)
        local ok = pcall(originalUpdateBackgroundOverlays, self, selectedClassData, selectedRaceData)
        if ok then
            return
        end

        local backgroundTextures = self and self.BGTex or nil
        if type(backgroundTextures) == "table" then
            local iter_ok, iter, state, first = pcall(ipairs, backgroundTextures)
            if iter_ok and type(iter) == "function" then
                local didSetAlpha = false
                for _, texture in iter, state, first do
                    if type(texture) == "table" and type(texture.SetAlpha) == "function" then
                        texture:SetAlpha(1)
                        didSetAlpha = true
                    end
                end
                if didSetAlpha then
                    return
                end
            end
        end

        if type(backgroundTextures) == "table" and type(backgroundTextures.SetAlpha) == "function" then
            backgroundTextures:SetAlpha(1)
        end
    end
    rawset(_G, "__wow_character_create_background_overlay_patched", true)
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CHARACTER_CREATION_NAMESPACE_DEFAULTS_LUA)?;
    crate::c_api::c_character_creation_seen::install(lua)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        // User-approved, not native evidence: record the character-create screen state.
        let state = lua.state_mut();
        let namespace = crate::c_api::ensure_namespace(state, "C_CharacterCreation")?;
        table_set_rust_fn_static(
            state,
            namespace,
            "SetInCharacterCreate",
            set_in_character_create,
        )?;
        // User-approved default until character customization preview choices are modeled.
        lua.exec("function C_CharacterCreation.ClearPreviewChoices(clearSavedChoices) end")?;
        // User-approved, not native evidence: facing feeds only 3D model rotation, a non-goal.
        lua.exec("function C_CharacterCreation.GetDefaultCharacterCreateFacing() return 0 end")?;
        // Stored facing round-trips as Blizzard_CharacterCreate's rotate code expects; it starts at the default.
        lua.exec(
            "local facing = 0; function C_CharacterCreation.GetCharacterCreateFacing() return facing end; function C_CharacterCreation.SetCharacterCreateFacing(value) facing = value end",
        )?;
    }
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        // User-approved defaults until realm name-reservation and regional naming policies are modeled.
        lua.exec("function IsNameReservationOnly() return false end; function C_CharacterCreation.AreRegionalUniqueNamesEnabled() return false end")?;
    }
    Ok(())
}

fn set_in_character_create(state: &mut LuaState) -> LuaResult<u32> {
    let value = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.in_character_create = value;
    Ok(0)
}

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(CHARACTER_CREATE_DEFAULTS_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_initial_character_create_state_and_buttons() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            BACK = "Back"
            BACKWARD_ARROW = "<"
            C_CharacterCreation = {
                GetSelectedRace = function()
                    return 9
                end,
                GetRaceDataByID = function(raceID)
                    return { raceID = raceID, enabled = true }
                end,
                GetSelectedClass = function()
                    return { classID = 5, earlyFactionChoice = false }
                end,
                GetFactionForRace = function(raceID)
                    return "RaceFaction" .. tostring(raceID)
                end,
            }
            CharacterCreateFrame = {
                RaceAndClassFrame = {},
                BackButton = {
                    text = "",
                    GetText = function(self)
                        return self.text
                    end,
                    UpdateText = function(self, text, arrow)
                        self.text = text
                        self.arrow = arrow
                    end,
                },
                UpdateForwardButton = function(self)
                    self.forwardUpdated = true
                end,
            }
            "#,
        )
        .expect("character-create test surface should install");

        patch(&env);

        let (race_id, class_id, faction, back_text, back_arrow, forward_updated): (
            i64,
            i64,
            String,
            String,
            String,
            bool,
        ) = env
            .eval(
                r#"
                local raceFrame = CharacterCreateFrame.RaceAndClassFrame
                return raceFrame.selectedRaceData.raceID,
                    raceFrame.selectedClassData.classID,
                    raceFrame.selectedFaction,
                    CharacterCreateFrame.BackButton.text,
                    CharacterCreateFrame.BackButton.arrow,
                    CharacterCreateFrame.forwardUpdated
                "#,
            )
            .expect("seeded character-create state should be readable");

        assert_eq!(race_id, 9);
        assert_eq!(class_id, 5);
        assert_eq!(faction, "RaceFaction9");
        assert_eq!(back_text, "Back");
        assert_eq!(back_arrow, "<");
        assert!(forward_updated);
    }

    #[test]
    fn create_character_wrapper_seeds_state_and_admin_name() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            BACK = "Back"
            BACKWARD_ARROW = "<"
            C_CharacterCreation = {
                GetSelectedRace = function()
                    return 1
                end,
                GetRaceDataByID = function(raceID)
                    return { raceID = raceID }
                end,
                GetSelectedClass = function()
                    return { classID = 2 }
                end,
                GetFactionForRace = function()
                    return "Alliance"
                end,
            }
            A_Admin = {
                SetPlayerName = function(name)
                    A_Admin.playerName = name
                end,
            }
            CharacterCreateMixin = {
                CreateCharacter = function(self)
                    self.created = true
                    return "created"
                end,
            }
            CharacterCreateFrame = {
                RaceAndClassFrame = {},
                GetSelectedName = function()
                    return "Calia"
                end,
                UpdateForwardButton = function() end,
            }
            "#,
        )
        .expect("create-character wrapper test surface should install");

        patch(&env);

        let (result, created, player_name, selected_faction): (String, bool, String, String) = env
            .eval(
                r#"
                local result = CharacterCreateMixin.CreateCharacter(CharacterCreateFrame)
                return result,
                    CharacterCreateFrame.created,
                    A_Admin.playerName,
                    CharacterCreateFrame.RaceAndClassFrame.selectedFaction
                "#,
            )
            .expect("wrapped CreateCharacter state should be readable");

        assert_eq!(result, "created");
        assert!(created);
        assert_eq!(player_name, "Calia");
        assert_eq!(selected_faction, "Alliance");
    }

    #[test]
    fn installs_character_creation_namespace_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (race_count, class_count, category_count, selected_race, selected_class, selected_sex): (
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
        ) = env
            .eval(
                r#"
                local races = C_CharacterCreation.GetAvailableRaces()
                local classes = C_CharacterCreation.GetAvailableClasses()
                local categories = C_CharacterCreation.GetAvailableCustomizations()
                C_CharacterCreation.SetSelectedRace(2)
                C_CharacterCreation.SetSelectedClass(1)
                C_CharacterCreation.SetSelectedSex(1)
                return #races,
                    #classes,
                    #categories,
                    C_CharacterCreation.GetSelectedRace(),
                    C_CharacterCreation.GetSelectedClass().classID,
                    C_CharacterCreation.GetSelectedSex()
                "#,
            )
            .expect("character-create namespace defaults should be readable");

        assert!(
            race_count >= 20,
            "expected many race defaults, got {race_count}"
        );
        assert!(
            class_count >= 13,
            "expected many class defaults, got {class_count}"
        );
        assert!(
            category_count >= 3,
            "expected customization categories, got {category_count}"
        );
        assert_eq!(selected_race, 2);
        assert_eq!(selected_class, 1);
        assert_eq!(selected_sex, 1);
    }

    #[test]
    fn background_overlay_fallback_sets_array_or_single_texture_alpha() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            CharacterCreateMixin = {
                UpdateBackgroundOverlays = function()
                    error("missing race/class art")
                end,
            }
            CharacterCreateFrame = { RaceAndClassFrame = {} }
            "#,
        )
        .expect("background overlay test surface should install");

        patch(&env);

        let (array_alpha, single_alpha): (i64, i64) = env
            .eval(
                r#"
                local arrayTexture = {
                    SetAlpha = function(self, alpha)
                        self.alpha = alpha
                    end,
                }
                CharacterCreateMixin.UpdateBackgroundOverlays({ BGTex = { arrayTexture } })

                local singleTexture = {
                    SetAlpha = function(self, alpha)
                        self.alpha = alpha
                    end,
                }
                CharacterCreateMixin.UpdateBackgroundOverlays({ BGTex = singleTexture })

                return arrayTexture.alpha, singleTexture.alpha
                "#,
            )
            .expect("background overlay fallback alpha should be readable");

        assert_eq!(array_alpha, 1);
        assert_eq!(single_alpha, 1);
    }
}
