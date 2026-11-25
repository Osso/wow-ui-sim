#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::Path;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn encounter_journal_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("simulator environment should start");
    let addon_dir = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let toc = addon_dir.join(format!(
        "Blizzard_EncounterJournal/{}",
        match wow_ui_sim::client_profile::ACTIVE {
            wow_ui_sim::client_profile::ClientProfile::Retail => {
                "Blizzard_EncounterJournal_Mainline.toc"
            }
            wow_ui_sim::client_profile::ClientProfile::WowForever => {
                "Blizzard_EncounterJournal.toc"
            }
            _ => unreachable!("this test is limited to Retail and WoW Forever"),
        }
    ));
    load_addon(&env.loader_env(), &toc).expect("Encounter Journal TOC should load");
    env.apply_post_load_workarounds();
    env
}

#[test]
fn encounter_journal_difficulty_queries_use_selected_map() {
    let env = encounter_journal_env();
    env.exec(
        "EJ_SelectInstance(63); A_Admin.SetEncounterJournalDifficultyMask(63, 2^0 + 2^1 + 2^13 + 2^23)",
    )
    .expect("the dungeon instance and its supplied difficulty mask should be selected");

    let dungeon: (bool, bool, bool, bool, bool, String) = env
        .eval(
            r#"
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
            "#,
        )
        .expect("the original dungeon dropdown should use selected-map difficulties");
    assert_eq!(
        dungeon,
        (true, true, false, true, false, "1,2,24".to_string())
    );

    env.exec("EJ_SelectInstance(72); A_Admin.SetEncounterJournalDifficultyMask(72, 60)")
        .expect("the raid instance and its supplied difficulty mask should be selected");
    let raid: (bool, bool, bool, String) = env
        .eval(
            r#"
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
            "#,
        )
        .expect("the original raid dropdown should use selected-map difficulties");
    assert_eq!(raid, (true, false, true, "3,4,5,6".to_string()));

    let bases: (i32, i32, i32) = env
        .eval(
            r#"
            return C_EncounterJournal.GetBaseDifficultyID(233),
                C_EncounterJournal.GetBaseDifficultyID(250),
                C_EncounterJournal.GetBaseDifficultyID(63)
            "#,
        )
        .expect("base difficulty queries should accept difficulty IDs");
    assert_eq!(bases, (16, 17, 63));
}

#[test]
fn encounter_journal_boss_overlay_reads_supplied_raid_locks() {
    let env = encounter_journal_env();
    let (instance_id, map_id, journal_id, encounter_id): (i32, i32, i32, i32) = env
        .eval(
            r#"
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
            A_Admin.SetCompletedEncounters({
                { mapID=selected.mapID, difficultyID=14, completedEncounterIDs={} },
                { mapID=selected.mapID, difficultyID=15,
                  completedEncounterIDs={selected.first.encounterID} },
            })
            EncounterJournal_OpenJournal(14, selected.instanceID)
            assert(EncounterJournal:IsShown(), "the original Encounter Journal should open")
            return selected.instanceID, selected.mapID,
                selected.first.journalID, selected.first.encounterID
            "#,
        )
        .expect("a raid encounter should render from its selected map");

    let initial: (bool, bool) = env
        .eval(&format!(
            r#"
            local shown
            EncounterJournal.encounter.info.BossesScrollBox:ForEachFrame(function(button)
                if button.encounterID == {journal_id} then
                    shown = button.DefeatedOverlay:IsShown()
                end
            end)
            return shown == false,
                C_RaidLocks.IsEncounterComplete({map_id}, {encounter_id}) == false
            "#
        ))
        .expect("the overlay and omitted-difficulty query should use the first matching row");
    assert_eq!(initial, (true, true));

    env.exec(&format!(
        r#"
        A_Admin.SetCompletedEncounters({{
            {{ mapID={map_id}, difficultyID=14, completedEncounterIDs={{{encounter_id}}} }},
            {{ mapID={map_id}, difficultyID=15, completedEncounterIDs={{}} }},
        }})
        RequestRaidInfo()
        "#
    ))
    .expect("the supplied raid-lock update should schedule UPDATE_INSTANCE_INFO");
    env.process_timers()
        .expect("the queued raid-info update should dispatch");

    let updated: bool = env
        .eval(&format!(
            r#"
            local shown
            EncounterJournal.encounter.info.BossesScrollBox:ForEachFrame(function(button)
                if button.encounterID == {journal_id} then
                    shown = button.DefeatedOverlay:IsShown()
                end
            end)
            return shown == true
            "#
        ))
        .expect("the original boss overlay should reflect the supplied completion row");
    assert!(updated);
    assert!(instance_id > 0);
}
