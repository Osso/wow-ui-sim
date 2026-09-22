-- Read-only interaction proposal for cached C_Everywhere 8912546.
-- Scope: prove the library is usable after explicit LibStub composition and
-- exercise the two behaviors covered by its packaged Tests.lua.
-- This is not full addon conformance or native Forever proof.

local C = assert(LibStub('C_Everywhere'), 'C_Everywhere library missing')
local results = {}

local function record(name, ok, detail)
    results[#results + 1] = {name = name, ok = ok, detail = detail}
    assert(ok, name .. ': ' .. tostring(detail))
end

-- Tests.lua:NamespaceTranslation.  C.CVar is created lazily and should resolve
-- the real C_CVar namespace (or the legacy global when C_CVar is absent).
local nativeGetCVar = C_CVar and C_CVar.GetCVar or GetCVar
local translatedGetCVar = C.CVar.GetCVar
record('namespace_translation', translatedGetCVar == nativeGetCVar,
    'C.CVar.GetCVar identity differs from the selected native/legacy function')

-- Use a real CVar read to prove the translated function is callable and returns
-- the same observable value as the native/legacy function.
local cvarName = 'useUiScale'
local nativeValue = nativeGetCVar(cvarName)
local translatedValue = translatedGetCVar(cvarName)
record('namespace_call', nativeValue == translatedValue,
    'CVar values differ: native=' .. tostring(nativeValue) ..
    ', translated=' .. tostring(translatedValue))

-- Tests.lua:OutputPacking.  For each inspected slot, compare the legacy/raw
-- return shape with C.Container.GetContainerItemInfo's packed table shape.
-- Empty slots must remain nil; populated slots must preserve iconFileID and,
-- when supplied by the raw API, stackCount.
A_Admin.ClearBags()
A_Admin.AddBagItem(0, 1, 6948, 3)
local rawContainerInfo = C_Container and C_Container.GetContainerItemInfo or GetContainerItemInfo
local packedContainerInfo = C.Container.GetContainerItemInfo
for slot = 1, 6 do
    local first, second = rawContainerInfo(0, slot)
    local packed = packedContainerInfo(0, slot)
    if not first then
        record('container_empty_' .. slot, packed == nil,
            'expected nil for empty slot, got ' .. tostring(packed))
    elseif type(first) == 'table' then
        record('container_table_' .. slot,
            type(packed) == 'table' and packed.itemID == 6948 and packed.stackCount == 3
                and packed.itemID == first.itemID and packed.stackCount == first.stackCount
                and packed.iconFileID == first.iconFileID,
            'populated item fields differ')
    elseif second then
        record('container_packed_' .. slot,
            type(packed) == 'table' and packed.iconFileID == first and packed.stackCount == second,
            'packed result did not preserve first two raw values')
    else
        record('container_scalar_' .. slot, packed == first,
            'single raw value changed: raw=' .. tostring(first) ..
            ', packed=' .. tostring(packed))
    end
end

A_Admin.RemoveBagItem(0, 1)
record('container_removed', packedContainerInfo(0, 1) == nil, 'removed item remained visible')
print('C_EVERYWHERE_INTERACTION_PASS ' .. #results .. ' checks')
