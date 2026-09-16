local root = assert(arg[1])
local passed, excluded = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() excluded = excluded + 1; error("excluded API") end
local function setup(list, info, validate, maximum)
    ApiContractProbeDB, SlashCmdList, excluded = nil, {}, 0
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_TransmogCollection = { GetCustomSets = list, GetCustomSetInfo = info,
        IsValidCustomSetName = validate, GetNumMaxCustomSets = maximum,
        NewCustomSet = forbidden, ModifyCustomSet = forbidden, RenameCustomSet = forbidden,
        DeleteCustomSet = forbidden, GetCustomSetItemTransmogInfoList = forbidden,
        GetCustomSetHyperlinkFromItemTransmogInfoList = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("custom-set-names " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "custom-set-names mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].customSetNames)
end
local function rows(result) return result.sets.values[1].entries end
local function test(name, fn)
    fn(); assert(excluded == 0); passed = passed + 1; print("PASS " .. name)
end

test("original fractional IDs and untruncated names with independent maxima", function()
    local infos, names, maxima = 0, 0, 0
    local original = string.rep("long name ", 40)
    setup(function(...) assert(select("#", ...) == 0); return { 1.25, -3.5 } end,
        function(...) infos = infos + 1; assert(select("#", ...) == 1)
            assert((...) == (infos == 1 and 1.25 or -3.5)); return original, 123, nil end,
        function(...) names = names + 1; assert(select("#", ...) == 1 and (...) == original)
            return names == 1, nil end,
        function(...) maxima = maxima + 1; assert(select("#", ...) == 0); return maxima end)
    local r = capture(); local e = rows(r)
    assert(maxima == 2 and infos == 2 and names == 2 and #e == 4)
    assert(r.maximum[1].values[1].value == 1 and r.maximum[2].values[1].value == 2)
    assert(e[1].id.value == 1.25 and e[2].id.value == -3.5)
    assert(e[1].info.n == 3 and e[1].info.values[2].value == 123 and e[1].info.values[3].kind == "nil")
    assert(#e[1].info.values[1].value == 256 and e[1].validation.n == 2)
    assert(e[1].validation.values[1].value and e[2].validation.values[1].value == false)
end)

test("missing APIs and namespace failures remain explicit", function()
    setup(nil, nil, nil, nil)
    local r = capture(); assert(r.maximum[1].status == "missing-api" and r.sets.status == "missing-api")
    C_TransmogCollection = secret
    r = capture(); assert(r.maximum[2].status == "field-error" and r.sets.status == "field-error")
    C_TransmogCollection = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().sets.status == "field-error")
    for _, key in ipairs({ "GetCustomSetInfo", "IsValidCustomSetName" }) do
        setup(function() return { 1 } end, function() return "name" end, function() return true end)
        C_TransmogCollection[key] = secret
        local row = rows(capture())[1]
        assert((key == "GetCustomSetInfo" and row.info or row.validation).status == "missing-api")
    end
end)

test("invalid and secret IDs or names skip only their own chain", function()
    for _, bad in ipairs({ false, "3", math.huge, -math.huge, 0/0, secret, {} }) do
        local calls = 0
        setup(function() return { bad, 2 } end, function(id) assert(id == 2); calls = calls + 1; return "ok" end,
            function(name) assert(name == "ok"); return true end)
        local e = rows(capture()); assert(calls == 1 and e[2].validation.status == "observed")
        assert(e[1].info.status ~= "observed")
    end
    for _, bad in ipairs({ false, 7, secret, {} }) do
        local calls = 0
        setup(function() return { 1, 2 } end, function(id) if id == 1 then return bad else return "" end end,
            function(name) assert(name == ""); calls = calls + 1 end)
        local e = rows(capture()); assert(calls == 1 and e[1].validation.status ~= "observed")
    end
end)

test("zero returns nil holes and errors preserve exact tuples", function()
    setup(function() end, forbidden, forbidden, function() error(secret) end)
    local r = capture(); assert(r.sets.n == 0 and r.maximum[1].status == "call-error")
    setup(function() return nil, { 1 }, nil end, forbidden, forbidden)
    r = capture(); assert(r.sets.n == 3 and r.sets.values[1].kind == "nil" and r.sets.values[2].entries == nil)
    setup(function() return { 1, 2, 3, 4 } end, function(id)
        if id == 1 then error(secret) elseif id == 2 then return nil, "not a name", nil
        elseif id == 3 then return "three", nil, nil end
    end, function(name) assert(name == "three"); return nil, false, nil end)
    local e = rows(capture())
    assert(e[1].info.status == "call-error" and e[2].info.n == 3 and e[4].info.n == 0)
    assert(e[3].validation.n == 3 and e[3].validation.values[2].value == false)
    setup(function() return { 1, 2 } end, function(id) return tostring(id) end,
        function(name) if name == "1" then error(secret) end end)
    e = rows(capture()); assert(e[1].validation.status == "call-error" and e[2].validation.n == 0)
end)

test("every original ID and name is rechecked after API lookup and function guards", function()
    for _, target in ipairs({ "GetCustomSetInfo", "IsValidCustomSetName" }) do
        for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
            for selected = 1, 4 do
                local revoked, calls = false, 0
                local value = target == "GetCustomSetInfo" and selected + 0.25 or "name" .. selected
                local fn = function(input) assert(not (revoked and input == value)); calls = calls + 1
                    if target == "GetCustomSetInfo" then return "name" end
                end
                setup(function() return { 1.25, 2.25, 3.25, 4.25 } end,
                    function(id) return "name" .. math.floor(id) end, function() end)
                local namespace = C_TransmogCollection
                namespace[target] = nil
                setmetatable(namespace, { __index = function(_, key)
                    if key ~= target then return nil end
                    if phase == "lookup" then revoked = true end
                    return fn
                end })
                issecretvalue = function(v)
                    if phase == "secret" and rawequal(v, fn) then revoked = true end
                    return rawequal(v, secret)
                end
                canaccessvalue = function(v)
                    if (phase == "namespace" and rawequal(v, namespace)) or
                        (phase == "access" and rawequal(v, fn)) then revoked = true end
                    return not rawequal(v, secret) and not (revoked and v == value)
                end
                local e = rows(capture())
                assert(calls == 3)
                assert((target == "GetCustomSetInfo" and e[selected].info or e[selected].validation).status == "restricted-input")
            end
        end
    end
end)

test("list receiver access is checked at each of four positions", function()
    for selected = 1, 4 do
        local revoked, reads, calls = false, 0, 0
        local list = setmetatable({}, { __index = function(_, index)
            assert(not revoked and index <= 4); reads = reads + 1
            if index == selected then revoked = true end
            return index
        end, __len = forbidden, __pairs = forbidden })
        setup(function() return list end, function() calls = calls + 1; return "name" end, function() end)
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        local e = rows(capture()); assert(reads == selected and calls == selected)
        if selected < 4 then assert(e[selected + 1].status == "field-error") end
    end
end)

test("serialization can revoke original ID or name before downstream calls", function()
    for _, phase in ipairs({ "id", "name" }) do
        local reads, blocked, validations = 0, false, 0
        setup(function() return { 1 } end, function(id) assert(phase ~= "id"); return "original", 123 end,
            function() validations = validations + 1 end)
        canaccessvalue = function(v)
            if phase == "id" and v == 1 then reads = reads + 1; if reads > 1 then return false end end
            if phase == "name" and v == 123 then blocked = true end
            return not rawequal(v, secret) and not (blocked and v == "original")
        end
        local e = rows(capture()); assert(validations == 0)
        assert((phase == "id" and e[1].info or e[1].validation).status == "restricted-input")
    end
end)

test("restricted list and opaque first names are never traversed", function()
    for _, list in ipairs({ secret, false, 3, "list", newproxy(true) }) do
        setup(function() return list end, forbidden, forbidden)
        assert(capture().sets.values[1].entries == nil)
    end
    local opaque = setmetatable({}, { __index = forbidden, __tostring = forbidden })
    setup(function() return { 1 } end, function() return opaque end, forbidden)
    assert(rows(capture())[1].validation.status == "unavailable-input")
end)

test("eleven calls tuple strings labels and shared snapshot limits", function()
    local calls = 0
    local many = {}; for i = 1, 20 do many[i] = string.rep("v", 300) end
    setup(function() calls = calls + 1; return { 1, 2, 3, 4, 5, 6 }, unpack(many) end,
        function(id) assert(id <= 4); calls = calls + 1; return "name", unpack(many) end,
        function() calls = calls + 1; return unpack(many) end,
        function() calls = calls + 1; return unpack(many) end)
    for i = 1, 12 do capture(string.rep("l", 200)) end
    assert(calls == 110 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 2)
    local record = ApiContractProbeDB.captures[1]; local r = record.customSetNames
    assert(#record.label == 128 and #rows(r) == 4 and r.sets.n == 21 and r.sets.truncated)
    assert(#r.sets.values == 16 and #r.sets.values[2].value == 256)
    assert(rows(r)[1].info.n == 21 and rows(r)[1].validation.n == 20)
    assert(#rows(r)[1].validation.values == 16 and r.maximum[1].truncated)
end)

test("all excludes custom sets and missing access guards fail closed", function()
    setup(forbidden, forbidden, forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all fixture")
    assert(ApiContractProbeDB.captures[1].customSetNames == nil)
    for _, key in ipairs({ "issecretvalue", "canaccessvalue" }) do
        setup(forbidden, forbidden, forbidden, forbidden); _G[key] = nil
        SlashCmdList.APICONTRACTPROBE("custom-set-names fixture")
        assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
    end
end)

print(passed .. " custom-set-names fixtures passed")
