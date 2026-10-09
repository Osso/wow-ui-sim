//! Single-entry publication/absence probe shared by factory and sweep tests.
//! Publication/absence only: no signature, output, security, or behavior parity claim.

use std::collections::BTreeMap;

use serde::Deserialize;
use wow_ui_sim::lua_api::WowLuaEnv;

#[derive(Deserialize)]
pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) section: String,
    pub(crate) direction: String,
    pub(crate) symbol: String,
    #[serde(default, alias = "default")]
    pub(crate) page_default: Option<String>,
    #[serde(default)]
    pub(crate) kind: Option<String>,
}

type ProbeResult = (String, String, bool, Option<String>, Option<String>);

// One classifier; constructor choices are per object kind, never per source symbol.
const CLASSIFIER: &str = r#"
local section, symbol, removed, entryKind, aliasTarget, aliasSource = ...
local function result(kind, detail, ok, value, default)
    return kind, detail, ok, value, default
end
local function matches_type(raw, lookup)
    if removed then return raw == 'nil' and lookup == 'nil' end
    return raw == 'function' or raw == 'table'
end
-- Removed symbols may be republished by cached Blizzard deprecation fallbacks
-- (loadDeprecationFallbacks defaults on). Direct aliases retain the destination's
-- source, so loaded cached assignments also need exact target identity proof.
local function deprecated_fallback_source(value)
    if type(value) ~= 'function' then return nil end
    local source = debug.getinfo(value, 'S').source or ''
    if string.find(source, 'Deprecated', 1, true) then return source end
    if aliasTarget ~= '' and GetCVarBool('loadDeprecationFallbacks') then
        local target = _G
        for part in string.gmatch(aliasTarget, '[^.]+') do
            if type(target) ~= 'table' then return nil end
            target = rawget(target, part)
        end
        if rawequal(value, target) then return aliasSource .. '; alias=' .. aliasTarget end
    end
    return nil
end
local function removed_result(kind, detail, raw_value, raw, lookup)
    local source = deprecated_fallback_source(raw_value)
    if source then
        return result(kind, detail .. '; deprecated-fallback=' .. source, true)
    end
    return result(kind, detail, matches_type(raw, lookup))
end
local function resolve_raw_path(path)
    local value = _G
    for part in string.gmatch(path, '[^.]+') do
        if type(value) ~= 'table' then return nil end
        value = rawget(value, part)
    end
    return value
end
local function resolve_lookup_path(path)
    local value = _G
    for part in string.gmatch(path, '[^.:]+') do
        if value == nil then return nil end
        value = value[part]
    end
    return value
end
local function probe_publication()
    local parent_path, member = string.match(symbol, '^(.*)[.:]([^.:]+)$')
    if not parent_path then
        local raw_value = rawget(_G, symbol)
        local raw = type(raw_value)
        local lookup_ok, lookup_value = pcall(function() return _G[symbol] end)
        local lookup = lookup_ok and type(lookup_value) or 'lookup-error'
        local detail = 'raw=' .. raw .. '; lookup=' .. lookup
        if removed then return removed_result('global', detail, raw_value, raw, lookup) end
        return result('global', detail, matches_type(raw, lookup))
    end
    local parent = resolve_raw_path(parent_path)
    if parent == nil then
        local lookup_ok, lookup_value = pcall(resolve_lookup_path, symbol)
        local lookup = lookup_ok and type(lookup_value) or 'lookup-error'
        return result('member', 'parent raw-absent; raw=nil; lookup=' .. lookup,
            removed and lookup == 'nil')
    end
    if type(parent) ~= 'table' then
        return result('unprobeable', 'parent is not a table: ' .. type(parent), false)
    end
    local raw_ok, raw_value = pcall(rawget, parent, member)
    if not raw_ok then return result('unprobeable', tostring(raw_value), false) end
    local lookup_ok, lookup_value = pcall(function() return parent[member] end)
    local raw = type(raw_value)
    local lookup = lookup_ok and type(lookup_value) or 'lookup-error'
    local detail = 'raw=' .. raw .. '; lookup=' .. lookup
    if removed then return removed_result('member', detail, raw_value, raw, lookup) end
    return result('member', detail, raw == 'function')
