//! Temporary `C_TransmogOutfitInfo` slot/outfit defaults.
//!
//! Outfit locks are state-backed in `lua_api::globals::transmog_outfit_info`.
//! Slot metadata, sheathe categories, and viewed metadata remain compatibility
//! defaults. Retail 12.0.5 applied selection belongs to the catalog-backed C API.

const TRANSMOG_OUTFIT_SLOT_DEFAULTS_LUA: &str = r#"
C_TransmogOutfitInfo = C_TransmogOutfitInfo or __wow_namespace()

local CURRENTLY_VIEWED_OUTFIT_ID_KEY = "__currentlyViewedOutfitID"
local PENDING_SHEATHE_CATEGORIES_KEY = "__pendingSheatheCategories"
local VALID_SHEATHE_SLOT_TRANSMOG_ID = 190001

local function enumValue(enumName, key, fallback)
    local enumRoot = rawget(_G, "Enum")
    local enumTable = enumRoot and enumRoot[enumName]
    local value = enumTable and enumTable[key]
    if type(value) == "number" then
        return value
    end

    return fallback
end

local function outfitIDValue(key)
    local value = rawget(C_TransmogOutfitInfo, key)
    if type(value) == "number" then
        return value
    end

    return 0
end

local function numberOrNumericString(value)
    if type(value) == "number" then
        return value
    end

    return tonumber(value)
end

local function luaKeyString(value)
    if type(value) == "number" and value % 1 == 0 then
        return string.format("%d", value)
    end

    return tostring(value)
end

local function pendingSheatheCategories()
    local pending = rawget(C_TransmogOutfitInfo, PENDING_SHEATHE_CATEGORIES_KEY)
    if type(pending) == "table" then
        return pending
    end

    pending = {}
    C_TransmogOutfitInfo[PENDING_SHEATHE_CATEGORIES_KEY] = pending
    return pending
end

local function slotInfo(slotName, inventorySlotID, collectionTypeName, collectionTypeFallback, isSecondary, transmogType)
    return {
        slot = math.max(inventorySlotID - 1, 0),
        type = transmogType,
        collectionType = enumValue("TransmogCollectionType", collectionTypeName, collectionTypeFallback),
        slotName = slotName,
        isSecondary = isSecondary == true,
    }
end

local appearanceSlotSpecs = {
    { "HEADSLOT", 1, "Head", 1, false },
    { "SHOULDERSLOT", 3, "Shoulder", 2, false },
    { "BACKSLOT", 15, "Back", 3, false },
    { "CHESTSLOT", 5, "Chest", 4, false },
    { "SHIRTSLOT", 4, "Shirt", 5, false },
    { "TABARDSLOT", 19, "Tabard", 6, false },
    { "WRISTSLOT", 9, "Wrist", 7, false },
    { "HANDSSLOT", 10, "Hands", 8, false },
    { "WAISTSLOT", 6, "Waist", 9, false },
    { "LEGSSLOT", 7, "Legs", 10, false },
    { "FEETSLOT", 8, "Feet", 11, false },
    { "MAINHANDSLOT", 16, "None", 0, false },
    { "SECONDARYHANDSLOT", 17, "None", 0, false },
    { "SHOULDERSLOT", 3, "Shoulder", 2, true },
}

local illusionSlotSpecs = {
    { "MAINHANDSLOT", 16, "None", 0, false },
    { "SECONDARYHANDSLOT", 17, "None", 0, false },
}

local function buildSlotArray(specs, transmogType)
    local slots = {}
    for index, spec in ipairs(specs) do
        slots[index] = slotInfo(spec[1], spec[2], spec[3], spec[4], spec[5], transmogType)
    end
    return slots
end

if rawget(C_TransmogOutfitInfo, "GetCurrentlyViewedOutfitID") == nil then
    function C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()
        return outfitIDValue(CURRENTLY_VIEWED_OUTFIT_ID_KEY)
    end
end

