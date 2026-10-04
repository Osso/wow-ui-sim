//! Temporary private-aura state surface.
//!
//! Private aura data and update callbacks remain temporary compatibility behavior.
//! Anchor registration, warning placement and anchor callbacks are owned by
//! c_api::private_aura_anchors.

const PRIVATE_AURA_STATE_LUA: &str = r#"
if type(C_UnitAuras) ~= "table" then
    C_UnitAuras = {}
end
if type(C_UnitAurasPrivate) ~= "table" then
    C_UnitAurasPrivate = {}
end

if type(C_UnitAurasPrivate._state) ~= "table" then
    C_UnitAurasPrivate._state = {}
end

local function PrivateAuraState()
    local state = C_UnitAurasPrivate._state
    if type(state.privateAurasByUnit) ~= "table" then
        state.privateAurasByUnit = {}
    end
    if type(state.auraDataByUnit) ~= "table" then
        state.auraDataByUnit = {}
    end
    if type(state.updateCallbacksByUnit) ~= "table" then
        state.updateCallbacksByUnit = {}
    end
    return state
end

PrivateAuraState()

local function CopyPrivateAuraValue(value, seen)
    if type(value) ~= "table" then
        return value
    end
    seen = seen or {}
    if seen[value] ~= nil then
        return seen[value]
    end
    local copy = {}
    seen[value] = copy
    for key, nested in pairs(value) do
        copy[CopyPrivateAuraValue(key, seen)] = CopyPrivateAuraValue(nested, seen)
    end
    local mt = getmetatable(value)
    if mt ~= nil then
        setmetatable(copy, mt)
    end
    return copy
end

local function CopyPrivateAuraList(list)
    local copy = {}
    for index = 1, #(list or {}) do
        copy[index] = CopyPrivateAuraValue(list[index])
    end
    return copy
end

if rawget(C_UnitAurasPrivate, "SetShowDispelTypeCallback") == nil then
    function C_UnitAurasPrivate.SetShowDispelTypeCallback(callback)
        C_UnitAurasPrivate._showDispelTypeCallback = callback
    end
end