end
local function create_object(owner)
    local frame = CreateFrame('Frame')
    local factories = {
        Object = function() return frame end,
        Frame = function() return frame end,
        FrameScriptObject = function() return frame end,
        ScriptRegion = function() return frame end,
        ScriptRegionResizing = function() return frame end,
        WorldFrame = function() return WorldFrame end,
        Scale = function() return frame:CreateAnimationGroup():CreateAnimation('Scale') end,
        Path = function() return frame:CreateAnimationGroup():CreateAnimation('Path') end,
        FlipBook = function()
            return frame:CreateTexture():CreateAnimationGroup():CreateAnimation('FlipBook')
        end,
        Region = function() return frame:CreateTexture() end,
        Line = function() return frame:CreateLine() end,
        Alpha = function() return frame:CreateAnimationGroup():CreateAnimation('Alpha') end,
        ColorCurveObject = function() return C_CurveUtil.CreateColorCurve() end,
        CurveObject = function() return C_CurveUtil.CreateCurve() end,
        -- CurveObjectBase is the interface implemented by the scalar curve.
        CurveObjectBase = function() return C_CurveUtil.CreateCurve() end,
        UnitHealPredictionCalculator = function() return CreateUnitHealPredictionCalculator() end,
        FontString = function() return frame:CreateFontString() end,
        -- FontInstance is an interface shared by fonts and text regions, not a frame type.
        FontInstance = function() return frame:CreateFontString() end,
        TextureBase = function() return frame:CreateTexture() end,
        Texture = function() return frame:CreateTexture() end,
        MaskTexture = function() return frame:CreateMaskTexture() end,
        VectorGraphics = function() return frame:CreateVectorGraphics() end,
        AnimationGroup = function() return frame:CreateAnimationGroup() end,
        Animation = function() return frame:CreateAnimationGroup():CreateAnimation() end,
        VertexColor = function()
            return frame:CreateTexture():CreateAnimationGroup():CreateAnimation('VertexColor')
        end,
        RadialProgress = function()
            local animation = frame:CreateAnimationGroup():CreateAnimation('RadialProgress')
            if animation:GetObjectType() ~= 'RadialProgress' then
                error('RadialProgress factory returned ' .. animation:GetObjectType())
            end
            return animation
        end,
        DurationTextBinding = function() return C_DurationUtil.CreateDurationTextBinding() end,
        SecondsFormatter = function() return C_StringUtil.CreateSecondsFormatter() end,
        DurationObject = function() return C_DurationUtil.CreateDuration() end,
        DurationManualClock = function() return C_DurationUtil.CreateManualClock() end,
        -- The manual clock is the only Lua-constructible DurationClock.
        DurationClock = function() return C_DurationUtil.CreateManualClock() end,
        Font = function()
            return rawget(_G, 'PublicationSweepFont') or CreateFont('PublicationSweepFont')
        end,
        ModelSceneActorBase = function() return CreateFrame('ModelScene'):CreateActor() end,
        ModelSceneActor = function() return CreateFrame('ModelScene'):CreateActor() end,
        AbbreviateConfig = function() return CreateAbbreviateConfig() end,
        AbbreviatedNumberFormatter = function()
            return C_StringUtil.CreateAbbreviatedNumberFormatter()
        end,
        -- NumericFormatter is the interface the abbreviated formatter implements.
        NumericFormatter = function() return C_StringUtil.CreateAbbreviatedNumberFormatter() end,
        NumericRuleFormatter = function() return C_StringUtil.CreateNumericRuleFormatter() end,
        HousingCatalogSearcher = function() return C_HousingCatalog.CreateCatalogSearcher() end,
    }
    local factory = factories[owner]
    if factory then return factory() end
    return CreateFrame(owner)
end
local function probe_object()
    local owner, method = string.match(symbol, '^([^:]+):([^:]+)$')
    if not owner then
        local created, object = pcall(create_object, symbol)
        local object_type = type(object)
        local published = created and (object_type == 'table' or object_type == 'userdata')
        local ok = published
        if removed then ok = not published end
        return result('object-kind', 'factory=' .. tostring(created) .. '; type=' .. object_type, ok)
    end
    local created, object = pcall(create_object, owner)
    if not created or object == nil then
        return result('unprobeable', 'factory: ' .. tostring(object), false)
    end
    local lookup = type(object[method])
    local ok = lookup == 'function'
    if removed then ok = lookup == 'nil' end
    return result('object-method', owner .. '; lookup=' .. lookup, ok)
