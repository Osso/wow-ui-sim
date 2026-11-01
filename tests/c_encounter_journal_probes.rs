//! Integration tests for C_EncounterJournal.GetEncounterInfo / GetInstanceInfo.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ---------------------------------------------------------------------------
// GetEncounterInfo
// ---------------------------------------------------------------------------

#[test]
fn test_get_encounter_info_known_returns_fields() {
    let env = env();
    let (name, enc_id, journal_inst_id, inst_id): (String, i64, i64, i64) = env
        .eval(
            r#"
            local name, _, eid, _, _, jiid, _, iid = C_EncounterJournal.GetEncounterInfo(2684)
            return name, eid, jiid, iid
            "#,
        )
        .unwrap();
    assert_eq!(name, "Plexus Sentinel");
    assert_eq!(enc_id, 2684_i64);
    assert_eq!(journal_inst_id, 1302_i64);
    assert_eq!(inst_id, 2460_i64);
}

#[test]
fn test_get_encounter_info_unknown_returns_nothing() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            local name = C_EncounterJournal.GetEncounterInfo(99999)
            return name == nil
            "#,
        )
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_encounter_info_fyrakk() {
    let env = env();
    let (name, inst_id): (String, i64) = env
        .eval(
            r#"
            local name, _, _, _, _, _, _, iid = C_EncounterJournal.GetEncounterInfo(2519)
            return name, iid
            "#,
        )
        .unwrap();
    assert_eq!(name, "Fyrakk the Blazing");
    assert_eq!(inst_id, 2238_i64);
}

#[test]
fn test_get_encounter_info_queen_ansurek() {
    let env = env();
    let (name, inst_id): (String, i64) = env
        .eval(
            r#"
            local name, _, _, _, _, _, _, iid = C_EncounterJournal.GetEncounterInfo(2602)
            return name, iid
            "#,
        )
        .unwrap();
    assert_eq!(name, "Queen Ansurek");
    assert_eq!(inst_id, 2295_i64);
}

#[test]
fn test_get_encounter_info_full_tuple_count() {
    let env = env();
    let count: i64 = env
        .eval(
            r#"
            local a,b,c,d,e,f,g,h = C_EncounterJournal.GetEncounterInfo(2684)
            local n = 0
            for _, v in ipairs({a,b,c,d,e,f,g,h}) do n = n + 1 end
            -- count non-nil manually since select('#',...) won't work here
            local t = {a,b,c,d,e,f,g,h}
            return #t
            "#,
        )
        .unwrap();
    assert_eq!(count, 8_i64);
}

// ---------------------------------------------------------------------------
// GetInstanceInfo
// ---------------------------------------------------------------------------

#[test]
fn test_get_instance_info_known_returns_name() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            local name = C_EncounterJournal.GetInstanceInfo(1302)
            return name
            "#,
        )
        .unwrap();
    assert_eq!(name, "Manaforge Omega");
}

#[test]
fn test_get_instance_info_unknown_returns_nothing() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            local name = C_EncounterJournal.GetInstanceInfo(99999)
            return name == nil
            "#,
        )
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_instance_info_links_current_dungeons_to_lfd_ids() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local _, _, _, _, _, _, _, _, linkDungeonID = C_EncounterJournal.GetInstanceInfo(1271)
            if linkDungeonID ~= 1201 then
                return "c_link=" .. tostring(linkDungeonID)
            end

            local legacyShouldDisplayDifficulty = select(9, EJ_GetInstanceInfo(1271))
            if legacyShouldDisplayDifficulty ~= false then
                return "legacy_difficulty=" .. tostring(legacyShouldDisplayDifficulty)
            end

            local legacyIsRaid = select(12, EJ_GetInstanceInfo(1271))
            if legacyIsRaid ~= false then
                return "legacy_is_raid=" .. tostring(legacyIsRaid)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "Encounter Journal LFD link ids: {result}");
}

#[test]
fn test_get_instance_info_amirdrassil() {
    let env = env();
    let (name, bg_image): (String, i64) = env
        .eval(
            r#"
            local name, _, bg = C_EncounterJournal.GetInstanceInfo(1207)
            return name, bg
            "#,
        )
        .unwrap();
    assert_eq!(name, "Amirdrassil, the Dream's Hope");
    assert!(
        bg_image > 0,
        "expected Amirdrassil to expose a numeric bg fileDataID, got: {bg_image}"
    );
}

