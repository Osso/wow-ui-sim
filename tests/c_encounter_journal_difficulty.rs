#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const DUNGEON_DIFFICULTY_QUERY: &str = r#"
            local hasNormal = C_EncounterJournal.InstanceHasDifficultyID(1)
            local hasHeroic = C_EncounterJournal.InstanceHasDifficultyID(2)
            local hasRaid = C_EncounterJournal.InstanceHasDifficultyID(14)
            local isHeroicValid = EJ_IsValidInstanceDifficulty(2)
            local raidMaskWithoutMapRow = EJ_IsValidInstanceDifficulty(14)
            EncounterJournal_SetupDifficultyDropdown(EncounterJournal)
            local values = {}
            EncounterJournal.encounter.info.difficulty.menuGenerator(
                EncounterJournal.encounter.info.difficulty,
                {
                    SetTag = function() end,
                    CreateRadio = function(_, _, _, _, value)
                        values[#values + 1] = value
                    end,
                })
            table.sort(values)
            return hasNormal, hasHeroic, hasRaid, isHeroicValid,
                raidMaskWithoutMapRow, table.concat(values, ",")
            "#;

const RAID_DIFFICULTY_QUERY: &str = r#"
            local hasTenPlayer = C_EncounterJournal.InstanceHasDifficultyID(3)
            local hasDungeon = C_EncounterJournal.InstanceHasDifficultyID(1)
            local isTenPlayerValid = EJ_IsValidInstanceDifficulty(3)
            EncounterJournal_SetupDifficultyDropdown(EncounterJournal)
            local values = {}
            EncounterJournal.encounter.info.difficulty.menuGenerator(
                EncounterJournal.encounter.info.difficulty,
                {
                    SetTag = function() end,
                    CreateRadio = function(_, _, _, _, value)
                        values[#values + 1] = value
                    end,
                })
            table.sort(values)
            return hasTenPlayer, hasDungeon, isTenPlayerValid, table.concat(values, ",")
            "#;

const BASE_DIFFICULTY_QUERY: &str = r#"
            return C_EncounterJournal.GetBaseDifficultyID(233),
                C_EncounterJournal.GetBaseDifficultyID(250),
                C_EncounterJournal.GetBaseDifficultyID(63)
            "#;

fn assert_dungeon_difficulty_queries(env: &WowLuaEnv) {
    env.exec(
        "EJ_SelectInstance(63); A_Admin.SetEncounterJournalDifficultyMask(63, 2^0 + 2^1 + 2^13 + 2^23)",
    )
    .expect("the dungeon instance and its supplied difficulty mask should be selected");
    let dungeon: (bool, bool, bool, bool, bool, String) = env
        .eval(DUNGEON_DIFFICULTY_QUERY)
        .expect("the original dungeon dropdown should use selected-map difficulties");
    assert_eq!(
        dungeon,
        (true, true, false, true, false, "1,2,24".to_string())
    );
}

fn assert_raid_difficulty_queries(env: &WowLuaEnv) {
    env.exec("EJ_SelectInstance(72); A_Admin.SetEncounterJournalDifficultyMask(72, 60)")
        .expect("the raid instance and its supplied difficulty mask should be selected");
    let raid: (bool, bool, bool, String) = env
        .eval(RAID_DIFFICULTY_QUERY)
        .expect("the original raid dropdown should use selected-map difficulties");
    assert_eq!(raid, (true, false, true, "3,4,5,6".to_string()));
}

fn assert_base_difficulty_queries(env: &WowLuaEnv) {
    let bases: (i32, i32, i32) = env
        .eval(BASE_DIFFICULTY_QUERY)
        .expect("base difficulty queries should accept difficulty IDs");
    assert_eq!(bases, (16, 17, 63));
}

prefork_full_ui_case! {
fn encounter_journal_difficulty_queries_use_selected_map(env: &WowLuaEnv) {
    assert_dungeon_difficulty_queries(env);
    assert_raid_difficulty_queries(env);
    assert_base_difficulty_queries(env);
}
}

const RAID_LOCK_BOSS_SETUP: &str = r#"
            local selected
            for instanceIndex = 1, 500 do
                local instanceID = EJ_GetInstanceByIndex(instanceIndex, true)
                if not instanceID then break end
                EJ_SelectInstance(instanceID)
                if C_EncounterJournal.InstanceHasDifficultyID(14) then
                    local encounters = {}
                    for bossIndex = 1, 100 do
                        local _, _, journalID = EJ_GetEncounterInfoByIndex(bossIndex, instanceID)
                        if not journalID then break end
                        local _, _, _, _, _, _, encounterID, mapID = EJ_GetEncounterInfo(journalID)
                        if encounterID and encounterID > 0 and mapID and mapID > 0 then
                            encounters[#encounters + 1] = {
                                journalID = journalID,
                                encounterID = encounterID,
                                mapID = mapID,
                            }
                        end
                    end
                    if #encounters >= 2 and encounters[1].mapID == encounters[2].mapID then
                        selected = {
                            instanceID = instanceID,
                            mapID = encounters[1].mapID,
                            first = encounters[1],
                        }
                        break
                    end
                end
            end
            assert(selected, "a raid with two mapped bosses and difficulty 14 should exist")
            A_Admin.SetEncounterJournalDifficultyMask(selected.instanceID,
                2^13 + 2^14 + 2^15 + 2^16)
            EJ_SetDifficulty(14)
            A_Admin.SetCompletedEncounters({
                { mapID=selected.mapID, difficultyID=14, completedEncounterIDs={} },
                { mapID=selected.mapID, difficultyID=15,
                  completedEncounterIDs={selected.first.encounterID} },
            })
            local boss = CreateFrame("Frame", "LtAc5eRaidLockBoss")
            boss.encounterID = selected.first.journalID
            boss.DefeatedOverlay = boss:CreateTexture(nil, "OVERLAY")
            EncounterJournalBossButton_UpdateDifficultyOverlay(boss)
            return selected.instanceID, selected.mapID,
                selected.first.journalID, selected.first.encounterID,
                boss.DefeatedOverlay:IsShown()
            "#;

fn read_raid_lock_boss_setup(
    env: &WowLuaEnv,
) -> (i32, i32, i32, i32, bool) {
    env.eval(RAID_LOCK_BOSS_SETUP)
        .expect("the original boss-overlay consumer should read the selected raid state")
}

fn assert_omitted_raid_lock_difficulty(env: &WowLuaEnv, map_id: i32, encounter_id: i32) {
    let omitted_difficulty: bool = env
        .eval(&format!(
            "return C_RaidLocks.IsEncounterComplete({map_id}, {encounter_id})"
        ))
        .expect("the no-difficulty query should read the first matching map row");
    assert_eq!(omitted_difficulty, false);
}

fn update_raid_lock_boss_overlay(env: &WowLuaEnv, map_id: i32, encounter_id: i32) {
    env.exec(&format!(
        r#"
        A_Admin.SetCompletedEncounters({{
            {{ mapID={map_id}, difficultyID=14, completedEncounterIDs={{{encounter_id}}} }},
            {{ mapID={map_id}, difficultyID=15, completedEncounterIDs={{}} }},
        }})
        EncounterJournalBossButton_UpdateDifficultyOverlay(LtAc5eRaidLockBoss)
        "#
    ))
    .expect("the original boss-overlay consumer should refresh from the new supplied rows");
}

fn assert_updated_raid_lock_boss_overlay(env: &WowLuaEnv, map_id: i32, encounter_id: i32) {
    let updated: bool = env
        .eval(&format!(
            r#"
            return LtAc5eRaidLockBoss.DefeatedOverlay:IsShown()
                and C_RaidLocks.IsEncounterComplete({map_id}, {encounter_id})
            "#
        ))
        .expect("the overlay and omitted-difficulty query should follow the first row");
    assert_eq!(updated, true);
}

prefork_full_ui_case! {
fn encounter_journal_boss_overlay_reads_supplied_raid_locks(env: &WowLuaEnv) {
    let (instance_id, map_id, journal_id, encounter_id, initial_shown) =
        read_raid_lock_boss_setup(env);
    assert_eq!(initial_shown, false);
    assert_omitted_raid_lock_difficulty(env, map_id, encounter_id);
    update_raid_lock_boss_overlay(env, map_id, encounter_id);
    assert_updated_raid_lock_boss_overlay(env, map_id, encounter_id);
    assert!(instance_id > 0 && journal_id > 0);
}
}
