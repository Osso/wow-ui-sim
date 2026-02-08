//! System-related C_* namespaces.
//!
//! This module contains WoW system APIs:
//! - C_XMLUtil - XML template utilities
//! - C_Console - Console command system
//! - C_VoiceChat - Voice chat and TTS
//! - C_TTSSettings - TTS configuration
//! - C_Reputation - Faction reputation system
//! - C_Texture - Texture and atlas handling
//! - C_CreatureInfo - NPC/creature/class/race information
//! - C_Covenants - Shadowlands covenant system
//! - C_Soulbinds - Shadowlands soulbind system

use mlua::{Lua, Result, Value};

/// Register system-related C_* namespaces.
pub fn register_c_system_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("C_XMLUtil", register_c_xml_util(lua)?)?;
    globals.set("C_Console", register_c_console(lua)?)?;
    globals.set("C_VoiceChat", register_c_voice_chat(lua)?)?;
    globals.set("C_TTSSettings", register_c_tts_settings(lua)?)?;
    globals.set("C_Reputation", register_c_reputation(lua)?)?;
    globals.set("C_Texture", register_c_texture(lua)?)?;
    globals.set("C_CreatureInfo", register_c_creature_info(lua)?)?;
    globals.set("C_Covenants", register_c_covenants(lua)?)?;
    globals.set("C_Soulbinds", register_c_soulbinds(lua)?)?;

    Ok(())
}

