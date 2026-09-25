local addonName = ...

local addon = {}
_G.ServerSnapshot = addon

local function ensureDatabase()
    if type(ServerSnapshotDB) ~= "table" then
        ServerSnapshotDB = {}
    end
    if type(ServerSnapshotDB.characters) ~= "table" then
        ServerSnapshotDB.characters = {}
    end
    if type(ServerSnapshotDB.history) ~= "table" then
        ServerSnapshotDB.history = {}
    end
    return ServerSnapshotDB
end

local function call(fn, ...)
    if type(fn) ~= "function" then
        return false
    end
    return pcall(fn, ...)
end

local function first(fn, ...)
    local ok, value = call(fn, ...)
    if ok then
        return value
    end
    return nil
end

local function getTime()
    return first(time) or 0
end

local function getCharacterKey()
    local name = first(UnitName, "player") or "Unknown"
    local realm = first(GetRealmName) or "UnknownRealm"
    return realm .. "/" .. name
end

local function getPlayerGuid()
    return first(UnitGUID, "player")
end

local function getBuildInfo()
    local ok, version, build, date, interface = call(GetBuildInfo)
    if not ok then
        return {}
    end
    return {
        version = version,
        build = build,
        date = date,
        interface = interface,
    }
end

local function getClassInfo()
    local ok, localized, english, classID = call(UnitClass, "player")
    if not ok then
        return {}
    end
    return {
        localized = localized,
        english = english,
        classID = classID,
    }
end

local function getSpecInfo()
    local specIndex = first(GetSpecialization)
    if not specIndex then
        return nil
    end

    local ok, id, name, description, icon, role, primaryStat = call(GetSpecializationInfo, specIndex)
    if not ok then
        return {
            index = specIndex,
        }
    end

    return {
        index = specIndex,
        id = id,
        name = name,
        description = description,
        icon = icon,
        role = role,
        primaryStat = primaryStat,
    }
end

local function actionCooldown(slot)
    local ok, start, duration, enabled, modRate = call(GetActionCooldown, slot)
    if not ok then
        return nil
    end
    return {
        start = start,
        duration = duration,
        enabled = enabled,
        modRate = modRate,
    }
end

local function actionCharges(slot)
    local ok, current, max, cooldownStart, cooldownDuration, chargeModRate = call(GetActionCharges, slot)
    if not ok then
        return nil
    end
    return {
        current = current,
        max = max,
        cooldownStart = cooldownStart,
        cooldownDuration = cooldownDuration,
        chargeModRate = chargeModRate,
    }
end

local function actionUsable(slot)
    local ok, usable, noMana = call(IsUsableAction, slot)
    if not ok then
        return nil
    end
    return {
        usable = usable,
        noMana = noMana,
    }
end

local function snapshotActionSlot(slot)
    local ok, actionType, id, subType, spellID = call(GetActionInfo, slot)
    if not ok then
        return {
            slot = slot,
            error = "GetActionInfo failed",
        }
    end

    if actionType == nil then
        return {
            slot = slot,
            empty = true,
        }
    end

    return {
        slot = slot,
        type = actionType,
        id = id,
        subType = subType,
        spellID = spellID,
        text = first(GetActionText, slot),
        texture = first(GetActionTexture, slot),
        count = first(GetActionCount, slot),
        cooldown = actionCooldown(slot),
        charges = actionCharges(slot),
        usable = actionUsable(slot),
        isAttack = first(IsAttackAction, slot),
        isCurrent = first(IsCurrentAction, slot),
        isEquipped = first(IsEquippedAction, slot),
        isPassive = first(IsActionPassive, slot),
    }
end

local function snapshotActionBars()
    local maxSlots = MAX_ACTIONBAR_ACTIONS or 180
    local slots = {}

    for slot = 1, maxSlots do
        slots[slot] = snapshotActionSlot(slot)
    end

    return {
        maxSlots = maxSlots,
        slots = slots,
    }
end

