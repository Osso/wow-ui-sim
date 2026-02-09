//! Tests for C_* system API namespaces (c_system_api.rs).
//!
//! Covers: C_XMLUtil, C_Console, C_VoiceChat, C_TTSSettings, C_Reputation,
//! C_Texture, C_CreatureInfo, C_Covenants, C_Soulbinds.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_XMLUtil
// ============================================================================

#[test]
fn test_c_xml_util_get_template_info_unknown() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_XMLUtil.GetTemplateInfo('NonExistentTemplate12345') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_xml_util_namespace_exists() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_XMLUtil) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_c_xml_util_get_template_info_is_function() {
    let env = env();
    let is_func: bool = env
        .eval("return type(C_XMLUtil.GetTemplateInfo) == 'function'")
        .unwrap();
    assert!(is_func);
}

// ============================================================================
// C_Console
// ============================================================================

#[test]
fn test_c_console_get_all_commands_returns_table() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_Console.GetAllCommands()) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_c_console_get_all_commands_empty() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for _ in pairs(C_Console.GetAllCommands()) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_c_console_get_color_from_type() {
    let env = env();
    let (r, g, b): (f64, f64, f64) = env
        .eval(
            r#"
            local c = C_Console.GetColorFromType(0)
            return c.r, c.g, c.b
            "#,
        )
        .unwrap();
    assert_eq!(r, 1.0);
    assert_eq!(g, 1.0);
    assert_eq!(b, 1.0);
}

// ============================================================================
// C_VoiceChat
// ============================================================================

#[test]
fn test_c_voice_chat_speak_text_noop() {
    let env = env();
    env.eval::<()>("C_VoiceChat.SpeakText(0, 'hello', 0, 0, 100)")
        .unwrap();
}

#[test]
fn test_c_voice_chat_stop_speaking_text_noop() {
    let env = env();
    env.eval::<()>("C_VoiceChat.StopSpeakingText()").unwrap();
}

#[test]
fn test_c_voice_chat_is_speaking_text_false() {
    let env = env();
    let speaking: bool = env
        .eval("return C_VoiceChat.IsSpeakingText()")
        .unwrap();
    assert!(!speaking);
}

#[test]
fn test_c_voice_chat_get_tts_voices_empty() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for _ in pairs(C_VoiceChat.GetTtsVoices()) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

// ============================================================================
// C_TTSSettings
// ============================================================================

#[test]
fn test_c_tts_settings_get_speech_rate() {
    let env = env();
    let rate: i32 = env
        .eval("return C_TTSSettings.GetSpeechRate()")
        .unwrap();
    assert_eq!(rate, 0);
}

#[test]
fn test_c_tts_settings_set_speech_rate_noop() {
    let env = env();
    env.eval::<()>("C_TTSSettings.SetSpeechRate(5)").unwrap();
}

#[test]
fn test_c_tts_settings_get_speech_volume() {
    let env = env();
    let volume: i32 = env
        .eval("return C_TTSSettings.GetSpeechVolume()")
        .unwrap();
    assert_eq!(volume, 100);
}

#[test]
fn test_c_tts_settings_set_speech_volume_noop() {
    let env = env();
    env.eval::<()>("C_TTSSettings.SetSpeechVolume(50)").unwrap();
}

