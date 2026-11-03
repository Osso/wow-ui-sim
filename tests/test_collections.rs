use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap()
}

#[test]
fn mount_journal_num_mounts() {
    let env = env();
    let count: i32 = env.eval("return C_MountJournal.GetNumMounts()").unwrap();
    assert_eq!(count, 10, "Should have 10 default mounts");
}

#[test]
fn mount_journal_num_displayed_mounts() {
    let env = env();
    let count: i32 = env
        .eval("return C_MountJournal.GetNumDisplayedMounts()")
        .unwrap();
    assert_eq!(count, 10, "Displayed mounts should equal total mounts");
}

#[test]
fn mount_journal_get_displayed_mount_info() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name, spellID, icon, isActive, isUsable, sourceType,
                  isFavorite, isFactionSpecific, faction, shouldHideOnChar,
                  isCollected, mountID = C_MountJournal.GetDisplayedMountInfo(2)
            if name ~= "Swift Palomino" then return "name=" .. tostring(name) end
            if mountID ~= 18 then return "mountID=" .. tostring(mountID) end
            if isCollected ~= true then return "collected=" .. tostring(isCollected) end
            if isUsable ~= true then return "usable=" .. tostring(isUsable) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetDisplayedMountInfo: {result}");
}

#[test]
fn mount_journal_search_filters_displayed_mounts() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            C_MountJournal.SetSearch("ashes")

            local count = C_MountJournal.GetNumDisplayedMounts()
            if count ~= 1 then return "count=" .. tostring(count) end

            local mountID = C_MountJournal.GetDisplayedMountID(1)
            if mountID ~= 107 then return "displayed_id=" .. tostring(mountID) end

            local name, _, _, _, _, _, _, _, _, _, _, infoMountID = C_MountJournal.GetDisplayedMountInfo(1)
            if name ~= "Ashes of Al'ar" then return "name=" .. tostring(name) end
            if infoMountID ~= 107 then return "info_id=" .. tostring(infoMountID) end

            C_MountJournal.SetSearch("")
            if C_MountJournal.GetNumDisplayedMounts() ~= C_MountJournal.GetNumMounts() then
                return "clear_count=" .. tostring(C_MountJournal.GetNumDisplayedMounts())
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "SetSearch should filter displayed mounts: {result}"
    );
}

#[test]
fn mount_journal_displayed_info_uncollected() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- Mount 10 is Mighty Caravan Brutosaur (not collected)
            local name, _, _, _, _, _, _, _, _, _, isCollected = C_MountJournal.GetDisplayedMountInfo(10)
            if name ~= "Mighty Caravan Brutosaur" then return "name=" .. tostring(name) end
            if isCollected ~= false then return "collected=" .. tostring(isCollected) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "Uncollected mount: {result}");
}

#[test]
fn mount_journal_displayed_info_invalid_index() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local r = C_MountJournal.GetDisplayedMountInfo(99)
            return r == nil and "nil" or "not_nil"
            "#,
        )
        .unwrap();
    assert_eq!(result, "nil", "Invalid index should return nil");
}

#[test]
fn mount_journal_get_mount_info_by_id() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name, spellID, icon, _, isUsable, _, _, _, _, _, isCollected, mountID
                = C_MountJournal.GetMountInfoByID(107)
            if name ~= "Ashes of Al'ar" then return "name=" .. tostring(name) end
            if mountID ~= 107 then return "id=" .. tostring(mountID) end
            if isCollected ~= true then return "collected=" .. tostring(isCollected) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetMountInfoByID: {result}");
}

#[test]
fn mount_journal_get_mount_info_by_id_invalid() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local r = C_MountJournal.GetMountInfoByID(99999)
            return r == nil and "nil" or "not_nil"
            "#,
        )
        .unwrap();
    assert_eq!(result, "nil");
}