local function spellBookItem(index, bookType)
    local ok, itemType, id = call(GetSpellBookItemInfo, index, bookType)
    if not ok then
        return nil
    end

    local name, subName = first(GetSpellBookItemName, index, bookType)
    local spellID = nil
    local okSpellID, resolvedSpellID = call(GetSpellBookItemSpellID, index, bookType)
    if okSpellID then
        spellID = resolvedSpellID
    end

    return {
        index = index,
        bookType = bookType,
        type = itemType,
        id = id,
        spellID = spellID,
        name = name,
        subName = subName,
        texture = first(GetSpellBookItemTexture, index, bookType),
    }
end

local function snapshotLegacySpellBook()
    local tabs = {}
    local numTabs = first(GetNumSpellTabs)
    if type(numTabs) ~= "number" then
        return tabs
    end

    for tabIndex = 1, numTabs do
        local ok, name, texture, offset, numSlots, isGuild, offSpecID = call(GetSpellTabInfo, tabIndex)
        if ok then
            local tab = {
                index = tabIndex,
                name = name,
                texture = texture,
                offset = offset,
                numSlots = numSlots,
                isGuild = isGuild,
                offSpecID = offSpecID,
                spells = {},
            }

            for localIndex = 1, (numSlots or 0) do
                local spell = spellBookItem((offset or 0) + localIndex, BOOKTYPE_SPELL or "spell")
                if spell then
                    tab.spells[localIndex] = spell
                end
            end

            tabs[tabIndex] = tab
        end
    end

    return tabs
end

local function snapshotCSpellBook()
    local cSpellBook = C_SpellBook
    if type(cSpellBook) ~= "table" then
        return nil
    end

    local result = {}

    if type(cSpellBook.GetSpellBookSkillLineInfo) == "function" and type(cSpellBook.GetNumSpellBookSkillLines) == "function" then
        local okNum, numLines = call(cSpellBook.GetNumSpellBookSkillLines)
        if okNum and type(numLines) == "number" then
            result.skillLines = {}
            for lineIndex = 1, numLines do
                local okLine, lineInfo = call(cSpellBook.GetSpellBookSkillLineInfo, lineIndex)
                if okLine then
                    result.skillLines[lineIndex] = lineInfo
                end
            end
        end
    end

    if type(cSpellBook.GetSpellBookItemInfo) == "function" then
        result.note = "C_SpellBook exists; legacy spellbook snapshot is stored separately when available."
    end

    return result
end

local function snapshotSpellBook()
    return {
        legacy = snapshotLegacySpellBook(),
        cSpellBook = snapshotCSpellBook(),
    }
end

local function snapshotBags()
    local container = C_Container
    local maxBagID = NUM_TOTAL_EQUIPPED_BAG_SLOTS
    if type(container) ~= "table" or BACKPACK_CONTAINER ~= 0
        or type(maxBagID) ~= "number"
        or type(container.GetContainerNumSlots) ~= "function"
        or type(container.GetContainerNumFreeSlots) ~= "function"
        or type(container.GetContainerItemInfo) ~= "function"
        or type(container.ContainerIDToInventoryID) ~= "function" then
        return nil
    end

    local bags = { maxBagID = maxBagID, containers = {} }
    for bagID = BACKPACK_CONTAINER, maxBagID do
        local okSlots, numSlots = call(container.GetContainerNumSlots, bagID)
        local okFamily, _, family = call(container.GetContainerNumFreeSlots, bagID)
        if not okSlots or type(numSlots) ~= "number" or not okFamily or type(family) ~= "number"
            or (bagID == 0 and numSlots == 0) then
            return nil
        end

        local bag = {
            numSlots = numSlots,
            family = family,
            name = first(GetBagName, bagID),
            items = {},
        }
        if bagID > 0 then
            bag.inventorySlot = first(container.ContainerIDToInventoryID, bagID)
            if type(bag.inventorySlot) ~= "number" then
                return nil
            end
            bag.itemID = first(GetInventoryItemID, "player", bag.inventorySlot)
            bag.hyperlink = first(GetInventoryItemLink, "player", bag.inventorySlot)
        end

        for slot = 1, numSlots do
            local okItem, info = call(container.GetContainerItemInfo, bagID, slot)
            if not okItem then
                return nil
            end
            if info then
                if type(info) ~= "table" or type(info.itemID) ~= "number"
                    or type(info.stackCount) ~= "number" then
                    return nil
                end
                bag.items[slot] = {
                    itemID = info.itemID,
                    stackCount = info.stackCount,
                    hyperlink = info.hyperlink,
                }
            end
        end
        bags.containers[bagID] = bag
    end
    return bags
