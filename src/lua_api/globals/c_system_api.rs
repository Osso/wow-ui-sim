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

    // C_XMLUtil namespace - XML template utilities
    let c_xml_util = lua.create_table()?;

    // GetTemplateInfo(templateName) - returns template type, width, height
    c_xml_util.set(
        "GetTemplateInfo",
        lua.create_function(|lua, template_name: String| {
            if let Some(info) = crate::xml::get_template_info(&template_name) {
                let result = lua.create_table()?;
                // WoW uses lowercase "type" for frame type
                result.set("type", info.frame_type)?;
                result.set("width", info.width)?;
                result.set("height", info.height)?;
                Ok(Value::Table(result))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;
    globals.set("C_XMLUtil", c_xml_util)?;

    // C_Console namespace - console command system
    let c_console = lua.create_table()?;
    c_console.set(
        "GetAllCommands",
        lua.create_function(|lua, ()| {
            // Return empty table of console commands
            lua.create_table()
        })?,
    )?;
    c_console.set(
        "GetColorFromType",
        lua.create_function(|lua, _command_type: i32| {
            let color = lua.create_table()?;
            color.set("r", 1.0)?;
            color.set("g", 1.0)?;
            color.set("b", 1.0)?;
            Ok(color)
        })?,
    )?;
    globals.set("C_Console", c_console)?;

    // C_VoiceChat namespace - voice chat and TTS
    let c_voice_chat = lua.create_table()?;
    c_voice_chat.set(
        "SpeakText",
        lua.create_function(|_, (_voice_id, _text, _dest, _rate, _volume): (i32, String, i32, i32, i32)| {
            // Stub - would play TTS in real game
            Ok(())
        })?,
    )?;
    c_voice_chat.set(
        "StopSpeakingText",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_voice_chat.set(
        "IsSpeakingText",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_voice_chat.set(
        "GetTtsVoices",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    globals.set("C_VoiceChat", c_voice_chat)?;

    // C_TTSSettings namespace - TTS settings
    let c_tts_settings = lua.create_table()?;
    c_tts_settings.set(
        "GetSpeechRate",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_tts_settings.set(
        "SetSpeechRate",
        lua.create_function(|_, _rate: i32| Ok(()))?,
    )?;
    c_tts_settings.set(
        "GetSpeechVolume",
        lua.create_function(|_, ()| Ok(100))?,
    )?;
    c_tts_settings.set(
        "SetSpeechVolume",
        lua.create_function(|_, _volume: i32| Ok(()))?,
    )?;
    c_tts_settings.set(
        "GetVoiceOptionID",
        lua.create_function(|_, _option: i32| Ok(0))?,
    )?;
    c_tts_settings.set(
        "SetVoiceOption",
        lua.create_function(|_, (_option, _voice_id): (i32, i32)| Ok(()))?,
    )?;
    globals.set("C_TTSSettings", c_tts_settings)?;

    // C_Reputation namespace - faction reputation system
    let c_reputation = lua.create_table()?;
    c_reputation.set(
        "GetFactionDataByID",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "IsFactionParagon",
        lua.create_function(|_, _faction_id: i32| Ok(false))?,
    )?;
    c_reputation.set(
        "GetFactionParagonInfo",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "GetNumFactions",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_reputation.set(
        "GetFactionInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "GetWatchedFactionData",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "SetWatchedFactionByID",
        lua.create_function(|_, _faction_id: i32| Ok(()))?,
    )?;
    globals.set("C_Reputation", c_reputation)?;

    // C_Texture namespace - texture handling
    let c_texture = lua.create_table()?;
    c_texture.set(
        "GetAtlasInfo",
        lua.create_function(|lua, atlas_name: String| {
            // Look up atlas in our database
            if let Some(atlas_info) = crate::atlas::get_atlas_info(&atlas_name) {
                let info = lua.create_table()?;
                info.set("width", atlas_info.width)?;
                info.set("height", atlas_info.height)?;
                info.set("leftTexCoord", atlas_info.left_tex_coord)?;
                info.set("rightTexCoord", atlas_info.right_tex_coord)?;
                info.set("topTexCoord", atlas_info.top_tex_coord)?;
                info.set("bottomTexCoord", atlas_info.bottom_tex_coord)?;
                info.set("file", atlas_info.file)?;
                info.set("tilesHorizontally", atlas_info.tiles_horizontally)?;
                info.set("tilesVertically", atlas_info.tiles_vertically)?;
                Ok(Value::Table(info))
            } else {
                // Return nil for unknown atlases
                Ok(Value::Nil)
            }
        })?,
    )?;
    c_texture.set(
        "GetFilenameFromFileDataID",
        lua.create_function(|_, _file_data_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_Texture", c_texture)?;

    // C_CreatureInfo namespace - NPC/creature information
    let c_creature_info = lua.create_table()?;
    c_creature_info.set(
        "GetClassInfo",
        lua.create_function(|lua, class_id: i32| {
            // Return class info table
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
        })?,
    )?;
    c_creature_info.set(
        "GetRaceInfo",
        lua.create_function(|lua, race_id: i32| {
            let info = lua.create_table()?;
            // WoW race data: (name, clientFileString)
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
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureTypeIDs",
        lua.create_function(|lua, ()| {
            // WoW creature types: Beast, Dragonkin, Demon, Elemental, Giant, Undead, Humanoid, Critter, Mechanical, etc.
            let ids = lua.create_table()?;
            for (i, id) in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter().enumerate() {
                ids.set(i + 1, *id)?;
            }
            Ok(ids)
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureTypeInfo",
        lua.create_function(|lua, creature_type_id: i32| {
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
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureFamilyIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_creature_info.set(
        "GetCreatureFamilyInfo",
        lua.create_function(|_, _family_id: i32| Ok(Value::Nil))?,
    )?;
    c_creature_info.set(
        "GetFactionInfo",
        lua.create_function(|lua, race_id: i32| {
            // Return faction info for a race
            let info = lua.create_table()?;
            // Map races to factions: Alliance (0) or Horde (1)
            let (name, group_tag) = match race_id {
                // Alliance races
                1 | 3 | 4 | 7 | 11 | 22 | 29 | 30 | 32 | 34 | 37 => ("Alliance", "Alliance"),
                // Horde races
                2 | 5 | 6 | 8 | 9 | 10 | 27 | 28 | 31 | 35 | 36 => ("Horde", "Horde"),
                // Neutral (Pandaren)
                24 | 25 | 26 => ("Neutral", "Neutral"),
                // Dracthyr - can be either, default to neutral
                52 | 70 => ("Neutral", "Neutral"),
                // Earthen - can be either, default to neutral
                84 | 85 => ("Neutral", "Neutral"),
                _ => return Ok(Value::Nil),
            };
            info.set("name", name)?;
            info.set("groupTag", group_tag)?;
            Ok(Value::Table(info))
        })?,
    )?;
    globals.set("C_CreatureInfo", c_creature_info)?;

    // C_Covenants namespace - Shadowlands covenant system
    let c_covenants = lua.create_table()?;
    c_covenants.set(
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
    c_covenants.set(
        "GetActiveCovenantID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_covenants.set(
        "GetCovenantIDs",
        lua.create_function(|lua, ()| {
            let ids = lua.create_table()?;
            ids.set(1, 1)?;
            ids.set(2, 2)?;
            ids.set(3, 3)?;
            ids.set(4, 4)?;
            Ok(ids)
        })?,
    )?;
    globals.set("C_Covenants", c_covenants)?;

    // C_Soulbinds namespace - Shadowlands soulbind system
    let c_soulbinds = lua.create_table()?;
    c_soulbinds.set(
        "GetActiveSoulbindID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_soulbinds.set(
        "GetSoulbindData",
        lua.create_function(|lua, _soulbind_id: i32| {
            let data = lua.create_table()?;
            data.set("ID", 0)?;
            data.set("name", "")?;
            data.set("covenantID", 0)?;
            Ok(Value::Table(data))
        })?,
    )?;
    c_soulbinds.set(
        "GetConduitCollection",
        lua.create_function(|lua, _conduit_type: i32| lua.create_table())?,
    )?;
    c_soulbinds.set(
        "GetConduitCollectionData",
        lua.create_function(|_, _conduit_id: i32| Ok(Value::Nil))?,
    )?;
    c_soulbinds.set(
        "IsConduitInstalled",
        lua.create_function(|_, (_soulbind_id, _conduit_id): (i32, i32)| Ok(false))?,
    )?;
    globals.set("C_Soulbinds", c_soulbinds)?;

    Ok(())
}
