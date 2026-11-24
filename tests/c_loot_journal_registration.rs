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
fn rust_raid_lock_registration_uses_first_map_row_without_difficulty() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ));
    env.set_screen_mode(wow_ui_sim::screen::ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui];
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env.exec(
        "assert(EncounterJournal_LoadUI(), 'Encounter Journal should load on demand')",
    )
    .expect("load Encounter Journal through its normal bootstrap");

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
            EJ_SelectInstance(selected.instanceID)
            A_Admin.SetCompletedEncounters({
                { mapID=selected.mapID, difficultyID=14, completedEncounterIDs={} },
                { mapID=selected.mapID, difficultyID=DifficultyUtil.ID.PrimaryRaidHeroic,
                  completedEncounterIDs={selected.encounterID} },
            })
            EncounterJournal_OpenJournal(
                DifficultyUtil.ID.PrimaryRaidHeroic, selected.instanceID)
            assert(EncounterJournal:IsShown(), "the client path must open Encounter Journal")
            assert(EJ_GetDifficulty() == DifficultyUtil.ID.PrimaryRaidHeroic,
                "Encounter Journal must select the Heroic difficulty")
            local overlayShown
            EncounterJournal.encounter.info.BossesScrollBox:ForEachFrame(function(button)
                if button.encounterID == selected.journalID then
                    overlayShown = button.DefeatedOverlay:IsShown()
                end
            end)
            assert(overlayShown ~= nil, "Encounter Journal must create the selected boss button")
            return C_RaidLocks.IsEncounterComplete(selected.mapID, selected.encounterID),
                C_RaidLocks.IsEncounterComplete(selected.mapID, selected.encounterID,
                    DifficultyUtil.ID.PrimaryRaidHeroic),
                overlayShown
            "#,
        )
        .expect("the loaded Encounter Journal consumer should read supplied raid locks");

    assert!(!omitted);
    assert!(explicit);
    assert!(overlay_shown);
}