/// C_XMLUtil namespace - XML template utilities.
fn register_c_xml_util(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    // GetTemplateInfo(templateName) - returns template type, width, height
    t.set(
        "GetTemplateInfo",
        lua.create_function(|lua, template_name: String| {
            if let Some(info) = crate::xml::get_template_info(&template_name) {
                let result = lua.create_table()?;
                result.set("type", info.frame_type)?;
                result.set("width", info.width)?;
                result.set("height", info.height)?;
                Ok(Value::Table(result))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    Ok(t)
}

/// C_Console namespace - console command system.
fn register_c_console(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetAllCommands",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "GetColorFromType",
        lua.create_function(|lua, _command_type: i32| {
            let color = lua.create_table()?;
            color.set("r", 1.0)?;
            color.set("g", 1.0)?;
            color.set("b", 1.0)?;
            Ok(color)
        })?,
    )?;

    Ok(t)
}

/// C_VoiceChat namespace - voice chat and TTS.
fn register_c_voice_chat(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "SpeakText",
        lua.create_function(
            |_, (_voice_id, _text, _dest, _rate, _volume): (i32, String, i32, i32, i32)| Ok(()),
        )?,
    )?;
    t.set(
        "StopSpeakingText",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    t.set(
        "IsSpeakingText",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    t.set(
        "GetTtsVoices",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set("IsSpeakForMeActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsTranscriptionAllowed", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsTranscribing", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActiveChannelType", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsMuted", lua.create_function(|_, ()| Ok(false))?)?;

    Ok(t)
}

/// C_TTSSettings namespace - TTS settings.
fn register_c_tts_settings(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetSpeechRate", lua.create_function(|_, ()| Ok(0))?)?;
    t.set("SetSpeechRate", lua.create_function(|_, _rate: i32| Ok(()))?)?;
    t.set("GetSpeechVolume", lua.create_function(|_, ()| Ok(100))?)?;
    t.set("SetSpeechVolume", lua.create_function(|_, _volume: i32| Ok(()))?)?;
    t.set("GetVoiceOptionID", lua.create_function(|_, _option: i32| Ok(0))?)?;
    t.set(
        "SetVoiceOption",
        lua.create_function(|_, (_option, _voice_id): (i32, i32)| Ok(()))?,
    )?;

    Ok(t)
}

/// C_Reputation namespace - faction reputation system.
fn register_c_reputation(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetFactionDataByID", lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?)?;
    t.set("IsFactionParagon", lua.create_function(|_, _faction_id: i32| Ok(false))?)?;
    t.set("GetFactionParagonInfo", lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?)?;
    t.set("GetNumFactions", lua.create_function(|_, ()| Ok(0))?)?;
    t.set("GetFactionInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    t.set("GetWatchedFactionData", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("SetWatchedFactionByID", lua.create_function(|_, _faction_id: i32| Ok(()))?)?;

    Ok(t)
}

/// C_Texture namespace - texture handling.
fn register_c_texture(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetAtlasInfo",
        lua.create_function(|lua, atlas_name: String| {
            if let Some(lookup) = crate::atlas::get_atlas_info(&atlas_name) {
                let info = lua.create_table()?;
                info.set("width", lookup.width())?;
                info.set("height", lookup.height())?;
                info.set("leftTexCoord", lookup.info.left_tex_coord)?;
                info.set("rightTexCoord", lookup.info.right_tex_coord)?;
                info.set("topTexCoord", lookup.info.top_tex_coord)?;
                info.set("bottomTexCoord", lookup.info.bottom_tex_coord)?;
                info.set("file", lookup.info.file)?;
                info.set("tilesHorizontally", lookup.info.tiles_horizontally)?;
                info.set("tilesVertically", lookup.info.tiles_vertically)?;
                Ok(Value::Table(info))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;
    t.set(
        "GetFilenameFromFileDataID",
        lua.create_function(|_, _file_data_id: i32| Ok(Value::Nil))?,
    )?;

    Ok(t)
}

/// C_CreatureInfo namespace - NPC/creature information.
fn register_c_creature_info(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetClassInfo", lua.create_function(create_get_class_info)?)?;
    t.set("GetRaceInfo", lua.create_function(create_get_race_info)?)?;
    t.set(
        "GetCreatureTypeIDs",
        lua.create_function(|lua, ()| {
            let ids = lua.create_table()?;
            for (i, id) in (1..=10).enumerate() {
                ids.set(i + 1, id)?;
            }
            Ok(ids)
        })?,
    )?;
    t.set("GetCreatureTypeInfo", lua.create_function(create_get_creature_type_info)?)?;
    t.set(
        "GetCreatureFamilyIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "GetCreatureFamilyInfo",
        lua.create_function(|_, _family_id: i32| Ok(Value::Nil))?,
    )?;
    t.set("GetFactionInfo", lua.create_function(create_get_faction_info)?)?;

    Ok(t)
}

fn create_get_class_info(lua: &Lua, class_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    let class_name = match class_id {
        1 => "WARRIOR",
        2 => "PALADIN",
        3 => "HUNTER",
        4 => "ROGUE",
        5 => "PRIEST",
        6 => "DEATHKNIGHT",
        7 => "SHAMAN",
        8 => "MAGE",
        9 => "WARLOCK",
        10 => "MONK",
        11 => "DRUID",
        12 => "DEMONHUNTER",
        13 => "EVOKER",
        _ => "UNKNOWN",
    };
    info.set("className", class_name)?;
    info.set("classFile", class_name)?;
    info.set("classID", class_id)?;
    Ok(Value::Table(info))
}

fn create_get_race_info(lua: &Lua, race_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    let (race_name, client_file) = match race_id {
        1 => ("Human", "Human"),
        2 => ("Orc", "Orc"),
        3 => ("Dwarf", "Dwarf"),
        4 => ("Night Elf", "NightElf"),
        5 => ("Undead", "Scourge"),
        6 => ("Tauren", "Tauren"),
        7 => ("Gnome", "Gnome"),
        8 => ("Troll", "Troll"),
        9 => ("Goblin", "Goblin"),
        10 => ("Blood Elf", "BloodElf"),
        11 => ("Draenei", "Draenei"),
        22 => ("Worgen", "Worgen"),
        24 => ("Pandaren", "Pandaren"),
        25 => ("Pandaren", "Pandaren"),
        26 => ("Pandaren", "Pandaren"),
        27 => ("Nightborne", "Nightborne"),
        28 => ("Highmountain Tauren", "HighmountainTauren"),
        29 => ("Void Elf", "VoidElf"),
        30 => ("Lightforged Draenei", "LightforgedDraenei"),
        31 => ("Zandalari Troll", "ZandalariTroll"),
        32 => ("Kul Tiran", "KulTiran"),
        34 => ("Dark Iron Dwarf", "DarkIronDwarf"),
        35 => ("Vulpera", "Vulpera"),
        36 => ("Mag'har Orc", "MagharOrc"),
        37 => ("Mechagnome", "Mechagnome"),
        52 | 70 => ("Dracthyr", "Dracthyr"),
        84 | 85 => ("Earthen", "Earthen"),
        _ => ("Unknown", "Unknown"),
    };
    info.set("raceName", race_name)?;
    info.set("raceID", race_id)?;
    info.set("clientFileString", client_file)?;
    Ok(Value::Table(info))
}

fn create_get_creature_type_info(lua: &Lua, creature_type_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    let name = match creature_type_id {
        1 => "Beast",
        2 => "Dragonkin",
        3 => "Demon",
        4 => "Elemental",
        5 => "Giant",
        6 => "Undead",
        7 => "Humanoid",
        8 => "Critter",
        9 => "Mechanical",
        10 => "Not specified",
        _ => "Unknown",
    };
    info.set("name", name)?;
    info.set("creatureTypeID", creature_type_id)?;
    Ok(Value::Table(info))
}

fn create_get_faction_info(lua: &Lua, race_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    let (name, group_tag) = match race_id {
        // Alliance races
        1 | 3 | 4 | 7 | 11 | 22 | 29 | 30 | 32 | 34 | 37 => ("Alliance", "Alliance"),
        // Horde races
        2 | 5 | 6 | 8 | 9 | 10 | 27 | 28 | 31 | 35 | 36 => ("Horde", "Horde"),
        // Neutral (Pandaren)
        24..=26 => ("Neutral", "Neutral"),
        // Dracthyr - can be either, default to neutral
        52 | 70 => ("Neutral", "Neutral"),
        // Earthen - can be either, default to neutral
        84 | 85 => ("Neutral", "Neutral"),
        _ => return Ok(Value::Nil),
    };
    info.set("name", name)?;
    info.set("groupTag", group_tag)?;
    Ok(Value::Table(info))
}

/// C_Covenants namespace - Shadowlands covenant system.
fn register_c_covenants(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetCovenantData",
        lua.create_function(|lua, covenant_id: i32| {
            let data = lua.create_table()?;
            let name = match covenant_id {
                1 => "Kyrian",
                2 => "Venthyr",
                3 => "Night Fae",
                4 => "Necrolord",
                _ => "None",
            };
            data.set("ID", covenant_id)?;
            data.set("name", name)?;
            data.set("textureKit", "")?;
            Ok(Value::Table(data))
        })?,
    )?;
    t.set(
        "GetActiveCovenantID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    t.set(
        "GetCovenantIDs",
        lua.create_function(|lua, ()| {
            let ids = lua.create_table()?;
            for i in 1..=4 {
                ids.set(i, i)?;
            }
            Ok(ids)
        })?,
    )?;

    Ok(t)
}

/// C_Soulbinds namespace - Shadowlands soulbind system.
fn register_c_soulbinds(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetActiveSoulbindID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    t.set(
        "GetSoulbindData",
        lua.create_function(|lua, _soulbind_id: i32| {
            let data = lua.create_table()?;
            data.set("ID", 0)?;
            data.set("name", "")?;
            data.set("covenantID", 0)?;
            Ok(Value::Table(data))
        })?,
    )?;
    t.set(
        "GetConduitCollection",
        lua.create_function(|lua, _conduit_type: i32| lua.create_table())?,
    )?;
    t.set(
        "GetConduitCollectionData",
        lua.create_function(|_, _conduit_id: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "IsConduitInstalled",
        lua.create_function(|_, (_soulbind_id, _conduit_id): (i32, i32)| Ok(false))?,
    )?;

    Ok(t)
}