if rawget(C_TransmogOutfitInfo, "GetAllTransmogOutfitOptionSheatheCategoryInfo") == nil then
    function C_TransmogOutfitInfo.GetAllTransmogOutfitOptionSheatheCategoryInfo(slotTransmogID)
        if numberOrNumericString(slotTransmogID) ~= VALID_SHEATHE_SLOT_TRANSMOG_ID then
            return nil
        end

        return {
            { sheatheCategory = enumValue("TransmogOutfitSlotOptionSheatheCategory", "Default", 0), categoryName = "Default" },
            { sheatheCategory = enumValue("TransmogOutfitSlotOptionSheatheCategory", "Back", 1), categoryName = "Back" },
            { sheatheCategory = enumValue("TransmogOutfitSlotOptionSheatheCategory", "Side", 2), categoryName = "Side" },
            { sheatheCategory = enumValue("TransmogOutfitSlotOptionSheatheCategory", "Hide", 3), categoryName = "Hide" },
        }
    end
end

if rawget(C_TransmogOutfitInfo, "SetPendingTransmogSheatheCategory") == nil then
    function C_TransmogOutfitInfo.SetPendingTransmogSheatheCategory(slotID, optionID, category)
        pendingSheatheCategories()[luaKeyString(slotID) .. ":" .. luaKeyString(optionID)] = category
    end
end

if rawget(C_TransmogOutfitInfo, "GetTransmogOutfitSlotFromInventorySlot") == nil then
    function C_TransmogOutfitInfo.GetTransmogOutfitSlotFromInventorySlot(slot)
        slot = numberOrNumericString(slot)
        if slot == nil or slot < 0 then
            return nil
        end

        return slot
    end
end

if rawget(C_TransmogOutfitInfo, "GetLinkedSlotInfo") == nil then
    function C_TransmogOutfitInfo.GetLinkedSlotInfo()
        return nil
    end
end

if rawget(C_TransmogOutfitInfo, "GetAllSlotLocationInfo") == nil then
    function C_TransmogOutfitInfo.GetAllSlotLocationInfo()
        local appearanceType = enumValue("TransmogType", "Appearance", 0)
        local illusionType = enumValue("TransmogType", "Illusion", 1)
        return buildSlotArray(appearanceSlotSpecs, appearanceType),
               buildSlotArray(illusionSlotSpecs, illusionType)
    end
end
"#;

// Earlier patch epochs retain their existing compatibility surface. This chunk
// is not compiled into Retail 12.0.5; it is not a fallback for the modeled APIs.
#[cfg(not(feature = "retail-12-0-5"))]
const LEGACY_OUTFIT_SELECTION_LUA: &str = r#"
local CURRENTLY_VIEWED_OUTFIT_ID_KEY = "__currentlyViewedOutfitID"
local PENDING_SHEATHE_CATEGORIES_KEY = "__pendingSheatheCategories"
local function outfitIDValue(key)
    local value = rawget(C_TransmogOutfitInfo, key)
    if type(value) == "number" then return value end
    return 0
end
local function numberOrNumericString(value)
    if type(value) == "number" then return value end
    return tonumber(value)
end
local ACTIVE_OUTFIT_ID_KEY = "__activeOutfitID"
local function setOutfitIDs(outfitID)
    C_TransmogOutfitInfo[ACTIVE_OUTFIT_ID_KEY] = outfitID
    C_TransmogOutfitInfo[CURRENTLY_VIEWED_OUTFIT_ID_KEY] = outfitID
end

local function resetOutfitState()
    setOutfitIDs(0)
    C_TransmogOutfitInfo[PENDING_SHEATHE_CATEGORIES_KEY] = {}
end

if rawget(C_TransmogOutfitInfo, "GetActiveOutfitID") == nil then
    function C_TransmogOutfitInfo.GetActiveOutfitID()
        return outfitIDValue(ACTIVE_OUTFIT_ID_KEY)
    end
end

if rawget(C_TransmogOutfitInfo, "ChangeToOutfit") == nil then
    function C_TransmogOutfitInfo.ChangeToOutfit(outfitID, clear)
        if clear then
            resetOutfitState()
            return
        end

        setOutfitIDs(numberOrNumericString(outfitID) or 0)
    end
end

