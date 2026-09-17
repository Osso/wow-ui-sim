local root = assert(arg[1], "addon directory required")
local passed, failed = 0, 0
local fields = { "baseSourceID", "baseVisualID", "appliedSourceID", "appliedVisualID",
    "pendingSourceID", "pendingVisualID", "hasUndo", "isHideVisual", "itemSubclass" }
local function pack(...) return { n = select("#", ...), ... } end
local function opaque() return setmetatable({}, { __tostring = function() error("stringified") end }) end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print("PASS " .. name)
    else failed = failed + 1; print("FAIL " .. name .. ": " .. tostring(err)) end
end
local function setup(factory, query, sourceQuery)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue, canaccessvalue = function() return false end, function() return true end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    Enum = { TransmogType = { Appearance = 23.5 } }
    TransmogUtil, C_Transmog = { CreateTransmogLocation = factory }, { GetSlotVisualInfo = query }
    C_TransmogCollection = { GetAppearanceSourceInfo = sourceQuery }
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("transmog-slot-visual-info " .. (label or "sample"))
    local db = assert(ApiContractProbeDB, "manual mode absent")
    return assert(db.captures[#db.captures].transmogSlotVisualInfo, "manual mode absent")
end
-- Distinct from the published Appearance value: data is validated, never remapped.
local function data() return { slotID = 71.5, type = 31.5, modification = 9.5 } end
local function location(value) return { GetData = function() return value end } end
local function visual()
    local t = {}; for i, name in ipairs(fields) do t[name] = i + 0.5 end
    t.hasUndo, t.isHideVisual = false, true
    return t
end

test("four original factory arguments, method receivers and unmodified data identities", function()
    local factoryCalls, getCalls, queryCalls, originals = 0, 0, 0, {}
    setup(function(...)
        local args = pack(...); factoryCalls = factoryCalls + 1
        assert(args.n == 3, "factory is a dot call, not a method")
        assert(args[1] == (factoryCalls <= 2 and "HEADSLOT" or "SHOULDERSLOT"))
        assert(args[2] == 23.5 and args[3] == (factoryCalls % 2 == 0))
        local d, obj = data(), {}; originals[factoryCalls] = d
        obj.GetData = function(...)
            assert(select("#", ...) == 1 and rawequal((...), obj)); getCalls = getCalls + 1
            return d, nil, "data-tail"
        end
        return obj, nil, "factory-tail"
    end, function(...)
        queryCalls = queryCalls + 1
        assert(select("#", ...) == 1 and rawequal((...), originals[queryCalls]))
        return visual(), nil, "visual-tail"
    end)
    local r = capture()
    assert(factoryCalls == 4 and getCalls == 4 and queryCalls == 4 and #r.cases == 4)
    for _, row in ipairs(r.cases) do
        assert(row.factory.n == 3 and row.factory.values[2].kind == "nil")
        assert(row.data.n == 3 and row.data.values[2].kind == "nil")
        assert(row.visual.n == 3 and row.visual.values[2].kind == "nil")
        local count = 0; for key in pairs(row.visual.object.fields) do count = count + 1; assert(visual()[key] ~= nil) end
        assert(count == 9 and row.visual.object.fields.hasUndo.value == false)
    end
end)

test("missing invalid enum and APIs fail independently without numeric fallback", function()
    local calls = 0
    for _, value in ipairs({ false, "23", {}, math.huge, 0/0 }) do
        setup(function() calls = calls + 1 end, function() calls = calls + 1 end)
        Enum.TransmogType.Appearance = value
        assert(capture().cases[1].factory.status ~= "observed")
    end
    assert(calls == 0)
    setup(nil, nil); assert(capture().cases[1].factory.status ~= "observed")
    setup(function() return location(data()) end, nil)
    assert(capture().cases[1].visual.status == "missing-api")
    setup(function() return location(data()) end, function() end)
    C_Transmog = setmetatable({}, { __index = function() error(opaque()) end })
    assert(capture().cases[1].visual.status == "field-error")
end)

test("factory input revocation during lookup and function guards blocks every case", function()
    for _, input in ipairs({ "descriptor", "type", "secondary" }) do
        for _, phase in ipairs({ "lookup", "secret", "access" }) do
            for selected = 1, 4 do
                local lookups, calls, denied = 0, 0, false
                local fn = function() assert(not denied); calls = calls + 1; return location(data()) end
                setup(fn, function() return visual() end)
                TransmogUtil = setmetatable({}, { __index = function(_, key)
                    assert(key == "CreateTransmogLocation"); lookups = lookups + 1
                    if lookups == selected and phase == "lookup" then denied = true end
                    return fn
                end })
                local target = input == "descriptor" and (selected <= 2 and "HEADSLOT" or "SHOULDERSLOT")
                    or input == "type" and 23.5 or selected % 2 == 0
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, fn) and lookups == selected then denied = true end
                    return false
                end
                canaccessvalue = function(v)
                    if phase == "access" and rawequal(v, fn) and lookups == selected then denied = true end
                    return not (denied and rawequal(v, target))
                end
                local r = capture()
                assert(r.cases[selected].factory.status ~= "observed" and calls < 4)
            end
        end
    end
end)

test("GetData receiver revoked by method lookup or either guard is never invoked", function()
    for _, phase in ipairs({ "lookup", "secret", "access" }) do
        local current, revoked, calls = nil, false, 0
        local fn = function() calls = calls + 1; error("revoked method invoked") end
        setup(function()
            revoked = false
            current = setmetatable({}, { __index = function(_, key)
                assert(key == "GetData"); if phase == "lookup" then revoked = true end; return fn
            end })
            return current
        end, function() error("query") end)
        issecretvalue = function(v) if phase == "secret" and rawequal(v, fn) then revoked = true end; return false end
        canaccessvalue = function(v)
            if phase == "access" and rawequal(v, fn) then revoked = true end
            return not (revoked and rawequal(v, current))
        end
        assert(capture().cases[1].data.status ~= "observed" and calls == 0)
    end
end)

test("original data and all three fields rechecked after API lookup and guards", function()
    for _, targetKey in ipairs({ "object", "slotID", "type", "modification" }) do
        for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
            for selected = 1, 4 do
                local index, current, armed, calls = 0, nil, false, 0
                local fn = function(d) assert(not armed); calls = calls + 1; return visual() end
                setup(function()
                    index = index + 1; current = data(); armed = false
                    return location(current)
                end, fn)
                local ns = setmetatable({}, { __index = function(_, key)
                    assert(key == "GetSlotVisualInfo")
                    if phase == "lookup" and index == selected then armed = true end
                    return fn
                end }); C_Transmog = ns
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, fn) and index == selected then armed = true end
                    return false
                end
                canaccessvalue = function(v)
                    if index == selected and ((phase == "namespace" and rawequal(v, ns)) or
                        (phase == "access" and rawequal(v, fn))) then armed = true end
                    local target = targetKey == "object" and current or current and current[targetKey]
                    return not (armed and rawequal(v, target))
                end
                local r = capture()
                assert(calls == 3 and r.cases[selected].visual.status ~= "observed",
                    targetKey .. "/" .. phase .. "/" .. selected .. " calls=" .. calls)
            end
        end
    end
end)

test("data field lookup errors invalid fields and later-field revocation block forwarding", function()
    for _, key in ipairs({ "slotID", "type", "modification" }) do
        for _, valueCase in ipairs({ {}, { false }, { "bad" }, { {} }, { math.huge }, { 0/0 } }) do
            local calls = 0
            setup(function() local d = data(); d[key] = valueCase[1]; return location(d) end,
                function() calls = calls + 1 end)
            assert(capture().cases[1].visual.status ~= "observed" and calls == 0)
        end
    end
    local calls, revoked = 0, false
    setup(function()
        revoked = false
        return location(setmetatable({}, { __index = function(_, key)
            if key == "modification" then revoked = true end
            return data()[key]
        end }))
    end, function() calls = calls + 1 end)
    canaccessvalue = function(v) return not (revoked and v == 71.5) end
    assert(capture().cases[1].visual.status ~= "observed" and calls == 0)
    setup(function() return location(setmetatable({}, { __index = function() error(opaque()) end })) end,
        function() calls = calls + 1 end)
    assert(capture().cases[1].visual.status == "field-error" and calls == 0)
end)

test("visual receiver rechecked at every field and values remain guarded", function()
    for selected = 1, 9 do
        local object, denied, reads = nil, false, 0
        setup(function() return location(data()) end, function()
            denied, reads = false, 0
            object = setmetatable({}, { __index = function(_, key)
                assert(not denied); reads = reads + 1
                if reads == selected then denied = true end
                return visual()[key]
            end }); return object
        end)
        canaccessvalue = function(v) return not (denied and rawequal(v, object)) end
        local r = capture()
        assert(reads == selected and r.cases[4].visual.object.fields[fields[selected]].status == "observed")
        if selected < 9 then assert(r.cases[4].visual.object.fields[fields[selected + 1]].status == "field-error") end
    end
    setup(function() return location(data()) end, function() return visual() end)
    issecretvalue = function(v) return v == 1.5 end
    assert(capture().cases[1].visual.object.fields.baseSourceID.status == "restricted")
end)

test("factory GetData and visual failures preserve arity and independent peers", function()
    local index, queries = 0, 0
    setup(function()
        index = index + 1
        if index == 1 then error(opaque()) end
        if index == 2 then return nil, location(data()) end
        if index == 3 then return { GetData = function() error(opaque()) end } end
        return location(data())
    end, function() queries = queries + 1; return nil, false, nil end)
    local r = capture()
    assert(r.cases[1].factory.status == "call-error" and r.cases[2].factory.n == 2)
    assert(r.cases[3].data.status == "call-error" and queries == 1)
    assert(r.cases[4].visual.n == 3 and r.cases[4].visual.values[3].kind == "nil")
    index = 0
    setup(function() index = index + 1; return location(data()) end, function()
        if index == 1 then return end
        if index == 2 then error(opaque()) end
        local t = {}; for i = 1, 18 do t[i] = string.rep("x", 300) end
        return unpack(t, 1, 18)
    end)
    r = capture()
    assert(r.cases[1].visual.n == 0 and r.cases[2].visual.status == "call-error")
    assert(r.cases[3].visual.n == 18 and r.cases[3].visual.truncated)
    assert(#r.cases[3].visual.values == 16 and #r.cases[3].visual.values[16].value == 256)
end)

test("serialization revocation blocks later stages and retained objects are collectible", function()
    local current, denied, queries = nil, false, 0
    setup(function()
        denied = false; current = data()
        return { GetData = function() return current, "revoke-data" end }
    end, function() queries = queries + 1 end)
    canaccessvalue = function(v)
        if v == "revoke-data" then denied = true end
        return not (denied and rawequal(v, current))
    end
    assert(capture().cases[1].visual.status ~= "observed" and queries == 0)
    local weak, count = setmetatable({}, { __mode = "v" }), 0
    local function retain(v) count = count + 1; weak[count] = v; return v end
    setup(function() return retain(location(retain(data()))) end, function() return retain(visual()) end)
    capture(); collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

test("ten snapshots twelve recorder calls labels and manual exclusion", function()
    local calls = 0
    setup(function()
        calls = calls + 1
        return { GetData = function() calls = calls + 1; return data() end }
    end, function() calls = calls + 1; return visual() end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls == 120 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    setup(function() error("factory from all") end, function() error("query from all") end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].transmogSlotVisualInfo == nil)
    canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("transmog-slot-visual-info")
    assert(ApiContractProbeDB.captures[2].status == "missing-access-api")
end)

-- Execute unmodified source sections containing the real factory and GetData, not a fixture reimplementation.
test("bounded pinned vendor factory Set and GetData chain", function()
    local sourcePath = "/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_TransmogShared/Blizzard_TransmogShared.lua"
    local file = assert(io.open(sourcePath)); local source = file:read("*a"); file:close()
    source = source:gsub("\r\n", "\n")
    local function section(first, last)
        local a = assert(source:find(first, 1, true)); local b = assert(source:find(last, a + #first, true))
        return source:sub(a, b - 1)
    end
    local queries = 0
    setup(nil, function(d)
        queries = queries + 1
        assert(d.slotID == (queries <= 2 and 1 or 3) and d.type == 23.5)
        assert(d.modification == (queries % 2 == 0 and 41 or 37))
        return visual()
    end)
    Enum.TransmogModification = { Main = 37, Secondary = 41 }
    C_PaperDollInfo = { GetInventorySlotInfo = function(name)
        assert(name == "HEADSLOT" or name == "SHOULDERSLOT"); return name == "HEADSLOT" and 1 or 3
    end }
    C_TransmogOutfitInfo = {
        GetTransmogOutfitSlotFromInventorySlot = function(index) assert(index == 0 or index == 2); return index + 10 end,
        GetLinkedSlotInfo = function(slot) return { secondarySlotInfo = { slot = slot + 100 } } end,
    }
    CreateFromMixins = function(mixin) local o = {}; for k, v in pairs(mixin) do o[k] = v end; return o end
    assert(loadstring(section("TransmogLocationMixin = {};", "function TransmogLocationMixin:IsAppearance"), "@vendor-mixin"))()
    assert(loadstring("local SLOT_ID_TO_NAME = {};\n" .. section("function TransmogUtil.GetSlotID(",
        "function TransmogUtil.GetTransmogLocation("), "@vendor-factory"))()
    assert(loadstring(section("function TransmogLocationMixin:GetData()", "-- This will indirectly populate"), "@vendor-data"))()
    assert(capture().cases[4].visual.status == "observed" and queries == 4)
end)

local appearanceFields = { "category", "itemAppearanceID", "canHaveIllusion", "icon", "isCollected",
    "itemLink", "transmoglink", "sourceType", "itemSubclass", "ignoreModelAttachmentChecksForIllusion" }
local sourceKeys = { "baseSourceID", "appliedSourceID" }
local function appearance()
    local value = {}
    for i, key in ipairs(appearanceFields) do value[key] = i + 0.25 end
    value.canHaveIllusion, value.isCollected = false, true
    value.itemLink, value.transmoglink = "item:fixture", "transmog:fixture"
    value.ignoreModelAttachmentChecksForIllusion = false
    return value
end
local function sourceID(position)
    return math.floor((position - 1) / 2) * 100 + (position % 2 == 1 and 1.25 or 2.25)
end
local function sourceSetup(query)
    local case = 0
    setup(function() return location(data()) end, function()
        case = case + 1
        local value = visual()
        value.baseSourceID, value.appliedSourceID = sourceID(case * 2 - 1), sourceID(case * 2)
        return value
    end, query)
end
local function sourceResult(result, position)
    local row = result.cases[math.floor((position - 1) / 2) + 1]
    return assert(row.sources, "source followups absent")[sourceKeys[position % 2 == 1 and 1 or 2]]
end

test("appearance followups reread original dynamic source IDs and preserve old visual fields", function()
    local calls, expected = 0, {}
    setup(function() return location(data()) end, function()
        local reads, base = {}, #expected * 100
        expected[#expected + 1], expected[#expected + 2] = base + 71.25, base + 72.25
        return setmetatable({}, { __index = function(_, key)
            if key == "baseSourceID" or key == "appliedSourceID" then
                reads[key] = (reads[key] or 0) + 1
                if reads[key] == 1 then return key == "baseSourceID" and 1.5 or 3.5 end
                return base + (key == "baseSourceID" and 71.25 or 72.25)
            end
            return visual()[key]
        end })
    end, function(...)
        calls = calls + 1
        assert(select("#", ...) == 1 and (...) == expected[calls], "must forward original reread ID")
        return appearance(), nil, "source-tail"
    end)
    local result = capture()
    assert(calls == 8, "eight independent source queries required")
    for position = 1, 8 do
        local row, observation = result.cases[math.floor((position - 1) / 2) + 1], sourceResult(result, position)
        assert(row.visual.object.fields.baseSourceID.value == 1.5)
        assert(row.visual.object.fields.appliedSourceID.value == 3.5)
        assert(observation.n == 3 and observation.values[2].kind == "nil")
        local count = 0
        for key, field in pairs(observation.object.fields) do
            count = count + 1; assert(field.value == appearance()[key])
        end
        assert(count == 10 and observation.object.fields.ignoreModelAttachmentChecksForIllusion.value == false)
    end
    calls = 0
    setup(function() return location(data()) end, function()
        local value = visual(); value.baseSourceID, value.appliedSourceID = 0, 0; return value
    end, function(id) assert(id == 0); calls = calls + 1; return nil end)
    capture(); assert(calls == 8, "zero/duplicate IDs are observed, not remapped or deduplicated")
end)

test("source queries reject absent invalid and restricted original IDs without suppressing peers", function()
    for _, key in ipairs(sourceKeys) do
        for _, values in ipairs({ {}, { false }, { "123" }, { {} }, { math.huge }, { -math.huge }, { 0/0 } }) do
            local calls = 0
            setup(function() return location(data()) end, function()
                local value = visual(); value[key] = values[1]; return value
            end, function() calls = calls + 1; return appearance() end)
            local result = capture()
            assert(result.cases[1].sources[key].status ~= "observed" and calls == 4)
        end
    end
    local calls = 0
    sourceSetup(function(id) assert(id ~= sourceID(1)); calls = calls + 1 end)
    issecretvalue = function(value) return value == sourceID(1) end
    assert(sourceResult(capture(), 1).status == "restricted-input" and calls == 7)
    setup(function() return location(data()) end, function() return nil, visual() end,
        function() error("second visual return forwarded") end)
    assert(sourceResult(capture(), 1).status == "unavailable-input")
    sourceSetup(nil)
    assert(sourceResult(capture(), 1).status == "missing-api")
    C_TransmogCollection = setmetatable({}, { __index = function() error(opaque()) end })
    assert(sourceResult(capture(), 1).status == "field-error")
end)

test("every original source ID is rechecked after namespace lookup and both function guards", function()
    for _, phase in ipairs({ "namespace-secret", "namespace-access", "lookup", "function-secret", "function-access" }) do
        for selected = 1, 8 do
            local attempt, calls, revoked = 0, 0, false
            local fn = function(id)
                assert(not (revoked and id == sourceID(selected)), "revoked ID forwarded")
                calls = calls + 1; return appearance()
            end
            sourceSetup(fn)
            local ns = setmetatable({}, { __index = function(_, key)
                assert(key == "GetAppearanceSourceInfo", "fallback query")
                if phase == "lookup" and attempt == selected then revoked = true end
                return fn
            end }); C_TransmogCollection = ns
            issecretvalue = function(value)
                if rawequal(value, ns) then
                    attempt = attempt + 1
                    if phase == "namespace-secret" and attempt == selected then revoked = true end
                end
                if phase == "function-secret" and rawequal(value, fn) and attempt == selected then revoked = true end
                return false
            end
            canaccessvalue = function(value)
                if attempt == selected and ((phase == "namespace-access" and rawequal(value, ns)) or
                    (phase == "function-access" and rawequal(value, fn))) then revoked = true end
                return not (revoked and rawequal(value, sourceID(selected)))
            end
            local result = capture()
            assert(sourceResult(result, selected).status == "restricted-input" and calls == 7,
                phase .. "/" .. selected .. " calls=" .. calls)
        end
    end
end)

test("visual serialization and earlier source outputs can revoke later followups", function()
    for _, revokeReceiver in ipairs({ false, true }) do
        local object, revoked, calls = nil, false, 0
        setup(function() return location(data()) end, function()
            revoked = false; object = visual(); object.itemSubclass = "revoke-original"; return object
        end, function(id) assert(id ~= 1.5); calls = calls + 1 end)
        canaccessvalue = function(value)
            if value == "revoke-original" then revoked = true end
            return not (revoked and rawequal(value, revokeReceiver and object or 1.5))
        end
        local result = capture()
        assert(sourceResult(result, 1).status ~= "observed")
        assert(calls == (revokeReceiver and 0 or 4))
    end
    local revoked, calls = false, 0
    sourceSetup(function(id)
        assert(not (revoked and id == sourceID(2)))
        calls = calls + 1
        if id == sourceID(1) then return nil, "revoke-applied" end
        return appearance()
    end)
    canaccessvalue = function(value)
        if value == "revoke-applied" then revoked = true end
        return not (revoked and rawequal(value, sourceID(2)))
    end
    assert(sourceResult(capture(), 2).status == "restricted-input" and calls == 7)
end)

test("appearance receiver and all ten fields stay guarded after every observation", function()
    for _, guard in ipairs({ "secret", "access" }) do
        for selected = 1, 10 do
            local object, revoked, reads = nil, false, 0
            sourceSetup(function()
                revoked, reads = false, 0
                object = setmetatable({}, { __index = function(_, key)
                    assert(not revoked, "revoked appearance receiver indexed")
                    reads = reads + 1
                    if reads == selected then revoked = true end
                    return appearance()[key]
                end })
                return object
            end)
            issecretvalue = function(value) return guard == "secret" and revoked and rawequal(value, object) end
            canaccessvalue = function(value) return not (guard == "access" and revoked and rawequal(value, object)) end
            local result = capture()
            local last = sourceResult(result, 8).object.fields
            assert(reads == selected and last[appearanceFields[selected]].status == "observed")
            if selected < 10 then assert(last[appearanceFields[selected + 1]].status == "field-error") end
        end
    end
    local secret = opaque()
    sourceSetup(function()
        local value = appearance(); value.sourceType = nil
        value.itemLink = string.rep("x", 300); value.icon = math.huge
        value.ignoreModelAttachmentChecksForIllusion = secret
        return value
    end)
    issecretvalue = function(value) return rawequal(value, secret) end
    local fieldsResult = sourceResult(capture(), 1).object.fields
    assert(fieldsResult.sourceType.kind == "nil" and fieldsResult.icon.status == "nonfinite")
    assert(fieldsResult.ignoreModelAttachmentChecksForIllusion.status == "restricted")
    assert(#fieldsResult.itemLink.value == 256 and fieldsResult.itemLink.truncated)
    sourceSetup(function() return setmetatable({}, { __index = function() error(opaque()) end }) end)
    assert(sourceResult(capture(), 1).object.fields.category.status == "field-error")
end)

test("source errors exact nil arity first-object-only bounds and collectibility", function()
    local calls = 0
    sourceSetup(function()
        calls = calls + 1
        if calls == 1 then return end
        if calls == 2 then error(opaque()) end
        if calls == 3 then return nil, appearance(), nil end
        local values = { appearance() }
        for i = 2, 18 do values[i] = string.rep("s", 300) end
        return unpack(values, 1, 18)
    end)
    local result = capture()
    assert(calls == 8 and sourceResult(result, 1).n == 0 and sourceResult(result, 2).status == "call-error")
    assert(sourceResult(result, 3).n == 3 and sourceResult(result, 3).object.kind == "nil")
    assert(sourceResult(result, 3).values[3].kind == "nil")
    local bounded = sourceResult(result, 8)
    assert(bounded.n == 18 and bounded.truncated and #bounded.values == 16)
    assert(#bounded.values[16].value == 256)
    local weak, n = setmetatable({}, { __mode = "v" }), 0
    local function track(value) n = n + 1; weak[n] = value; return value end
    setup(function() return track(location(track(data()))) end, function() return track(visual()) end,
        function() return track(appearance()), track(opaque()) end)
    assert(sourceResult(capture(), 1).status == "observed")
    collectgarbage("collect"); collectgarbage("collect"); assert(next(weak) == nil)
end)

test("source extension caps twenty recorder calls and remains absent from all", function()
    local calls, sources = 0, 0
    setup(function()
        calls = calls + 1; return { GetData = function() calls = calls + 1; return data() end }
    end, function() calls = calls + 1; return visual() end, function()
        calls = calls + 1; sources = sources + 1; return appearance()
    end)
    for i = 1, 11 do capture(string.rep("l", 200)) end
    assert(calls == 200 and sources == 80, "20 recorder invocations per snapshot")
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    setup(function() error("factory from all") end, function() error("visual from all") end,
        function() error("appearance from all") end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].transmogSlotVisualInfo == nil)
end)

print(string.format("transmog-slot-visual-info: %d passed, %d failed", passed, failed))
if failed > 0 then os.exit(1) end
