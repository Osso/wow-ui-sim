//! C_Collection namespaces for mounts, pets, toys, transmog, and heirlooms.
//!
//! Contains collection journal API functions for various game collectibles.

use mlua::{Lua, Result, Value};

/// Register collection-related C_* namespaces.
pub fn register_c_collection_api(lua: &Lua) -> Result<()> {
    register_pet_journal(lua)?;
    register_mount_journal(lua)?;
    register_toy_box(lua)?;
    register_transmog_collection(lua)?;
    register_transmog(lua)?;
    register_transmog_util(lua)?;
    register_heirloom(lua)?;
    register_transmog_sets(lua)?;
    Ok(())
}

/// C_PetJournal namespace - battle pet utilities.
fn register_pet_journal(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumPets", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set(
        "GetPetInfoByIndex",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetPetInfoByPetID",
        lua.create_function(|_, _pet_id: String| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetPetInfoBySpeciesID",
        lua.create_function(|_, _species_id: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "PetIsSummonable",
        lua.create_function(|_, _pet_id: String| Ok(false))?,
    )?;
    t.set(
        "GetNumCollectedInfo",
        lua.create_function(|_, _species_id: i32| Ok((0i32, 0i32)))?,
    )?;
    lua.globals().set("C_PetJournal", t)?;
    Ok(())
}

/// C_MountJournal namespace - mount collection.
fn register_mount_journal(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumMounts", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set(
        "GetNumDisplayedMounts",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    t.set(
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
    t.set(
        "GetMountIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "GetCollectedFilterSetting",
        lua.create_function(|_, _filter_index: i32| Ok(true))?,
    )?;
    t.set(
        "SetCollectedFilterSetting",
        lua.create_function(|_, (_filter_index, _is_checked): (i32, bool)| Ok(()))?,
    )?;
    t.set(
        "GetIsFavorite",
        lua.create_function(|_, _mount_index: i32| Ok((false, false)))?,
    )?;
    t.set(
        "SetIsFavorite",
        lua.create_function(|_, (_mount_index, _is_favorite): (i32, bool)| Ok(()))?,
    )?;
    t.set("Summon", lua.create_function(|_, _mount_id: i32| Ok(()))?)?;
    t.set("Dismiss", lua.create_function(|_, ()| Ok(()))?)?;
    lua.globals().set("C_MountJournal", t)?;
    Ok(())
}

/// C_ToyBox namespace - toy collection.
fn register_toy_box(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetToyInfo",
        lua.create_function(|_, _item_id: i32| {
            // Returns: itemID, toyName, icon, isFavorite, hasFanfare, itemQuality
            Ok((0i32, "", 0i32, false, false, 0i32))
        })?,
    )?;
    t.set(
        "IsToyUsable",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    t.set("GetNumToys", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set(
        "GetToyFromIndex",
        lua.create_function(|_, _index: i32| Ok(0i32))?,
    )?;
    t.set(
        "GetNumFilteredToys",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    lua.globals().set("C_ToyBox", t)?;
    Ok(())
}

/// C_TransmogCollection namespace - transmog/appearance collection.
fn register_transmog_collection(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetAppearanceSources",
        lua.create_function(|lua, _appearance_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetSourceInfo",
        lua.create_function(|lua, _source_id: i32| {
            let info = lua.create_table()?;
            info.set("sourceID", 0)?;
            info.set("visualID", 0)?;
            info.set("categoryID", 0)?;
            info.set("itemID", 0)?;
            info.set("isCollected", false)?;
            Ok(info)
        })?,
    )?;
    t.set(
        "PlayerHasTransmog",
        lua.create_function(|_, (_item_id, _appearance_mod): (i32, Option<i32>)| Ok(false))?,
    )?;
    t.set(
        "PlayerHasTransmogByItemInfo",
        lua.create_function(|_, _item_info: String| Ok(false))?,
    )?;
    t.set(
        "PlayerHasTransmogItemModifiedAppearance",
        lua.create_function(|_, _item_modified_appearance_id: i32| Ok(false))?,
    )?;
    t.set(
        "GetItemInfo",
        lua.create_function(|_, _item_modified_appearance_id: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetAllAppearanceSources",
        lua.create_function(|lua, _visual_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetIllusions",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "GetOutfits",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "GetNumMaxOutfits",
        lua.create_function(|_, ()| Ok(20i32))?,
    )?;
    t.set(
        "GetOutfitInfo",
        lua.create_function(|_, _outfit_id: i32| {
            Ok((Value::Nil, Value::Nil)) // name, icon
        })?,
    )?;
    t.set(
        "GetAppearanceCameraID",
        lua.create_function(|_, _appearance_id: i32| Ok(0i32))?,
    )?;
    t.set(
        "GetCategoryAppearances",
        lua.create_function(|lua, (_category, _location): (i32, Value)| lua.create_table())?,
    )?;
    t.set(
        "PlayerKnowsSource",
        lua.create_function(|_, _source_id: i32| Ok(false))?,
    )?;
    t.set(
        "IsAppearanceHiddenVisual",
        lua.create_function(|_, _appearance_id: i32| Ok(false))?,
    )?;
    t.set(
        "IsSourceTypeFilterChecked",
        lua.create_function(|_, _filter: i32| Ok(true))?,
    )?;
    t.set(
        "GetShowMissingSourceInItemTooltips",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    lua.globals().set("C_TransmogCollection", t)?;
    Ok(())
}

/// C_Transmog namespace - transmogrification API.
fn register_transmog(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetAllSetAppearancesByID",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetAppliedSourceID",
        lua.create_function(|_, _slot: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetSlotInfo",
        lua.create_function(|_, _slot: i32| {
            Ok((false, false, false, false, false, Value::Nil))
        })?,
    )?;
    lua.globals().set("C_Transmog", t)?;
    Ok(())
}

/// TransmogUtil - utility functions for transmog system.
fn register_transmog_util(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetTransmogLocation",
        lua.create_function(|lua, (slot, transmog_type, modification): (String, i32, i32)| {
            let location = lua.create_table()?;
            location.set("slotName", slot)?;
            location.set("transmogType", transmog_type)?;
            location.set("modification", modification)?;
            Ok(location)
        })?,
    )?;
    t.set(
        "CreateTransmogLocation",
        lua.create_function(|lua, (slot_id, transmog_type, modification): (i32, i32, i32)| {
            let location = lua.create_table()?;
            location.set("slotID", slot_id)?;
            location.set("transmogType", transmog_type)?;
            location.set("modification", modification)?;
            Ok(location)
        })?,
    )?;
    t.set(
        "GetBestItemModifiedAppearanceID",
        lua.create_function(|_, _item_loc: mlua::Value| Ok(Value::Nil))?,
    )?;
    lua.globals().set("TransmogUtil", t)?;
    Ok(())
}

/// C_Heirloom namespace - heirloom collection.
fn register_heirloom(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetHeirloomInfo",
        lua.create_function(|_, _item_id: i32| {
            // Returns: name, itemEquipLoc, isPvP, itemTexture, upgradeLevel, source,
            // searchFiltered, effectiveLevel, minLevel, maxLevel
            Ok((
                Value::Nil, Value::Nil, false, 0i32, 0i32, 0i32, false, 0i32, 0i32, 0i32,
            ))
        })?,
    )?;
    t.set(
        "GetHeirloomMaxUpgradeLevel",
        lua.create_function(|_, _item_id: i32| Ok(0i32))?,
    )?;
    t.set(
        "GetNumHeirlooms",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    t.set(
        "GetNumKnownHeirlooms",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    t.set(
        "PlayerHasHeirloom",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    t.set(
        "GetHeirloomLink",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "CanHeirloomUpgradeFromPending",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    t.set(
        "GetClassAndSpecFilters",
        lua.create_function(|_, ()| Ok((0i32, 0i32)))?,
    )?;
    lua.globals().set("C_Heirloom", t)?;
    Ok(())
}

/// C_TransmogSets namespace - transmog set collection.
fn register_transmog_sets(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetBaseSetID",
        lua.create_function(|_, _set_id: i32| Ok(0i32))?,
    )?;
    t.set(
        "GetVariantSets",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    t.set(
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
    t.set(
        "GetSetPrimaryAppearances",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetAllSets",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "GetUsableSets",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    t.set(
        "IsBaseSetCollected",
        lua.create_function(|_, _set_id: i32| Ok(false))?,
    )?;
    t.set(
        "GetSourcesForSlot",
        lua.create_function(|lua, (_set_id, _slot): (i32, i32)| lua.create_table())?,
    )?;
    lua.globals().set("C_TransmogSets", t)?;
    Ok(())
}