if rawget(C_TransmogOutfitInfo, "ClearOutfit") == nil then
    function C_TransmogOutfitInfo.ClearOutfit()
        resetOutfitState()
    end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(TRANSMOG_OUTFIT_SLOT_DEFAULTS_LUA)?;
    #[cfg(not(feature = "retail-12-0-5"))]
    lua.exec(LEGACY_OUTFIT_SELECTION_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn outfit_slot_from_inventory_slot_preserves_valid_nonnegative_values() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: (i32, bool, bool) = env
            .eval(
                r#"
                return C_TransmogOutfitInfo.GetTransmogOutfitSlotFromInventorySlot(16),
                       C_TransmogOutfitInfo.GetTransmogOutfitSlotFromInventorySlot(-1) == nil,
                       C_TransmogOutfitInfo.GetLinkedSlotInfo(16) == nil
                "#,
            )
            .expect("outfit slot helpers should be queryable");

        assert_eq!(result, (16, true, true));
    }

    #[test]
    fn slot_location_info_exposes_appearance_and_illusion_slots() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: (i32, i32, String, bool, String) = env
            .eval(
                r#"
                local appearanceSlotInfo, illusionSlotInfo = C_TransmogOutfitInfo.GetAllSlotLocationInfo()
                return #appearanceSlotInfo,
                       #illusionSlotInfo,
                       appearanceSlotInfo[1].slotName,
                       appearanceSlotInfo[#appearanceSlotInfo].isSecondary,
                       illusionSlotInfo[1].slotName
                "#,
            )
            .expect("slot location info should be queryable");

        assert_eq!(
            result,
            (
                14,
                2,
                "HEADSLOT".to_string(),
                true,
                "MAINHANDSLOT".to_string()
            )
        );
    }

    #[test]
    #[cfg(not(feature = "retail-12-0-5"))]
    fn outfit_state_methods_track_active_outfit_and_pending_sheathe_categories() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: String = env
            .eval(
                r#"
                if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 0 then return "active" end
                if C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 0 then return "viewed" end
                if C_TransmogOutfitInfo.GetOutfitInfo(7) ~= nil then return "outfit_info" end

                local categoryInfo = C_TransmogOutfitInfo.GetAllTransmogOutfitOptionSheatheCategoryInfo(190001)
                if #categoryInfo ~= 4 then return "category_count" end
                if categoryInfo[1].categoryName ~= "Default" then return "default_category" end
                if categoryInfo[4].categoryName ~= "Hide" then return "hide_category" end
                if C_TransmogOutfitInfo.GetAllTransmogOutfitOptionSheatheCategoryInfo(0) ~= nil then
                    return "unexpected_category"
                end

                C_TransmogOutfitInfo.ChangeToOutfit(7, false)
                if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 7 then return "changed_active" end
                if C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 7 then return "changed_viewed" end

                C_TransmogOutfitInfo.SetPendingTransmogSheatheCategory(16, 2, Enum.TransmogOutfitSlotOptionSheatheCategory.Side)
                local pending = rawget(C_TransmogOutfitInfo, "__pendingSheatheCategories")
                if pending["16:2"] ~= Enum.TransmogOutfitSlotOptionSheatheCategory.Side then return "pending" end

                C_TransmogOutfitInfo.ChangeToOutfit(7, true)
                if C_TransmogOutfitInfo.GetActiveOutfitID() ~= 0 then return "cleared_active" end
                if next(rawget(C_TransmogOutfitInfo, "__pendingSheatheCategories")) ~= nil then return "cleared_pending" end

                C_TransmogOutfitInfo.ChangeToOutfit(9, false)
                C_TransmogOutfitInfo.ClearOutfit()
                if C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() ~= 0 then return "clear_outfit" end
                return "ok"
                "#,
            )
            .expect("outfit state methods should be queryable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_state_backed_lock_methods() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.state().borrow_mut().transmog_outfit_locks.insert(7);

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (bool, String) = env
            .eval(
                r#"
                return C_TransmogOutfitInfo.IsLockedOutfit(7),
                       type(C_TransmogOutfitInfo.GetAllSlotLocationInfo)
                "#,
            )
            .expect("state-backed lock method should be preserved");

        assert_eq!(result, (true, "function".to_string()));
    }
}
