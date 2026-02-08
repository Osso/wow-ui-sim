//! WoW Enum table containing game enumerations.
//!
//! This module registers the global `Enum` table which contains various game
//! enumerations used by addons, such as item quality, quest types, UI widget
//! types, and other game constants.

use super::enum_data::{EXPLICIT_ENUMS, SEQUENTIAL_ENUMS};
use mlua::{Lua, Result};

/// Register the Enum table with all WoW game enumerations.
pub fn register_enum_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let enum_table = lua.create_table()?;

    // Register enums from data file
    register_sequential_enums(lua, &enum_table)?;
    register_explicit_enums(lua, &enum_table)?;

    // Register remaining enums not yet migrated to data file
    register_item_enums(lua, &enum_table)?;
    register_quest_enums(lua, &enum_table)?;
    register_transmog_enums(lua, &enum_table)?;
    register_housing_enums(lua, &enum_table)?;
    register_ui_enums(lua, &enum_table)?;
    register_misc_enums(lua, &enum_table)?;

    globals.set("Enum", enum_table)?;
    Ok(())
}

/// Register all sequential enums (values are 0, 1, 2, ...).
fn register_sequential_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    for (name, variants) in SEQUENTIAL_ENUMS {
        let table = lua.create_table()?;
        for (i, variant) in variants.iter().enumerate() {
            table.set(*variant, i as i32)?;
        }
        enum_table.set(*name, table)?;
    }
    Ok(())
}

/// Register all explicit value enums (values are explicitly specified).
fn register_explicit_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    for (name, variants) in EXPLICIT_ENUMS {
        let table = lua.create_table()?;
        for (variant, value) in *variants {
            table.set(*variant, *value)?;
        }
        enum_table.set(*name, table)?;
    }
    Ok(())
}

/// Helper to register a sequential enum inline.
fn seq_enum(lua: &Lua, enum_table: &mlua::Table, name: &str, variants: &[&str]) -> Result<()> {
    let table = lua.create_table()?;
    for (i, variant) in variants.iter().enumerate() {
        table.set(*variant, i as i32)?;
    }
    enum_table.set(name, table)
}

/// Helper to register an explicit value enum inline.
fn val_enum(lua: &Lua, enum_table: &mlua::Table, name: &str, variants: &[(&str, i32)]) -> Result<()> {
    let table = lua.create_table()?;
    for (variant, value) in variants {
        table.set(*variant, *value)?;
    }
    enum_table.set(name, table)
}

// ============================================================================
// Item Enums
// ============================================================================

fn register_item_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "InventoryType", &[
        "IndexNonEquipType", "IndexHeadType", "IndexNeckType", "IndexShoulderType",
        "IndexBodyType", "IndexChestType", "IndexWaistType", "IndexLegsType",
        "IndexFeetType", "IndexWristType", "IndexHandType", "IndexFingerType",
        "IndexTrinketType", "IndexWeaponType", "IndexShieldType", "IndexRangedType",
        "IndexCloakType", "Index2HweaponType", "IndexBagType", "IndexTabardType",
        "IndexRobeType", "IndexWeaponmainhandType", "IndexWeaponoffhandType",
        "IndexHoldableType", "IndexAmmoType", "IndexThrownType", "IndexRangedrightType",
        "IndexQuiverType", "IndexRelicType", "IndexProfessionToolType",
        "IndexProfessionGearType", "IndexEquipablespellOffensiveType",
        "IndexEquipablespellUtilityType", "IndexEquipablespellDefensiveType",
        "IndexEquipablespellWeaponType",
    ])?;

    seq_enum(lua, enum_table, "ItemWeaponSubclass", &[
        "Axe1H", "Axe2H", "Bows", "Guns", "Mace1H", "Mace2H", "Polearm", "Sword1H",
        "Sword2H", "Warglaive", "Staff", "Bearclaw", "Catclaw", "Unarmed", "Generic",
        "Dagger", "Thrown", "Obsolete3", "Crossbow", "Wand", "Fishingpole",
    ])?;

    seq_enum(lua, enum_table, "ItemArmorSubclass", &[
        "Generic", "Cloth", "Leather", "Mail", "Plate", "Cosmetic", "Shield",
        "Libram", "Idol", "Totem", "Sigil", "Relic",
    ])?;

    val_enum(lua, enum_table, "ItemQuality", &[
        ("Poor", 0), ("Common", 1), ("Uncommon", 2), ("Good", 2), // Good = Uncommon alias
        ("Rare", 3), ("Epic", 4), ("Legendary", 5), ("Artifact", 6),
        ("Heirloom", 7), ("WoWToken", 8),
    ])?;

    val_enum(lua, enum_table, "ItemQualityMeta", &[("NumValues", 9)])?;

    val_enum(lua, enum_table, "WorldQuestQuality", &[
        ("Common", 0), ("Rare", 1), ("Epic", 2),
    ])?;

    val_enum(lua, enum_table, "GarrFollowerQuality", &[
        ("Common", 1), ("Uncommon", 2), ("Rare", 3),
        ("Epic", 4), ("Legendary", 5), ("Title", 6),
    ])?;

    seq_enum(lua, enum_table, "ItemMiscellaneousSubclass", &[
        "Junk", "Reagent", "CompanionPet", "Holiday", "Other", "Mount", "MountEquipment",
    ])?;

    val_enum(lua, enum_table, "MountTypeMeta", &[("NumValues", 20)])?;

    seq_enum(lua, enum_table, "ItemClass", &[
        "Consumable", "Container", "Weapon", "Gem", "Armor", "Reagent", "Projectile",
        "Tradegoods", "ItemEnhancement", "Recipe", "CurrencyTokenObsolete", "Quiver",
        "Questitem", "Key", "PermanentObsolete", "Miscellaneous", "Glyph", "Battlepet",
        "WoWToken", "Profession",
    ])?;

    seq_enum(lua, enum_table, "ItemBind", &[
        "None", "OnAcquire", "OnEquip", "OnUse", "Quest",
    ])?;

    Ok(())
}

