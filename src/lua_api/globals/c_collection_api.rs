//! C_Collection namespaces for mounts, pets, toys, transmog, and heirlooms.
//!
//! Contains collection journal API functions for various game collectibles.

use mlua::{Lua, Result, Value};

/// Register collection-related C_* namespaces.
pub fn register_c_collection_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // C_PetJournal namespace - battle pet utilities
    let c_pet_journal = lua.create_table()?;
    c_pet_journal.set(
        "GetNumPets",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_pet_journal.set(
        "GetPetInfoByIndex",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_pet_journal.set(
        "GetPetInfoByPetID",
        lua.create_function(|_, _pet_id: String| Ok(Value::Nil))?,
    )?;
    c_pet_journal.set(
        "GetPetInfoBySpeciesID",
        lua.create_function(|_, _species_id: i32| Ok(Value::Nil))?,
    )?;
    c_pet_journal.set(
        "PetIsSummonable",
        lua.create_function(|_, _pet_id: String| Ok(false))?,
    )?;
    c_pet_journal.set(
        "GetNumCollectedInfo",
        lua.create_function(|_, _species_id: i32| Ok((0i32, 0i32)))?,
    )?;
    globals.set("C_PetJournal", c_pet_journal)?;

    // C_MountJournal namespace - mount collection
    let c_mount_journal = lua.create_table()?;
    c_mount_journal.set(
        "GetNumMounts",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mount_journal.set(
        "GetNumDisplayedMounts",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mount_journal.set(
        "GetMountInfoByID",
        lua.create_function(|_, _mount_id: i32| {
            // Returns: name, spellID, icon, isActive, isUsable, sourceType, isFavorite,
            // isFactionSpecific, faction, shouldHideOnChar, isCollected, mountID, ...
            Ok((
                Value::Nil, // name
                Value::Nil, // spellID
                Value::Nil, // icon
                false,      // isActive
                false,      // isUsable
                0i32,       // sourceType
                false,      // isFavorite
                false,      // isFactionSpecific
                Value::Nil, // faction
                false,      // shouldHideOnChar
                false,      // isCollected
                0i32,       // mountID
            ))
        })?,
    )?;
    c_mount_journal.set(
        "GetMountIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_mount_journal.set(
        "GetCollectedFilterSetting",
        lua.create_function(|_, _filter_index: i32| Ok(true))?,
    )?;
    c_mount_journal.set(
        "SetCollectedFilterSetting",
        lua.create_function(|_, (_filter_index, _is_checked): (i32, bool)| Ok(()))?,
    )?;
    c_mount_journal.set(
        "GetIsFavorite",
        lua.create_function(|_, _mount_index: i32| Ok((false, false)))?,
    )?;
    c_mount_journal.set(
        "SetIsFavorite",
        lua.create_function(|_, (_mount_index, _is_favorite): (i32, bool)| Ok(()))?,
    )?;
    c_mount_journal.set(
        "Summon",
        lua.create_function(|_, _mount_id: i32| Ok(()))?,
    )?;
    c_mount_journal.set(
        "Dismiss",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    globals.set("C_MountJournal", c_mount_journal)?;

    // C_ToyBox namespace - toy collection
    let c_toy_box = lua.create_table()?;
    c_toy_box.set(
        "GetToyInfo",
        lua.create_function(|_, _item_id: i32| {
            // Returns: itemID, toyName, icon, isFavorite, hasFanfare, itemQuality
            Ok((0i32, "", 0i32, false, false, 0i32))
        })?,
    )?;
    c_toy_box.set(
        "IsToyUsable",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    c_toy_box.set(
        "GetNumToys",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_toy_box.set(
        "GetToyFromIndex",
        lua.create_function(|_, _index: i32| Ok(0i32))?,
    )?;
    c_toy_box.set(
        "GetNumFilteredToys",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    globals.set("C_ToyBox", c_toy_box)?;

    // C_TransmogCollection namespace - transmog/appearance collection
    let c_transmog_collection = lua.create_table()?;
    c_transmog_collection.set(
        "GetAppearanceSources",
        lua.create_function(|lua, _appearance_id: i32| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetSourceInfo",
        lua.create_function(|lua, _source_id: i32| {
            // Returns sourceInfo table
            let info = lua.create_table()?;
            info.set("sourceID", 0)?;
            info.set("visualID", 0)?;
            info.set("categoryID", 0)?;
            info.set("itemID", 0)?;
            info.set("isCollected", false)?;
            Ok(info)
        })?,
    )?;
    c_transmog_collection.set(
        "PlayerHasTransmog",
        lua.create_function(|_, (_item_id, _appearance_mod): (i32, Option<i32>)| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "PlayerHasTransmogByItemInfo",
        lua.create_function(|_, _item_info: String| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "PlayerHasTransmogItemModifiedAppearance",
        lua.create_function(|_, _item_modified_appearance_id: i32| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "GetItemInfo",
        lua.create_function(|_, _item_modified_appearance_id: i32| Ok(Value::Nil))?,
    )?;
    c_transmog_collection.set(
        "GetAllAppearanceSources",
        lua.create_function(|lua, _visual_id: i32| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetIllusions",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetOutfits",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetNumMaxOutfits",
        lua.create_function(|_, ()| Ok(20i32))?,
    )?;
    c_transmog_collection.set(
        "GetOutfitInfo",
        lua.create_function(|_, _outfit_id: i32| {
            Ok((Value::Nil, Value::Nil)) // name, icon
        })?,
    )?;
    c_transmog_collection.set(
        "GetAppearanceCameraID",
        lua.create_function(|_, _appearance_id: i32| Ok(0i32))?,
    )?;
    c_transmog_collection.set(
        "GetCategoryAppearances",
        lua.create_function(|lua, (_category, _location): (i32, Value)| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "PlayerKnowsSource",
        lua.create_function(|_, _source_id: i32| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "IsAppearanceHiddenVisual",
        lua.create_function(|_, _appearance_id: i32| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "IsSourceTypeFilterChecked",
        lua.create_function(|_, _filter: i32| Ok(true))?,
    )?;
    c_transmog_collection.set(
        "GetShowMissingSourceInItemTooltips",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    globals.set("C_TransmogCollection", c_transmog_collection)?;

    // C_Transmog namespace - transmogrification API
    let c_transmog = lua.create_table()?;
    c_transmog.set(
        "GetAllSetAppearancesByID",
        lua.create_function(|lua, _set_id: i32| {
            // Returns array of appearance info for a transmog set
            lua.create_table()
        })?,
    )?;
    c_transmog.set(
        "GetAppliedSourceID",
        lua.create_function(|_, _slot: i32| Ok(Value::Nil))?,
    )?;
    c_transmog.set(
        "GetSlotInfo",
        lua.create_function(|_, _slot: i32| {
            Ok((false, false, false, false, false, Value::Nil))
        })?,
    )?;
    globals.set("C_Transmog", c_transmog)?;

    // TransmogUtil - utility functions for transmog system
    let transmog_util = lua.create_table()?;
    transmog_util.set(
        "GetTransmogLocation",
        lua.create_function(|lua, (slot, transmog_type, modification): (String, i32, i32)| {
            // Return a transmog location table
            let location = lua.create_table()?;
            location.set("slotName", slot)?;
            location.set("transmogType", transmog_type)?;
            location.set("modification", modification)?;
            Ok(location)
        })?,
    )?;
    transmog_util.set(
        "CreateTransmogLocation",
        lua.create_function(|lua, (slot_id, transmog_type, modification): (i32, i32, i32)| {
            let location = lua.create_table()?;
            location.set("slotID", slot_id)?;
            location.set("transmogType", transmog_type)?;
            location.set("modification", modification)?;
            Ok(location)
        })?,
    )?;
    transmog_util.set(
        "GetBestItemModifiedAppearanceID",
        lua.create_function(|_, _item_loc: mlua::Value| Ok(Value::Nil))?,
    )?;
    globals.set("TransmogUtil", transmog_util)?;

    // C_Heirloom namespace - heirloom collection
    let c_heirloom = lua.create_table()?;
    c_heirloom.set(
        "GetHeirloomInfo",
        lua.create_function(|_, _item_id: i32| {
            // Returns: name, itemEquipLoc, isPvP, itemTexture, upgradeLevel, source, searchFiltered, effectiveLevel, minLevel, maxLevel
            Ok((Value::Nil, Value::Nil, false, 0i32, 0i32, 0i32, false, 0i32, 0i32, 0i32))
        })?,
    )?;
    c_heirloom.set(
        "GetHeirloomMaxUpgradeLevel",
        lua.create_function(|_, _item_id: i32| Ok(0i32))?,
    )?;
    c_heirloom.set(
        "GetNumHeirlooms",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_heirloom.set(
        "GetNumKnownHeirlooms",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_heirloom.set(
        "PlayerHasHeirloom",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    c_heirloom.set(
        "GetHeirloomLink",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;
    c_heirloom.set(
        "CanHeirloomUpgradeFromPending",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    c_heirloom.set(
        "GetClassAndSpecFilters",
        lua.create_function(|_, ()| Ok((0i32, 0i32)))?,
    )?;
    globals.set("C_Heirloom", c_heirloom)?;

    // C_TransmogSets namespace - transmog set collection
    let c_transmog_sets = lua.create_table()?;
    c_transmog_sets.set(
        "GetBaseSetID",
        lua.create_function(|_, _set_id: i32| Ok(0i32))?,
    )?;
    c_transmog_sets.set(
        "GetVariantSets",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "GetSetInfo",
        lua.create_function(|lua, _set_id: i32| {
            let info = lua.create_table()?;
            info.set("setID", 0)?;
            info.set("name", "")?;
            info.set("description", "")?;
            info.set("label", "")?;
            info.set("expansionID", 0)?;
            info.set("collected", false)?;
            Ok(info)
        })?,
    )?;
    c_transmog_sets.set(
        "GetSetPrimaryAppearances",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "GetAllSets",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "GetUsableSets",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "IsBaseSetCollected",
        lua.create_function(|_, _set_id: i32| Ok(false))?,
    )?;
    c_transmog_sets.set(
        "GetSourcesForSlot",
        lua.create_function(|lua, (_set_id, _slot): (i32, i32)| lua.create_table())?,
    )?;
    globals.set("C_TransmogSets", c_transmog_sets)?;

    Ok(())
}
