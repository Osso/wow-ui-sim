#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

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
fn rust_raid_lock_queries_use_first_map_row_when_difficulty_is_omitted() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let (omitted, explicit_second, explicit_first, unknown): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local mapID, encounterID = 12345, 67890
            local firstDifficultyID, secondDifficultyID = 14, 15
            A_Admin.SetCompletedEncounters({
                { mapID=mapID, difficultyID=firstDifficultyID, completedEncounterIDs={} },
                { mapID=mapID, difficultyID=secondDifficultyID,
                  completedEncounterIDs={encounterID} },
            })
            return C_RaidLocks.IsEncounterComplete(mapID, encounterID),
                C_RaidLocks.IsEncounterComplete(mapID, encounterID, secondDifficultyID),
                C_RaidLocks.IsEncounterComplete(mapID, encounterID, firstDifficultyID),
                C_RaidLocks.IsEncounterComplete(mapID, encounterID + 1, secondDifficultyID)
            "#,
        )
        .expect("C_RaidLocks should evaluate supplied ordered raid-lock rows");

    assert!(!omitted);
    assert!(explicit_second);
    assert!(!explicit_first);
    assert!(!unknown);
}