// ============================================================================
// Quest Enums
// ============================================================================

fn register_quest_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "QuestClassification", &[
        "Normal", "Questline", "Important", "Legendary", "Campaign", "Calling",
        "Meta", "Recurring", "BonusObjective", "Threat", "WorldQuest",
    ])?;

    seq_enum(lua, enum_table, "QuestTagType", &[
        "Tag", "Profession", "Normal", "PvP", "PetBattle", "Bounty", "Dungeon",
        "Invasion", "Raid", "Contribution", "RatedReward", "InvasionWrapper",
        "FactionAssault", "Islands", "Threat", "CovenantCalling", "DragonRiderRacing",
        "Capstone", "WorldBoss",
    ])?;

    val_enum(lua, enum_table, "QuestTag", &[
        ("Dungeon", 62), ("Raid", 63), ("Raid10", 82), ("Raid25", 83),
        ("Scenario", 98), ("Group", 1), ("Heroic", 104), ("PvP", 41),
        ("Account", 102), ("Legendary", 128), ("Delve", 288),
    ])?;

    seq_enum(lua, enum_table, "ContentTrackingTargetType", &[
        "JournalEncounter", "Vendor", "Achievement", "Profession", "Quest",
    ])?;

    val_enum(lua, enum_table, "QuestRewardContextFlags", &[
        ("None", 0), ("FirstCompletionBonus", 1), ("RepeatCompletionBonus", 2),
    ])?;

    seq_enum(lua, enum_table, "QuestWatchType", &[
        "Automatic", "Manual",
    ])?;

    seq_enum(lua, enum_table, "QuestLineFloorLocation", &[
        "Below", "Same", "Above",
    ])?;

    seq_enum(lua, enum_table, "QuestFrequency", &[
        "Default", "Daily", "Weekly",
    ])?;

    Ok(())
}

// ============================================================================
// Transmog Enums
// ============================================================================

fn register_transmog_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "TransmogCollectionType", &[
        "None", "Head", "Shoulder", "Back", "Chest", "Shirt", "Tabard", "Wrist", "Hands",
        "Waist", "Legs", "Feet", "Wand", "OneHAxe", "OneHSword", "OneHMace",
        "Dagger", "Fist", "Shield", "Holdable", "TwoHAxe", "TwoHSword", "TwoHMace",
        "Staff", "Polearm", "Bow", "Gun", "Crossbow", "Warglaives", "Paired",
    ])?;

    val_enum(lua, enum_table, "TransmogSource", &[
        ("None", 0), ("JournalEncounter", 1), ("Quest", 2), ("Vendor", 3),
        ("WorldDrop", 4), ("HiddenUntilCollected", 5), ("CantCollect", 6),
        ("Achievement", 7), ("Profession", 8), ("NotValidForTransmog", 9),
    ])?;

    seq_enum(lua, enum_table, "TransmogType", &[
        "Appearance", "Illusion",
    ])?;

    seq_enum(lua, enum_table, "TransmogPendingType", &[
        "Apply", "Revert", "ToggleOn", "ToggleOff",
    ])?;

    seq_enum(lua, enum_table, "TransmogModification", &[
        "None", "RightShoulder", "LeftShoulder", "Main", "Secondary",
    ])?;

    Ok(())
}

// ============================================================================
// Housing Enums
// ============================================================================

fn register_housing_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "HouseEditorMode", &[
        "None", "BasicDecor", "ExpertDecor", "Layout", "Customize",
    ])?;

    seq_enum(lua, enum_table, "EditModeSettingDisplayType", &[
        "Dropdown", "Checkbox", "Slider",
    ])?;

    Ok(())
}

// ============================================================================
// UI Enums
// ============================================================================

fn register_ui_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "UITextureSliceMode", &["Stretched", "Tiled"])?;

    seq_enum(lua, enum_table, "UIMapType", &[
        "Cosmic", "World", "Continent", "Zone", "Dungeon", "Micro", "Orphan",
    ])?;

    seq_enum(lua, enum_table, "TugOfWarStyleValue", &[
        "None", "DefaultYellow", "ArchaeologyBrown", "Arrow", "Flames",
    ])?;

    seq_enum(lua, enum_table, "UIWidgetSetLayoutDirection", &[
        "Vertical", "Horizontal", "HorizontalReverse", "VerticalReverse",
    ])?;

    Ok(())
}