#[test]
fn pet_journal_num_pets() {
    let env = env();
    let (total, owned): (i32, i32) = env.eval("return C_PetJournal.GetNumPets()").unwrap();
    assert_eq!(total, 10, "Should have 10 default pets");
    assert_eq!(owned, 9, "9 collected (Pocopoc is not collected)");
}

#[test]
fn pet_journal_num_collected_info() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local collected, total = C_PetJournal.GetNumCollectedInfo(0)
            return collected .. "," .. total
            "#,
        )
        .unwrap();
    assert_eq!(result, "9,10", "9 collected out of 10 total");
}

#[test]
fn pet_journal_get_pet_info_by_index() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local petID, speciesID, isOwned, customName, level, isFavorite, isRevoked, name, icon, petType
                = C_PetJournal.GetPetInfoByIndex(1)
            if type(petID) ~= "string" then return "petID_type=" .. type(petID) end
            if name ~= "Mechanical Squirrel" then return "name=" .. tostring(name) end
            if speciesID ~= 39 then return "species=" .. tostring(speciesID) end
            if isOwned ~= true then return "owned=" .. tostring(isOwned) end
            if level ~= 25 then return "level=" .. tostring(level) end
            if type(level) ~= "number" then return "level_type=" .. type(level) end
            if customName ~= nil then return "customName=" .. tostring(customName) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetPetInfoByIndex: {result}");
}

#[test]
fn pet_journal_get_pet_info_by_species_id() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name, icon, petType, creatureID, sourceText, description, isWild, canBattle, tradable, unique, obtainable, displayID
                = C_PetJournal.GetPetInfoBySpeciesID(254)
            if name ~= "Lil' Ragnaros" then return "name=" .. tostring(name) end
            if type(sourceText) ~= "string" then return "source_type=" .. type(sourceText) end
            if type(description) ~= "string" then return "desc_type=" .. type(description) end
            if type(displayID) ~= "number" then return "displayID_type=" .. type(displayID) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetPetInfoBySpeciesID: {result}");
}

#[test]
fn pet_journal_get_pet_info_by_pet_id_has_strings_for_card_fields() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local petID = C_PetJournal.GetPetInfoByIndex(1)
            local speciesID, customName, level, xp, maxXp, displayID, isFavorite, name, icon, petType, creatureID, sourceText, description, isWild, canBattle, tradable, unique
                = C_PetJournal.GetPetInfoByPetID(petID)
            if type(name) ~= "string" then return "name_type=" .. type(name) end
            if type(sourceText) ~= "string" then return "source_type=" .. type(sourceText) end
            if type(description) ~= "string" then return "desc_type=" .. type(description) end
            if maxXp == nil or maxXp <= 0 then return "maxXp=" .. tostring(maxXp) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "GetPetInfoByPetID card fields should be non-nil strings: {result}"
    );
}

#[test]
fn pet_journal_get_pet_model_scene_info_returns_numeric_scene_ids() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local petID, speciesID = C_PetJournal.GetPetInfoByIndex(1)
            local cardModelSceneID, loadoutModelSceneID = C_PetJournal.GetPetModelSceneInfoBySpeciesID(speciesID)
            if type(cardModelSceneID) ~= "number" then return "card_type=" .. type(cardModelSceneID) end
            if type(loadoutModelSceneID) ~= "number" then return "loadout_type=" .. type(loadoutModelSceneID) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "PetJournal model scene info must be numeric for ModelScene:TransitionToModelSceneID: {result}"
    );
}

#[test]
fn pet_journal_get_pet_stats_reports_collected_pets_alive() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local petID = C_PetJournal.GetPetInfoByIndex(1)
            local health, maxHealth, attack, speed, rarity = C_PetJournal.GetPetStats(petID)
            if health == nil then return "health=nil" end
            if maxHealth == nil then return "maxHealth=nil" end
            if health <= 0 then return "health=" .. tostring(health) end
            if maxHealth <= 0 then return "maxHealth=" .. tostring(maxHealth) end
            if health > maxHealth then return "health_gt_max=" .. tostring(health) .. "," .. tostring(maxHealth) end
            if attack <= 0 then return "attack=" .. tostring(attack) end
            if speed <= 0 then return "speed=" .. tostring(speed) end
            if rarity ~= 3 then return "rarity=" .. tostring(rarity) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Collected pets should not be reported as dead: {result}"
    );
}