#[test]
fn test_c_tts_settings_get_voice_option_id() {
    let env = env();
    let id: i32 = env
        .eval("return C_TTSSettings.GetVoiceOptionID(1)")
        .unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_c_tts_settings_set_voice_option_noop() {
    let env = env();
    env.eval::<()>("C_TTSSettings.SetVoiceOption(1, 2)").unwrap();
}

// ============================================================================
// C_Reputation
// ============================================================================

#[test]
fn test_c_reputation_get_faction_data_by_id_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Reputation.GetFactionDataByID(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_reputation_is_faction_paragon_false() {
    let env = env();
    let is_paragon: bool = env
        .eval("return C_Reputation.IsFactionParagon(1)")
        .unwrap();
    assert!(!is_paragon);
}

#[test]
fn test_c_reputation_get_faction_paragon_info_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Reputation.GetFactionParagonInfo(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_reputation_get_num_factions() {
    let env = env();
    let count: i32 = env
        .eval("return C_Reputation.GetNumFactions()")
        .unwrap();
    assert!(count > 0, "expected faction list to have entries");
}

#[test]
fn test_c_reputation_get_faction_info() {
    let env = env();
    let name: String = env
        .eval("return C_Reputation.GetFactionInfo(1).name")
        .unwrap();
    assert!(!name.is_empty(), "expected first faction to have a name");
}

#[test]
fn test_c_reputation_get_faction_info_out_of_range_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Reputation.GetFactionInfo(9999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_reputation_get_watched_faction_data() {
    let env = env();
    let name: String = env
        .eval("return C_Reputation.GetWatchedFactionData().name")
        .unwrap();
    assert!(!name.is_empty(), "expected watched faction to have a name");
}

#[test]
fn test_c_reputation_set_watched_faction_by_id_noop() {
    let env = env();
    env.eval::<()>("C_Reputation.SetWatchedFactionByID(1)")
        .unwrap();
}

// ============================================================================
// C_Texture
// ============================================================================

#[test]
fn test_c_texture_get_atlas_info_unknown_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Texture.GetAtlasInfo('NonExistentAtlas99999') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_texture_get_atlas_info_is_function() {
    let env = env();
    let is_func: bool = env
        .eval("return type(C_Texture.GetAtlasInfo) == 'function'")
        .unwrap();
    assert!(is_func);
}

#[test]
fn test_c_texture_get_filename_from_file_data_id_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Texture.GetFilenameFromFileDataID(12345) == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// C_CreatureInfo - GetClassInfo
// ============================================================================

#[test]
fn test_c_creature_info_get_class_info_warrior() {
    let env = env();
    let (class_name, class_file, class_id): (String, String, i32) = env
        .eval(
            r#"
            local info = C_CreatureInfo.GetClassInfo(1)
            return info.className, info.classFile, info.classID
            "#,
        )
        .unwrap();
    assert_eq!(class_name, "WARRIOR");
    assert_eq!(class_file, "WARRIOR");
    assert_eq!(class_id, 1);
}

#[test]
fn test_c_creature_info_get_class_info_evoker() {
    let env = env();
    let class_name: String = env
        .eval("return C_CreatureInfo.GetClassInfo(13).className")
        .unwrap();
    assert_eq!(class_name, "EVOKER");
}

#[test]
fn test_c_creature_info_get_class_info_unknown() {
    let env = env();
    let class_name: String = env
        .eval("return C_CreatureInfo.GetClassInfo(99).className")
        .unwrap();
    assert_eq!(class_name, "UNKNOWN");
}

// ============================================================================
// C_CreatureInfo - GetRaceInfo
// ============================================================================

#[test]
fn test_c_creature_info_get_race_info_human() {
    let env = env();
    let (race_name, client_file, race_id): (String, String, i32) = env
        .eval(
            r#"
            local info = C_CreatureInfo.GetRaceInfo(1)
            return info.raceName, info.clientFileString, info.raceID
            "#,
        )
        .unwrap();
    assert_eq!(race_name, "Human");
    assert_eq!(client_file, "Human");
    assert_eq!(race_id, 1);
}

#[test]
fn test_c_creature_info_get_race_info_night_elf() {
    let env = env();
    let (race_name, client_file): (String, String) = env
        .eval(
            r#"
            local info = C_CreatureInfo.GetRaceInfo(4)
            return info.raceName, info.clientFileString
            "#,
        )
        .unwrap();
    assert_eq!(race_name, "Night Elf");
    assert_eq!(client_file, "NightElf");
}

#[test]
fn test_c_creature_info_get_race_info_dracthyr() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetRaceInfo(52).raceName")
        .unwrap();
    assert_eq!(name, "Dracthyr");
    let name: String = env
        .eval("return C_CreatureInfo.GetRaceInfo(70).raceName")
        .unwrap();
    assert_eq!(name, "Dracthyr");
}

#[test]
fn test_c_creature_info_get_race_info_earthen() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetRaceInfo(84).raceName")
        .unwrap();
    assert_eq!(name, "Earthen");
    let name: String = env
        .eval("return C_CreatureInfo.GetRaceInfo(85).raceName")
        .unwrap();
    assert_eq!(name, "Earthen");
}

#[test]
fn test_c_creature_info_get_race_info_unknown() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetRaceInfo(9999).raceName")
        .unwrap();
    assert_eq!(name, "Unknown");
}

// ============================================================================
// C_CreatureInfo - GetCreatureTypeIDs / GetCreatureTypeInfo
// ============================================================================

#[test]
fn test_c_creature_info_get_creature_type_ids() {
    let env = env();
    let count: i32 = env
        .eval("return #C_CreatureInfo.GetCreatureTypeIDs()")
        .unwrap();
    assert_eq!(count, 10);
}

#[test]
fn test_c_creature_info_get_creature_type_info_beast() {
    let env = env();
    let (name, type_id): (String, i32) = env
        .eval(
            r#"
            local info = C_CreatureInfo.GetCreatureTypeInfo(1)
            return info.name, info.creatureTypeID
            "#,
        )
        .unwrap();
    assert_eq!(name, "Beast");
    assert_eq!(type_id, 1);
}

#[test]
fn test_c_creature_info_get_creature_type_info_humanoid() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetCreatureTypeInfo(7).name")
        .unwrap();
    assert_eq!(name, "Humanoid");
}