end

local function snapshotMacros()
    local ok, globalCount, characterCount = call(GetNumMacros)
    if not ok then
        return nil
    end

    local macros = {
        globalCount = globalCount,
        characterCount = characterCount,
        entries = {},
    }

    local total = (globalCount or 0) + (characterCount or 0)
    for index = 1, total do
        local okInfo, name, icon, body, isLocal = call(GetMacroInfo, index)
        if okInfo and name then
            macros.entries[index] = {
                index = index,
                name = name,
                icon = icon,
                body = body,
                isLocal = isLocal,
            }
        end
    end

    return macros
end

local function addonApi()
    if type(C_AddOns) == "table" then
        return C_AddOns
    end
    return nil
end

local function addonNameAt(index)
    local cAddOns = addonApi()
    if cAddOns and type(cAddOns.GetAddOnName) == "function" then
        return first(cAddOns.GetAddOnName, index)
    end

    local ok, name = call(GetAddOnInfo, index)
    if ok then
        return name
    end
    return nil
end

local function addonEnableState(index)
    local cAddOns = addonApi()
    if cAddOns and type(cAddOns.GetAddOnEnableState) == "function" then
        return first(cAddOns.GetAddOnEnableState, index)
    end

    local ok, enabled = call(GetAddOnEnableState, index)
    if ok then
        return enabled
    end
    return nil
end

local function addonInfo(index)
    local cAddOns = addonApi()
    if cAddOns and type(cAddOns.GetAddOnInfo) == "function" then
        local ok, name, title, notes, loadable, reason = call(cAddOns.GetAddOnInfo, index)
        if ok then
            return name, title, notes, loadable, reason
        end
    end

    local ok, name, title, notes, loadable, reason = call(GetAddOnInfo, index)
    if ok then
        return name, title, notes, loadable, reason
    end
    return nil
end

local function snapshotAddons()
    local cAddOns = addonApi()
    local count = nil
    if cAddOns and type(cAddOns.GetNumAddOns) == "function" then
        count = first(cAddOns.GetNumAddOns)
    end
    if type(count) ~= "number" then
        count = first(GetNumAddOns)
    end
    if type(count) ~= "number" then
        return nil
    end

    local addons = {
        count = count,
        entries = {},
        order = {},
    }

    for index = 1, count do
        local folderName = addonNameAt(index)
        local infoName, title, notes, loadable, reason = addonInfo(index)
        folderName = folderName or infoName
        if type(folderName) == "string" and folderName ~= "" then
            local enableState = addonEnableState(index)
            local enabled = nil
            if type(enableState) == "number" then
                enabled = enableState > 0
            elseif type(enableState) == "boolean" then
                enabled = enableState
            end

            table.insert(addons.order, folderName)
            addons.entries[folderName] = {
                index = index,
                title = title,
                notes = notes,
                loadable = loadable,
                reason = reason,
                enableState = enableState,
                enabled = enabled,
            }
        end
    end

    return addons
end

local KEYBINDING_PROBE_KEYS = {
    "ESCAPE", "SPACE", "F1", "F2", "F3", "F4", "F5", "F6",
    "F7", "F8", "F9", "F10", "F11", "F12",
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0",
    "-", "=", "CTRL-1", "CTRL-2", "CTRL-3", "CTRL-4", "CTRL-5",
    "SHIFT-1", "SHIFT-2", "SHIFT-3", "SHIFT-4", "SHIFT-5",
    "ALT-1", "ALT-2", "ALT-3", "ALT-4", "ALT-5",
    "B", "C", "G", "I", "J", "K", "L", "M", "N", "O", "P",
    "S", "U", "Y", "Z", "ALT-P", "SHIFT-B", "SHIFT-C", "SHIFT-I",
    "SHIFT-J", "SHIFT-M", "SHIFT-P", "SHIFT-S", "SHIFT-T", "CTRL-M",
}