#[test]
fn test_select_instance_clears_stale_encounter_for_instance_loot() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            EJ_SelectInstance(1302)
            local instanceLoot = EJ_GetNumLoot()
            if instanceLoot <= 0 then
                return "instance_empty"
            end

            EJ_SelectEncounter(2684)
            local encounterLoot = EJ_GetNumLoot()
            if encounterLoot <= 0 then
                return "encounter_empty"
            end

            EJ_SelectInstance(1302)
            local restoredInstanceLoot = EJ_GetNumLoot()
            if restoredInstanceLoot ~= instanceLoot then
                return "stale_encounter:" .. tostring(restoredInstanceLoot) .. "/" .. tostring(instanceLoot)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "instance loot should not use stale boss selection: {result}"
    );
}

#[test]
fn test_magisters_terrace_loot_rows_have_item_metadata() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            EJ_SelectInstance(1300)
            local count = EJ_GetNumLoot()
            if count < 20 then
                return "count=" .. tostring(count)
            end

            for i = 1, count do
                local item = EJ_GetLootInfoByIndex(i)
                if not item then
                    return "missing_item:" .. tostring(i)
                end
                if item.name == nil or item.name == "" then
                    return "missing_name:" .. tostring(i) .. ":" .. tostring(item.itemID)
                end
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Magisters' Terrace loot should render concrete item rows: {result}"
    );
}

#[test]
fn test_magisters_terrace_loot_rows_are_not_marked_seasonal() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            EJ_SelectInstance(1300)
            local count = EJ_GetNumLoot()
            if count <= 0 then
                return "count=" .. tostring(count)
            end

            for i = 1, count do
                local item = C_EncounterJournal.GetLootInfoByIndex(i)
                if item.displaySeasonID ~= nil then
                    return "seasonal:" .. tostring(i) .. ":" .. tostring(item.itemID) .. ":" .. tostring(item.displaySeasonID)
                end
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "ordinary Adventure Guide loot must not be truthy seasonal loot: {result}"
    );
}

#[test]
fn test_magisters_terrace_loot_rows_have_renderable_text_and_icons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            EJ_SelectInstance(1300)
            local count = EJ_GetNumLoot()
            if count <= 0 then
                return "count=" .. tostring(count)
            end

            for i = 1, count do
                local item = C_EncounterJournal.GetLootInfoByIndex(i)
                if type(item.itemQuality) ~= "string" or not item.itemQuality:match("^ff%x%x%x%x%x%x$") then
                    return "quality:" .. tostring(i) .. ":" .. tostring(item.itemID) .. ":" .. tostring(item.itemQuality)
                end
                if type(item.icon) ~= "number" or item.icon <= 0 then
                    return "icon:" .. tostring(i) .. ":" .. tostring(item.itemID) .. ":" .. tostring(item.icon)
                end
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Adventure Guide loot rows need color-code quality strings and renderable icons: {result}"
    );
}

#[test]
fn test_refueling_orb_uses_spell_name_icon() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            EJ_SelectInstance(1300)
            for i = 1, EJ_GetNumLoot() do
                local item = C_EncounterJournal.GetLootInfoByIndex(i)
                if item.itemID == 250246 then
                    return tostring(item.icon)
                end
            end
            return "missing"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "4914670",
        "Refueling Orb should use its matching non-generic spell icon"
    );
}

#[test]
fn test_adventure_guide_raid_loot_rows_have_item_metadata() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            for _, instanceID in ipairs({1278, 1302, 1307, 1308, 1312, 1314}) do
                EJ_SelectInstance(instanceID)
                local count = EJ_GetNumLoot()
                if count <= 0 then
                    return "count=" .. tostring(instanceID) .. ":" .. tostring(count)
                end

                for i = 1, count do
                    local item = EJ_GetLootInfoByIndex(i)
                    if not item then
                        return "missing_item:" .. tostring(instanceID) .. ":" .. tostring(i)
                    end
                    if item.name == nil or item.name == "" then
                        return "missing_name:" .. tostring(instanceID) .. ":" .. tostring(i) .. ":" .. tostring(item.itemID)
                    end
                end
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Adventure Guide raid loot should render concrete item rows: {result}"
    );
}

#[test]
fn test_encounter_journal_tier_selection_round_trips() {
    let env = env();
    let (initial_tier, selected_tier): (i64, i64) = env
        .eval(
            r#"
            C_EncounterJournal.InitalizeSelectedTier()
            local initialTier = EJ_GetCurrentTier()
            EJ_SelectTier(11)
            return initialTier, EJ_GetCurrentTier()
            "#,
        )
        .unwrap();
    assert_eq!(initial_tier, 10_i64);
    assert_eq!(selected_tier, 11_i64);
}