#[test]
fn test_c_creature_info_get_creature_type_info_unknown() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetCreatureTypeInfo(99).name")
        .unwrap();
    assert_eq!(name, "Unknown");
}

// ============================================================================
// C_CreatureInfo - GetCreatureFamilyIDs / GetCreatureFamilyInfo
// ============================================================================

#[test]
fn test_c_creature_info_get_creature_family_ids_empty() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for _ in pairs(C_CreatureInfo.GetCreatureFamilyIDs()) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_c_creature_info_get_creature_family_info_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_CreatureInfo.GetCreatureFamilyInfo(1) == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// C_CreatureInfo - GetFactionInfo
// ============================================================================

#[test]
fn test_c_creature_info_get_faction_info_alliance_human() {
    let env = env();
    let (name, group_tag): (String, String) = env
        .eval(
            r#"
            local info = C_CreatureInfo.GetFactionInfo(1)
            return info.name, info.groupTag
            "#,
        )
        .unwrap();
    assert_eq!(name, "Alliance");
    assert_eq!(group_tag, "Alliance");
}

#[test]
fn test_c_creature_info_get_faction_info_horde_orc() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetFactionInfo(2).name")
        .unwrap();
    assert_eq!(name, "Horde");
}

#[test]
fn test_c_creature_info_get_faction_info_neutral_pandaren() {
    let env = env();
    let name: String = env
        .eval("return C_CreatureInfo.GetFactionInfo(24).name")
        .unwrap();
    assert_eq!(name, "Neutral");
}

#[test]
fn test_c_creature_info_get_faction_info_unknown_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_CreatureInfo.GetFactionInfo(9999) == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// C_Covenants
// ============================================================================

#[test]
fn test_c_covenants_get_covenant_data_kyrian() {
    let env = env();
    let (id, name): (i32, String) = env
        .eval(
            r#"
            local data = C_Covenants.GetCovenantData(1)
            return data.ID, data.name
            "#,
        )
        .unwrap();
    assert_eq!(id, 1);
    assert_eq!(name, "Kyrian");
}

#[test]
fn test_c_covenants_get_covenant_data_necrolord() {
    let env = env();
    let name: String = env
        .eval("return C_Covenants.GetCovenantData(4).name")
        .unwrap();
    assert_eq!(name, "Necrolord");
}

#[test]
fn test_c_covenants_get_covenant_data_unknown() {
    let env = env();
    let name: String = env
        .eval("return C_Covenants.GetCovenantData(99).name")
        .unwrap();
    assert_eq!(name, "None");
}

#[test]
fn test_c_covenants_get_active_covenant_id_zero() {
    let env = env();
    let id: i32 = env
        .eval("return C_Covenants.GetActiveCovenantID()")
        .unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_c_covenants_get_covenant_ids() {
    let env = env();
    let (a, b, c, d): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local ids = C_Covenants.GetCovenantIDs()
            return ids[1], ids[2], ids[3], ids[4]
            "#,
        )
        .unwrap();
    assert_eq!((a, b, c, d), (1, 2, 3, 4));
}

// ============================================================================
// C_Soulbinds
// ============================================================================

#[test]
fn test_c_soulbinds_get_active_soulbind_id_zero() {
    let env = env();
    let id: i32 = env
        .eval("return C_Soulbinds.GetActiveSoulbindID()")
        .unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_c_soulbinds_get_soulbind_data_defaults() {
    let env = env();
    let (id, name, covenant_id): (i32, String, i32) = env
        .eval(
            r#"
            local data = C_Soulbinds.GetSoulbindData(1)
            return data.ID, data.name, data.covenantID
            "#,
        )
        .unwrap();
    assert_eq!(id, 0);
    assert_eq!(name, "");
    assert_eq!(covenant_id, 0);
}

#[test]
fn test_c_soulbinds_get_conduit_collection_empty() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for _ in pairs(C_Soulbinds.GetConduitCollection(1)) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_c_soulbinds_get_conduit_collection_data_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Soulbinds.GetConduitCollectionData(1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_soulbinds_is_conduit_installed_false() {
    let env = env();
    let installed: bool = env
        .eval("return C_Soulbinds.IsConduitInstalled(1, 1)")
        .unwrap();
    assert!(!installed);
}

// ============================================================================
// All namespaces exist as tables
// ============================================================================

#[test]
fn test_all_namespaces_are_tables() {
    let env = env();
    for ns in &[
        "C_XMLUtil",
        "C_Console",
        "C_VoiceChat",
        "C_TTSSettings",
        "C_Reputation",
        "C_Texture",
        "C_CreatureInfo",
        "C_Covenants",
        "C_Soulbinds",
    ] {
        let is_table: bool = env
            .eval(&format!("return type({}) == 'table'", ns))
            .unwrap();
        assert!(is_table, "{} should be a table", ns);
    }
}