local function snapshotKeybindings()
    local count = first(GetNumBindings)
    if type(count) ~= "number" then
        return nil
    end

    local bindings = {
        count = count,
        entries = {},
        keys = {},
    }

    for index = 1, count do
        local ok, action, category, key1, key2 = call(GetBinding, index)
        if ok and type(action) == "string" and action ~= "" then
            local keys = {}
            if type(key1) == "string" and key1 ~= "" then
                table.insert(keys, key1)
            end
            if type(key2) == "string" and key2 ~= "" then
                table.insert(keys, key2)
            end
            table.insert(bindings.entries, {
                index = index,
                action = action,
                category = category,
                keys = keys,
            })
        end
    end

    for _, key in ipairs(KEYBINDING_PROBE_KEYS) do
        local action = first(GetBindingAction, key)
        bindings.keys[key] = type(action) == "string" and action or ""
    end

    return bindings
end

local function snapshotTalentConfig()
    local classTalents = C_ClassTalents
    if type(classTalents) ~= "table" then
        return nil
    end

    local talents = {}

    if type(classTalents.GetActiveConfigID) == "function" then
        talents.activeConfigID = first(classTalents.GetActiveConfigID)
    end

    if type(classTalents.GetLastSelectedSavedConfigID) == "function" then
        talents.lastSelectedSavedConfigID = first(classTalents.GetLastSelectedSavedConfigID)
    end

    if type(classTalents.GetConfigIDsBySpecID) == "function" then
        local spec = getSpecInfo()
        local specID = spec and spec.id
        if specID then
            talents.configIDsBySpecID = first(classTalents.GetConfigIDsBySpecID, specID)
        end
    end

    if type(classTalents.GetConfigInfo) == "function" and talents.activeConfigID then
        talents.activeConfigInfo = first(classTalents.GetConfigInfo, talents.activeConfigID)
    end

    return talents
end

local function snapshotPvpTalents()
    local result = {}

    if type(C_SpecializationInfo) == "table" and type(C_SpecializationInfo.GetAllSelectedPvpTalentIDs) == "function" then
        result.selectedIDs = first(C_SpecializationInfo.GetAllSelectedPvpTalentIDs)
    end

    if type(GetPvpTalentInfoByID) == "function" and type(result.selectedIDs) == "table" then
        result.selected = {}
        for index, talentID in ipairs(result.selectedIDs) do
            local ok, talentIDValue, name, icon, selected, available, spellID, unlocked, row, column, known = call(GetPvpTalentInfoByID, talentID)
            if ok then
                result.selected[index] = {
                    talentID = talentIDValue,
                    name = name,
                    icon = icon,
                    selected = selected,
                    available = available,
                    spellID = spellID,
                    unlocked = unlocked,
                    row = row,
                    column = column,
                    known = known,
                }
            end
        end
    end

    return result
end

local function editModeActiveLayoutName()
    -- EditModeManagerFrame resolves presets + saved layouts and tracks the
    -- active one, so its layoutName is the value the live client is using.
    local emm = EditModeManagerFrame
    if type(emm) == "table" and type(emm.GetActiveLayoutInfo) == "function" then
        local ok, info = call(emm.GetActiveLayoutInfo, emm)
        if ok and type(info) == "table" and type(info.layoutName) == "string" then
            return info.layoutName
        end
    end
    return nil
end

local function loadEditModeAddon()
    if type(UIParentLoadAddOn) == "function" then
        local ok, loaded = call(UIParentLoadAddOn, "Blizzard_EditMode")
        if ok and loaded then
            return
        end
    end
    if type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
        call(C_AddOns.LoadAddOn, "Blizzard_EditMode")
    end
end

