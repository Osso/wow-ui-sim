#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn encounter_journal_toc() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
    .join(format!(
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
    ))
}

prefork_full_ui_case! {
fn encounter_journal_renders_supplied_item_sets_and_items(env: &WowLuaEnv) {
    env.exec("A_Admin.SetPlayerClass(2); A_Admin.SetSpec(3)")
        .expect("the player scenario should select the Paladin specialization");
    load_addon(&env.loader_env(), &encounter_journal_toc())
        .expect("selected Encounter Journal TOC should load");
    env.apply_post_load_workarounds();

    let rendered: String = env
        .eval(RENDER_SUPPLIED_ITEM_SETS)
        .expect("Encounter Journal loot item-set consumer should render supplied values");

    assert_eq!(
        rendered,
        "Supplied Alpha:41001:91001|Supplied Beta:41002:91002"
    );
}
}

const RENDER_SUPPLIED_ITEM_SETS: &str = r#"
            A_Admin.SetLootJournalItemSets({
                { setID=501, name="Supplied Alpha", itemLevel=620, classID=2, specID=70,
                  items={{ itemID=41001, icon=91001, slot=1, invType=13 }} },
                { setID=502, name="Supplied Beta", itemLevel=630, classID=2, specID=70,
                  items={{ itemID=41002, icon=91002, slot=2, invType=14 }} },
                { setID=503, name="Filtered Out", itemLevel=640, classID=3, specID=73,
                  items={{ itemID=41003, icon=91003, slot=3, invType=15 }} },
            })
            EncounterJournal_OpenJournal()
            EncounterJournal.LootJournalTab:Click()
            assert(EncounterJournal:IsShown(), "the client path must open Encounter Journal")
            assert(EncounterJournal.LootJournalItems:IsShown(), "the loot tab must be selected")
            local list = EncounterJournal.LootJournalItems.ItemSetsFrame
            assert(list:IsShown(), "the item-sets view must be active")
            local provider = list.ScrollBox:GetDataProvider()
            assert(provider:GetSize() == 2, "the loot-journal consumer must filter to two sets")
            local rendered = {}
            list.ScrollBox:ForEachFrame(function(button)
                local item = button.ItemButtons[1]
                rendered[#rendered + 1] = table.concat({
                    button.SetName:GetText(), item.itemID, item.Icon:GetTexture()
                }, ":")
            end)
            table.sort(rendered)
            return table.concat(rendered, "|")
            "#;

/// Supplied raid-lock fixture: one raid instance and two of its mapped bosses.
struct RaidLockBosses {
    instance_id: i32,
    map_id: i32,
    journal_a: i32,
    encounter_a: i32,
    journal_b: i32,
    encounter_b: i32,
}

prefork_full_ui_case! {
fn encounter_journal_boss_list_uses_supplied_raid_locks(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &encounter_journal_toc())
        .expect("selected Encounter Journal TOC should load");
    env.apply_post_load_workarounds();
    let bosses = open_raid_with_first_boss_complete(env);
    assert_initial_raid_lock_overlays(env, &bosses);
    complete_second_boss_and_request_raid_info(env, &bosses);
    env.process_timers()
        .expect("the queued raid-info reply should dispatch on the next timer tick");
    assert_second_boss_overlay_shown(env, &bosses);
}
}

fn open_raid_with_first_boss_complete(env: &WowLuaEnv) -> RaidLockBosses {
    let (instance_id, map_id, journal_a, encounter_a, journal_b, encounter_b): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
    ) = env
        .eval(SELECT_RAID_AND_OPEN_JOURNAL)
        .expect("a raid instance with two mapped Encounter Journal bosses should exist");
    RaidLockBosses {
        instance_id,
        map_id,
        journal_a,
        encounter_a,
        journal_b,
        encounter_b,
    }
}