if rawget(C_UnitAurasPrivate, "AddPrivateAuraUpdateCallback") == nil then
    function C_UnitAurasPrivate.AddPrivateAuraUpdateCallback(unitToken, callback)
        local state = PrivateAuraState()
        local key = tostring(unitToken or "")
        local callbacks = state.updateCallbacksByUnit[key]
        if type(callbacks) ~= "table" then
            callbacks = {}
            state.updateCallbacksByUnit[key] = callbacks
        end
        callbacks[#callbacks + 1] = callback
    end
end

if rawget(C_UnitAurasPrivate, "_TriggerPrivateAuraUpdate") == nil then
    function C_UnitAurasPrivate._TriggerPrivateAuraUpdate(unitToken, privateSource, updateInfo)
        local state = PrivateAuraState()
        local callbacks = state.updateCallbacksByUnit[tostring(unitToken or "")]
        local fired = 0
        for index = 1, #(callbacks or {}) do
            if type(callbacks[index]) == "function" then
                callbacks[index](privateSource, CopyPrivateAuraValue(updateInfo))
                fired = fired + 1
            end
        end
        return fired
    end
end

if rawget(C_UnitAurasPrivate, "GetAllPrivateAuras") == nil then
    function C_UnitAurasPrivate.GetAllPrivateAuras(unitToken)
        local state = PrivateAuraState()
        return CopyPrivateAuraList(state.privateAurasByUnit[tostring(unitToken or "")] or {})
    end
end

if rawget(C_UnitAurasPrivate, "GetAuraDataByAuraInstanceIDPrivate") == nil then
    function C_UnitAurasPrivate.GetAuraDataByAuraInstanceIDPrivate(unitToken, auraInstanceID)
        local state = PrivateAuraState()
        local byUnit = state.auraDataByUnit[tostring(unitToken or "")]
        if type(byUnit) ~= "table" then
            return nil
        end
        local aura = byUnit[auraInstanceID]
        if aura == nil and auraInstanceID ~= nil then
            aura = byUnit[tonumber(auraInstanceID)]
        end
        if aura == nil then
            return nil
        end
        return CopyPrivateAuraValue(aura)
    end
end
"#;

// 12.1.0 removed the public trigger.
#[cfg(not(feature = "retail-12-1-0"))]
const SHOW_DISPEL_TYPE_TRIGGER_LUA: &str = r#"
if rawget(C_UnitAuras, "TriggerPrivateAuraShowDispelType") == nil then
    function C_UnitAuras.TriggerPrivateAuraShowDispelType(showDispelType)
        local state = C_UnitAurasPrivate._state
        state.lastShowDispelType = showDispelType
        if type(C_UnitAurasPrivate._showDispelTypeCallback) == "function" then
            C_UnitAurasPrivate._showDispelTypeCallback(showDispelType)
        end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PRIVATE_AURA_STATE_LUA)?;
    #[cfg(not(feature = "retail-12-1-0"))]
    lua.exec(SHOW_DISPEL_TYPE_TRIGGER_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_private_aura_state_and_callbacks() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local addedID, removedID, dispel, updateSource, updateSpell
                C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(anchor)
                    addedID = anchor.anchorID
                    anchor.unitToken = "mutated"
                end)
                C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(anchorID)
                    removedID = anchorID
                end)
                local anchorID = C_UnitAuras.AddPrivateAuraAnchor({
                    unitToken = "player",
                    auraIndex = 1,
                    parent = CreateFrame("Frame"),
                })
                if anchorID ~= 1 or addedID ~= 1 then
                    return "bad_anchor_add"
                end
                local anchors = C_UnitAurasPrivate.GetPrivateAuraAnchors("player")
                if #anchors ~= 1 or anchors[1].unitToken ~= "player" then
                    return "bad_anchor_copy"
                end
                anchors[1].unitToken = "mutated"
                if C_UnitAurasPrivate.GetPrivateAuraAnchors("player")[1].unitToken ~= "player" then
                    return "leaked_anchor_copy"
                end
                if select('#', C_UnitAuras.RemovePrivateAuraAnchor(anchorID)) ~= 0 or removedID ~= anchorID then
                    return "bad_anchor_remove"
                end
                local warning = CreateFrame("Frame")
                C_UnitAurasPrivate.SetPrivateWarningTextFrame(warning)
                if C_UnitAurasPrivate._state.warningTextFrame ~= warning then
                    return "bad_warning_frame"
                end
                if rawget(C_UnitAuras, "TriggerPrivateAuraShowDispelType") ~= nil then
                    C_UnitAurasPrivate.SetShowDispelTypeCallback(function(value)
                        dispel = value
                    end)
                    C_UnitAuras.TriggerPrivateAuraShowDispelType(true)
                    if dispel ~= true or C_UnitAurasPrivate._state.lastShowDispelType ~= true then
                        return "bad_dispel"
                    end
                end
                C_UnitAurasPrivate.AddPrivateAuraUpdateCallback("player", function(source, info)
                    updateSource = source.name
                    updateSpell = info.spellID
                    info.spellID = 0
                end)
                local fired = C_UnitAurasPrivate._TriggerPrivateAuraUpdate("player", { name = "source" }, { spellID = 123 })
                if fired ~= 1 or updateSource ~= "source" or updateSpell ~= 123 then
                    return "bad_update"
                end
                C_UnitAurasPrivate._state.privateAurasByUnit.player = {
                    { auraInstanceID = 7, spellID = 321 },
                }
                local privateAuras = C_UnitAurasPrivate.GetAllPrivateAuras("player")
                privateAuras[1].spellID = 0
                if C_UnitAurasPrivate.GetAllPrivateAuras("player")[1].spellID ~= 321 then
                    return "bad_private_aura_copy"
                end
                C_UnitAurasPrivate._state.auraDataByUnit.player = {
                    [7] = { auraInstanceID = 7, spellID = 456 },
                }
                local aura = C_UnitAurasPrivate.GetAuraDataByAuraInstanceIDPrivate("player", "7")
                aura.spellID = 0
                if C_UnitAurasPrivate.GetAuraDataByAuraInstanceIDPrivate("player", 7).spellID ~= 456 then
                    return "bad_aura_data_copy"
                end
                return "ok"
                "#,
            )
            .expect("private aura probe should run");

        assert_eq!(result, "ok");
    }
}