local function snapshotEditMode()
    loadEditModeAddon()

    local result = {
        activeLayoutName = editModeActiveLayoutName(),
    }

    local cEditMode = C_EditMode
    if type(cEditMode) == "table" and type(cEditMode.GetLayouts) == "function" then
        local ok, layoutInfo = call(cEditMode.GetLayouts)
        if ok and type(layoutInfo) == "table" then
            result.activeLayout = tonumber(layoutInfo.activeLayout)
            if type(layoutInfo.layouts) == "table" then
                local names = {}
                for index, layout in ipairs(layoutInfo.layouts) do
                    if type(layout) == "table" then
                        names[index] = layout.layoutName
                    end
                end
                result.layoutNames = names
                if result.activeLayoutName == nil and result.activeLayout ~= nil then
                    result.activeLayoutName = names[result.activeLayout]
                end
            end
        end
    end

    if result.activeLayoutName == nil and result.layoutNames == nil then
        return nil
    end
    return result
end

function addon:Snapshot(reason)
    local db = ensureDatabase()
    local characterKey = getCharacterKey()

    local snapshot = {
        addon = addonName,
        reason = reason or "manual",
        capturedAt = getTime(),
        characterKey = characterKey,
        guid = getPlayerGuid(),
        build = getBuildInfo(),
        class = getClassInfo(),
        specialization = getSpecInfo(),
        actionBars = snapshotActionBars(),
        bags = snapshotBags(),
        spellBook = snapshotSpellBook(),
        macros = snapshotMacros(),
        addons = snapshotAddons(),
        keybindings = snapshotKeybindings(),
        talents = snapshotTalentConfig(),
        pvpTalents = snapshotPvpTalents(),
        editMode = snapshotEditMode(),
    }

    db.characters[characterKey] = snapshot
    db.lastCharacterKey = characterKey
    db.lastCapturedAt = snapshot.capturedAt
    db.version = 1

    table.insert(db.history, {
        characterKey = characterKey,
        capturedAt = snapshot.capturedAt,
        reason = snapshot.reason,
    })

    while #db.history > 50 do
        table.remove(db.history, 1)
    end

    return snapshot
end

local function printStatus(reason)
    local snapshot = addon:Snapshot(reason)
    local actionBars = snapshot.actionBars or {}
    local slots = actionBars.slots or {}
    print(string.format("%s: saved %s with %d action slots", addonName, snapshot.characterKey, #slots))
end

local events = {
    PLAYER_LOGIN = true,
    PLAYER_ENTERING_WORLD = true,
    BAG_UPDATE_DELAYED = true,
    BAG_CONTAINER_UPDATE = true,
    PLAYER_LOGOUT = true,
    ACTIONBAR_SLOT_CHANGED = true,
    ACTIONBAR_PAGE_CHANGED = true,
    UPDATE_BONUS_ACTIONBAR = true,
    UPDATE_OVERRIDE_ACTIONBAR = true,
    UPDATE_POSSESS_BAR = true,
    SPELLS_CHANGED = true,
    PLAYER_TALENT_UPDATE = true,
    ACTIVE_TALENT_GROUP_CHANGED = true,
    TRAIT_CONFIG_UPDATED = true,
    PLAYER_SPECIALIZATION_CHANGED = true,
    UPDATE_MACROS = true,
    UPDATE_BINDINGS = true,
    EDIT_MODE_LAYOUTS_UPDATED = true,
}

local frame = CreateFrame("Frame")
for event in pairs(events) do
    pcall(frame.RegisterEvent, frame, event)
end

frame:SetScript("OnEvent", function(_, event)
    addon:Snapshot(event)
end)

SLASH_SERVERSNAPSHOT1 = "/serversnapshot"
SLASH_SERVERSNAPSHOT2 = "/ssnap"
SlashCmdList.SERVERSNAPSHOT = function()
    printStatus("slash")
end

if type(hooksecurefunc) == "function" and type(AddonList_OnOkay) == "function" then
    hooksecurefunc("AddonList_OnOkay", function()
        addon:Snapshot("AddonList_OnOkay")
    end)
end

local function safeSnapshot(reason)
    local ok, err = pcall(addon.Snapshot, addon, reason)
    if not ok and DEFAULT_CHAT_FRAME then
        DEFAULT_CHAT_FRAME:AddMessage(addonName .. ": snapshot failed: " .. tostring(err))
    end
end

safeSnapshot("load")

if C_Timer and type(C_Timer.After) == "function" then
    C_Timer.After(1, function()
        safeSnapshot("delayed-load")
    end)
end
