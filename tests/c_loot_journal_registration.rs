#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[cfg(feature = "client-wowforever")]
use std::path::PathBuf;

#[cfg(feature = "client-wowforever")]
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn rust_item_set_registration_survives_bootstrap() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        A_Admin.SetLootJournalItemSets({
            { setID=501, name="Registration Alpha", itemLevel=620, classID=1, specID=71,
              items={{ itemID=41001, icon=91001, slot=1, invType=13 }} },
        })
        "#,
    )
    .expect("host-supplied item-set data should be accepted");

    let result: String = env
        .eval(
            r#"
            local sets = C_LootJournal.GetItemSets(1, 71)
            local items = C_LootJournal.GetItemSetItems(501)
            return #sets .. ":" .. sets[1].name .. ":" .. #items .. ":" .. items[1].itemID
            "#,
        )
        .expect("C_LootJournal should return the modeled item-set data");

    assert_eq!(result, "1:Registration Alpha:1:41001");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn rust_raid_lock_registration_uses_first_map_row_without_difficulty() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    load_addon(&env.loader_env(), &encounter_journal_toc())
        .expect("Forever Encounter Journal should load from its selected TOC");
    env.apply_post_load_workarounds();

    let (omitted, explicit, overlay_shown): (bool, bool, bool) = env
        .eval(
            r#"
            local selected
            for instanceIndex = 1, 500 do
                local instanceID = EJ_GetInstanceByIndex(instanceIndex, true)
                if not instanceID then break end
                for bossIndex = 1, 100 do
                    local _, _, journalID = EJ_GetEncounterInfoByIndex(bossIndex, instanceID)
                    if not journalID then break end
                    local _, _, _, _, _, _, encounterID, mapID = EJ_GetEncounterInfo(journalID)
                    if encounterID and encounterID > 0 and mapID and mapID > 0 then
                        selected = { journalID=journalID, encounterID=encounterID, mapID=mapID }
                        break
                    end
                end
                if selected then break end
            end
            assert(selected, "Encounter Journal must contain an encounter with map data")
            A_Admin.SetCompletedEncounters({
                { mapID=selected.mapID, difficultyID=14, completedEncounterIDs={} },
                { mapID=selected.mapID, difficultyID=DifficultyUtil.ID.PrimaryRaidHeroic,
                  completedEncounterIDs={selected.encounterID} },
            })
            EJ_SetDifficulty(DifficultyUtil.ID.PrimaryRaidHeroic)
            local button = CreateFrame("Button")
            button.encounterID = selected.journalID
            button.DefeatedOverlay = CreateFrame("Frame", nil, button)
            EncounterJournalBossButton_UpdateDifficultyOverlay(button)
            return C_RaidLocks.IsEncounterComplete(selected.mapID, selected.encounterID),
                C_RaidLocks.IsEncounterComplete(selected.mapID, selected.encounterID,
                    DifficultyUtil.ID.PrimaryRaidHeroic),
                button.DefeatedOverlay:IsShown()
            "#,
        )
        .expect("the loaded Encounter Journal consumer should read supplied raid locks");

    assert!(!omitted);
    assert!(explicit);
    assert!(overlay_shown);
}

#[cfg(feature = "client-wowforever")]
fn encounter_journal_toc() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
        .join("Blizzard_EncounterJournal/Blizzard_EncounterJournal.toc")
}