#[test]
fn pet_journal_get_pet_info_invalid() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local r = C_PetJournal.GetPetInfoByIndex(99)
            return r == nil and "nil" or "not_nil"
            "#,
        )
        .unwrap();
    assert_eq!(result, "nil");
}

#[test]
fn mount_journal_get_mount_info_extra_by_id() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local _, _, _, _, mountTypeID = C_MountJournal.GetMountInfoExtraByID(107)
            -- Ashes of Al'ar is flying (mount_type 248)
            if mountTypeID ~= 248 then return "type=" .. tostring(mountTypeID) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetMountInfoExtraByID: {result}");
}

#[test]
fn toy_box_num_total_displayed() {
    let env = env();
    let count: i32 = env
        .eval("return C_ToyBox.GetNumTotalDisplayedToys()")
        .unwrap();
    assert_eq!(count, 10);
}

#[test]
fn toy_box_num_learned_displayed() {
    let env = env();
    let count: i32 = env
        .eval("return C_ToyBox.GetNumLearnedDisplayedToys()")
        .unwrap();
    assert_eq!(count, 9, "9 collected out of 10");
}

#[test]
fn toy_box_get_toy_from_index() {
    let env = env();
    let id: i32 = env.eval("return C_ToyBox.GetToyFromIndex(1)").unwrap();
    assert_eq!(id, 166779, "First toy should be Transmorpher Beacon");
}

#[test]
fn toy_box_get_toy_info() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local itemID, name, icon = C_ToyBox.GetToyInfo(13379)
            if name ~= "Piccolo of the Flaming Fire" then return "name=" .. tostring(name) end
            if itemID ~= 13379 then return "id=" .. tostring(itemID) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetToyInfo: {result}");
}

// --- Admin collect/uncollect ---

#[test]
fn admin_collect_uncollect_mount() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- Brutosaur (mount 1039) is not collected by default
            local _, _, _, _, _, _, _, _, _, _, collected = C_MountJournal.GetMountInfoByID(1039)
            if collected then return "already_collected" end
            A_Admin.CollectMount(1039)
            local _, _, _, _, _, _, _, _, _, _, collected2 = C_MountJournal.GetMountInfoByID(1039)
            if not collected2 then return "not_collected_after" end
            A_Admin.UncollectMount(1039)
            local _, _, _, _, _, _, _, _, _, _, collected3 = C_MountJournal.GetMountInfoByID(1039)
            if collected3 then return "still_collected" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "CollectMount/UncollectMount: {result}");
}

#[test]
fn admin_collect_uncollect_pet() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- Pocopoc (species 2403) is not collected by default
            local _, owned = C_PetJournal.GetNumPets()
            if owned ~= 9 then return "initial_owned=" .. tostring(owned) end
            A_Admin.CollectPet(2403)
            local _, owned2 = C_PetJournal.GetNumPets()
            if owned2 ~= 10 then return "after_collect=" .. tostring(owned2) end
            A_Admin.UncollectPet(2403)
            local _, owned3 = C_PetJournal.GetNumPets()
            if owned3 ~= 9 then return "after_uncollect=" .. tostring(owned3) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "CollectPet/UncollectPet: {result}");
}

#[test]
fn admin_collect_uncollect_toy() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- Earpieces (187421) is not collected by default
            A_Admin.CollectToy(187421)
            local learned = C_ToyBox.GetNumLearnedDisplayedToys()
            if learned ~= 10 then return "learned=" .. tostring(learned) end
            A_Admin.UncollectToy(187421)
            local learned2 = C_ToyBox.GetNumLearnedDisplayedToys()
            if learned2 ~= 9 then return "learned2=" .. tostring(learned2) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "CollectToy/UncollectToy: {result}");
}