#[test]
fn test_get_section_icon_flags_returns_array_or_nil() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local flags = C_EncounterJournal.GetSectionIconFlags(820)
            if type(flags) ~= "table" then
                return "nonzero_type=" .. type(flags)
            end
            if flags[1] ~= 1 then
                return "first_flag=" .. tostring(flags[1])
            end

            local noFlags = C_EncounterJournal.GetSectionIconFlags(819)
            if noFlags ~= nil then
                return "zero_type=" .. type(noFlags)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "section icon flag array shape: {result}");
}

#[test]
fn test_get_section_info_resolves_boss_spell_icon() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local section = C_EncounterJournal.GetSectionInfo(31622)
            if not section then
                return "missing_section"
            end
            if section.spellID ~= 1217649 then
                return "spell_id=" .. tostring(section.spellID)
            end

            if type(section.abilityIcon) ~= "number" or section.abilityIcon <= 0 then
                return "ability_icon=" .. tostring(section.abilityIcon)
            end

            local olderSection = C_EncounterJournal.GetSectionInfo(816)
            if olderSection.abilityIcon ~= 132368 then
                return "older_icon=" .. tostring(olderSection.abilityIcon)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "boss spell sections should expose abilityIcon from spell data: {result}"
    );
}

#[test]
fn adventure_journal_suggestions_seed_visible_cards() {
    let env = env();
    env.exec(
        r#"
        assert(C_AdventureJournal.CanBeShown() == true)
        C_AdventureJournal.UpdateSuggestions()

        local suggestions = {}
        C_AdventureJournal.GetSuggestions(suggestions)

        assert(C_AdventureJournal.GetNumAvailableSuggestions() >= 3)
        assert(#suggestions == 3)
        for index, suggestion in ipairs(suggestions) do
            assert(type(suggestion.title) == "string" and suggestion.title ~= "", index)
            assert(type(suggestion.description) == "string" and suggestion.description ~= "", index)
            assert(type(suggestion.buttonText) == "string" and suggestion.buttonText ~= "", index)
            assert(type(suggestion.iconPath) == "string" and suggestion.iconPath ~= "", index)
        end

        assert(suggestions[1].iconPath == "Interface\\Icons\\Achievement_Dungeon_UlduarRaid_Titan_01")
        assert(suggestions[2].iconPath == "Interface\\Icons\\INV_Nerubian_Ring_01_Color5")
        assert(suggestions[3].iconPath == "Interface\\Icons\\INV_Misc_ScrollUnrolled03")

        C_AdventureJournal.SetPrimaryOffset(1)
        assert(C_AdventureJournal.GetPrimaryOffset() == 1)
        C_AdventureJournal.SetPrimaryOffset(999)
        assert(C_AdventureJournal.GetPrimaryOffset() == C_AdventureJournal.GetNumAvailableSuggestions() - 1)
        "#,
    )
    .unwrap();
}

#[test]
fn encounter_journal_search_surface_is_available_and_empty() {
    let env = env();
    env.exec(
        r#"
        assert(type(EJ_EndSearch) == "function")
        assert(type(EJ_ClearSearch) == "function")
        assert(type(EJ_SetSearch) == "function")
        assert(type(EJ_GetSearchSize) == "function")
        assert(type(EJ_GetSearchProgress) == "function")
        assert(type(EJ_GetNumSearchResults) == "function")
        assert(type(EJ_GetSearchResult) == "function")
        assert(type(EJ_IsSearchFinished) == "function")

        EJ_SetSearch("fyrakk")
        EJ_EndSearch()
        EJ_ClearSearch()

        assert(EJ_GetSearchSize() == 0)
        assert(EJ_GetSearchProgress() == 0)
        assert(EJ_GetNumSearchResults() == 0)
        assert(EJ_GetSearchResult(1) == nil)
        assert(EJ_IsSearchFinished() == true)
        "#,
    )
    .unwrap();
}

#[test]
fn get_section_info_matches_section_info_shape() {
    let env = env();
    env.exec(
        r#"
        local info = C_EncounterJournal.GetSectionInfo(820)
        assert(type(info.abilityIcon) == "number", "section info should expose abilityIcon")
        assert(type(info.startsOpen) == "boolean", "section info should expose startsOpen")
        assert(info.iconFlags == nil, "section icon flags belong to GetSectionIconFlags")
        "#,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// GetInstanceForGameMap
// ---------------------------------------------------------------------------

#[test]
fn get_instance_for_game_map_resolves_journal_instances_from_game_maps() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local function show(mapID)
                return tostring(C_EncounterJournal.GetInstanceForGameMap(mapID))
            end
            return table.concat({ show(2810), show(429), show(0), show(999999) }, " ")
            "#,
        )
        .unwrap();
    // Manaforge Omega; Dire Maul's three wings share map 429 and the lowest
    // journal instance (Capital Gardens) wins; map 0 and unknown maps give nil.
    assert_eq!(result, "1302 230 nil nil");
}