// ============================================================================
// Miscellaneous Enums
// ============================================================================

fn register_misc_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "BankType", &["Character", "Account"])?;
    seq_enum(lua, enum_table, "AddOnEnableState", &["None", "Some", "All"])?;

    seq_enum(lua, enum_table, "AddOnProfilerMetric", &[
        "RecentAverageTime", "SessionAverageTime", "PeakTime", "EncounterAverageTime",
    ])?;

    seq_enum(lua, enum_table, "WeeklyRewardChestThresholdType", &[
        "None", "Activities", "Raid", "MythicPlus", "RankedPvP", "World",
    ])?;

    val_enum(lua, enum_table, "Damageclass", &[
        ("MaskPhysical", 1), ("MaskHoly", 2), ("MaskFire", 4), ("MaskNature", 8),
        ("MaskFrost", 16), ("MaskShadow", 32), ("MaskArcane", 64),
    ])?;

    seq_enum(lua, enum_table, "AuctionHouseFilter", &[
        "None", "CommonQuality", "UncommonQuality", "RareQuality", "EpicQuality",
        "LegendaryQuality", "ArtifactQuality", "HeirloomQuality", "UncollectedOnly",
        "CanUseOnly", "UpgradesOnly", "ExactMatch",
    ])?;

    seq_enum(lua, enum_table, "PowerType", &[
        "HealthCost", "Mana", "Rage", "Focus", "Energy", "ComboPoints", "Runes",
        "RunicPower", "SoulShards", "LunarPower", "HolyPower", "Alternate",
        "Maelstrom", "Chi", "Insanity", "Obsolete", "Obsolete2", "ArcaneCharges",
        "Fury", "Pain", "Essence", "RuneBlood", "RuneFrost", "RuneUnholy",
        "AlternateQuest", "AlternateEncounter", "AlternateMount", "NumPowerTypes",
    ])?;

    seq_enum(lua, enum_table, "PingSubjectType", &[
        "Attack", "Warning", "Assist", "OnMyWay", "AlertThreat", "AlertNotThreat",
    ])?;

    seq_enum(lua, enum_table, "TtsBoolSetting", &[
        "PlaySoundSeparatingChatLineBreaks", "AddCharacterNameToSpeech",
        "PlayActivitySoundWhenNotFocused", "AlternateSystemVoice", "NarrateMyMessages",
    ])?;

    register_tooltip_enums(lua, enum_table)?;
    Ok(())
}

fn register_tooltip_enums(lua: &Lua, enum_table: &mlua::Table) -> Result<()> {
    seq_enum(lua, enum_table, "TooltipDataType", &[
        "Item", "Spell", "Unit", "Corpse", "Object", "Currency", "QuestLine",
        "QuestObjective", "QuestLink", "BattlePet", "CompanionPet", "Mount",
        "Toy", "PetAction", "Macro", "EquipmentSet", "Totem", "Achievement",
        "Perk", "RecipeRankInfo", "ItemUpgrade", "Difficulty", "PvPTalent",
        "Flyout", "GuildPerk", "UnitBuff", "UnitDebuff", "UnitAura", "ConduitRank",
        "RuneforgeLegendary", "SoulbindConduit", "Covenant", "LFGDungeonReward",
        "LFGDungeonShortage", "SetBonus", "TraitEntry", "SkillLine", "NewAddon",
        "Addon", "Action", "AzeriteEmpoweredItem", "AzeriteEssence", "AzeritePower",
        "Enchant", "GarrisonBuilding", "GarrisonFollower", "GarrisonMission",
        "GarrisonMissionCompleted", "GarrisonShipment", "GarrisonTalent",
        "GlyphSlot", "ItemCost", "InGameShopItem", "BagItem", "GuildBank",
        "TradeItem", "HeirloomItem", "InstanceLock", "MinimapMouseover",
        "LevelUpReward", "PetCage", "TransmogIllusion", "TransmogAppearance",
    ])?;

    val_enum(lua, enum_table, "TooltipDataLineType", &[
        ("None", 0), ("ItemBinding", 1), ("ItemUnique", 2), ("ItemRefundable", 3),
        ("ItemSellPrice", 4), ("ItemUsable", 5), ("ItemMadeBy", 6), ("ItemLevel", 7),
        ("UnitLevel", 8), ("UnitFaction", 9), ("UnitClass", 10), ("SpellName", 11),
        ("SpellLevel", 12), ("Restricted", 13), ("Health", 14), ("Amount", 15),
        ("SpellTime", 16), ("EquipSlot", 17), ("DifficultyDescription", 18),
    ])?;

    seq_enum(lua, enum_table, "TooltipComparisonMethod", &[
        "Single", "WithBothHands", "WithBagMainHandItem", "WithBagOffHandItem",
    ])?;

    Ok(())
}