const SELECT_RAID_AND_OPEN_JOURNAL: &str = r#"
            local selected
            for instanceIndex = 1, 500 do
                local instanceID = EJ_GetInstanceByIndex(instanceIndex, true)
                if not instanceID then break end
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
                if #encounters >= 2
                    and encounters[1].mapID == encounters[2].mapID then
                    selected = {
                        instanceID = instanceID,
                        mapID = encounters[1].mapID,
                        first = encounters[1],
                        second = encounters[2],
                    }
                    break
                end
            end
            assert(selected, "Encounter Journal must contain two raid bosses with map data")
            EJ_SelectInstance(selected.instanceID)
            assert(EJ_InstanceIsRaid(), "selected Encounter Journal instance must be a raid")
            A_Admin.SetCompletedEncounters({
                { mapID=selected.mapID, difficultyID=14,
                  completedEncounterIDs={selected.second.encounterID} },
                { mapID=selected.mapID, difficultyID=15,
                  completedEncounterIDs={selected.first.encounterID} },
            })
            EncounterJournal_OpenJournal(15, selected.instanceID)
            assert(EncounterJournal:IsShown(), "the client path must open Encounter Journal")
            return selected.instanceID, selected.mapID,
                selected.first.journalID, selected.first.encounterID,
                selected.second.journalID, selected.second.encounterID
            "#;

fn assert_initial_raid_lock_overlays(env: &WowLuaEnv, b: &RaidLockBosses) {
    let RaidLockBosses {
        instance_id,
        map_id,
        journal_a,
        encounter_a,
        journal_b,
        encounter_b,
    } = b;
    let initial: (bool, bool, bool, bool, bool, bool) = env
        .eval(&format!(
            r#"
            local overlays = {{}}
            EncounterJournal.encounter.info.BossesScrollBox:ForEachFrame(function(button)
                if button.encounterID then
                    overlays[button.encounterID] = button.DefeatedOverlay:IsShown()
                end
            end)
            return EncounterJournal.instanceID == {instance_id},
                overlays[{journal_a}] == true,
                overlays[{journal_b}] == false,
                C_RaidLocks.IsEncounterComplete({map_id}, {encounter_a}, 14),
                C_RaidLocks.IsEncounterComplete({map_id}, {encounter_a}),
                C_RaidLocks.IsEncounterComplete({map_id}, {encounter_b}, 14)
            "#
        ))
        .expect("Encounter Journal boss overlays should read supplied raid-lock state");
    assert_eq!(initial, (true, true, true, false, true, false));
}

fn complete_second_boss_and_request_raid_info(env: &WowLuaEnv, b: &RaidLockBosses) {
    let RaidLockBosses {
        map_id,
        encounter_a,
        journal_b,
        encounter_b,
        ..
    } = b;
    env.exec(&format!(
        r#"
        A_Admin.SetCompletedEncounters({{
            {{ mapID={map_id}, difficultyID=14, completedEncounterIDs={{{encounter_b}}} }},
            {{ mapID={map_id}, difficultyID=15,
               completedEncounterIDs={{{encounter_a}, {encounter_b}}} }},
        }})
        RequestRaidInfo()
        local overlays = {{}}
        EncounterJournal.encounter.info.BossesScrollBox:ForEachFrame(function(button)
            if button.encounterID then
                overlays[button.encounterID] = button.DefeatedOverlay:IsShown()
            end
        end)
        assert(overlays[{journal_b}] == false,
            "RequestRaidInfo must not publish UPDATE_INSTANCE_INFO synchronously")
        "#
    ))
    .expect("the second supplied completion should wait for the raid-info reply tick");
}

fn assert_second_boss_overlay_shown(env: &WowLuaEnv, b: &RaidLockBosses) {
    let journal_b = b.journal_b;
    let updated: bool = env
        .eval(&format!(
            r#"
            local shown
            EncounterJournal.encounter.info.BossesScrollBox:ForEachFrame(function(button)
                if button.encounterID == {journal_b} then
                    shown = button.DefeatedOverlay:IsShown()
                end
            end)
            return shown == true
            "#
        ))
        .expect("UPDATE_INSTANCE_INFO should refresh the second boss overlay");
    assert!(
        updated,
        "the second boss overlay should show after the reply tick"
    );
}