end
local function probe_event()
    if string.find(symbol, '*', 1, true) then
        return result('unprobeable', 'wildcard occurrence is not a concrete event', false)
    end
    local frame = CreateFrame('Frame')
    local called, returned = pcall(frame.RegisterEvent, frame, symbol)
    local registered = called and frame:IsEventRegistered(symbol) == true
    if registered then frame:UnregisterEvent(symbol) end
    local unknown = not called and string.find(tostring(returned), 'Attempt to register unknown event', 1, true) ~= nil
    local ok = called and registered
    if removed then ok = unknown or (called and not registered) end
    local detail = 'pcall=' .. tostring(called) .. '; return=' .. tostring(returned)
    return result('event', detail .. '; registered=' .. tostring(registered), ok)
end
local function probe_cvar()
    local value = C_CVar.GetCVar(symbol)
    local default = C_CVar.GetCVarDefault(symbol)
    local ok = value ~= nil and default ~= nil
    if removed then ok = value == nil and default == nil end
    return result('cvar', 'value/default queried', ok, value, default)
end
local function probe_command()
    local present = false
    for _, record in ipairs(C_Console.GetAllCommands()) do
        if record.command == symbol and record.commandType == Enum.ConsoleCommandType.Command then
            present = true
            break
        end
    end
    return result('command', 'Command record present=' .. tostring(present), present ~= removed)
end
local function classify()
    if entryKind == 'widget-script' then
        -- Older handler links include an owner; unqualified movie handlers retain their owner.
        local owner, script = string.match(symbol, '^([^:]+):([^:]+)$')
        if not owner then owner, script = string.match(symbol, '^(%S+)%s+(%S+)$') end
        -- Literal OnTooltip handlers belong to GameTooltip, not MovieFrame.
        owner = owner or (string.match(symbol, '^OnTooltip') and 'GameTooltip' or 'MovieFrame')
        script = script or symbol
        local object = create_object(owner)
        local supported = object:HasScript(script)
        return result('widget-script', owner .. ' HasScript=' .. tostring(supported), supported ~= removed)
    end
    if entryKind == 'click-modifier' then
        local value = GetModifiedClick(symbol)
        local unknown = GetModifiedClick('PUBLICATION_SWEEP_UNKNOWN_CLICK_MODIFIER')
        -- The temporary input defaults return NONE for unknown actions too.
        -- No catalog discriminates existence; this is not a console command.
        return result('unprobeable', 'click-modifier value=' .. tostring(value)
            .. '; unknown=' .. tostring(unknown) .. '; no discriminating catalog', false)
    end
    if entryKind == 'command' then return probe_command() end
    if section == 'events' then return probe_event() end
    if section == 'cvars' then return probe_cvar() end
    if section == 'widgets' or section == 'scriptobjects' then return probe_object() end
    return probe_publication()
end
local called, kind, detail, ok, value, default = pcall(classify)
if not called then return result('probe-error', tostring(kind), false) end
return kind, detail, ok, value, default
"#;

// Lua decimal byte escapes avoid JSON's incompatible \u escapes and script injection.
fn quote_lua(value: &str) -> String {
    let escaped: String = value.bytes().map(|byte| format!("\\{byte:03}")).collect();
    format!("\"{escaped}\"")
}

pub(crate) fn probe_entry(
    env: &WowLuaEnv,
    entry: &Entry,
    removed: bool,
    aliases: &BTreeMap<String, (String, String)>,
) -> ProbeResult {
    let (target, source) = aliases.get(&entry.symbol).cloned().unwrap_or_default();
    let code = format!(
        "return (function(...) {CLASSIFIER} end)({}, {}, {}, {}, {}, {})",
        quote_lua(&entry.section),
        quote_lua(&entry.symbol),
        removed,
        quote_lua(entry.kind.as_deref().unwrap_or("")),
        quote_lua(&target),
        quote_lua(&source),
    );
    env.eval::<ProbeResult>(&code)
        .unwrap_or_else(|error| ("probe-error".into(), error.to_string(), false, None, None))
}
