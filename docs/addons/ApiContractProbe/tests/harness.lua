local root = assert(arg[1], "addon directory required")
local passed = 0
local function test(name, fn)
    fn()
    passed = passed + 1
    print("PASS " .. name)
end
local secret = setmetatable({}, { __tostring = function() error("secret stringified") end })
local copyPoints, reversePoints, restrictedPoint = true, false, false
local function reset()
    ApiContractProbeDB = nil
    SlashCmdList = {}
    issecretvalue = function(value) return value == secret end
    canaccessvalue = function(value) return value ~= secret end
    GetBuildInfo = function() return "fixture", "123", "date", 120100 end
    GetLocale = function() return "enUS" end
    C_StringUtil = { FloorToNearestString = function() return "floor fixture" end,
        RoundToNearestString = function() return "round fixture" end }
    time = function() return 123456 end
    UnitExists = function(unit) return unit == "player" or unit == "target" end
    UnitName = function(unit) return "Name-" .. unit, nil end
    UnitNameUnmodified = function(unit) return "Original-" .. unit, "FixtureRealm" end
    UnitCastingInfo = function() return end
    UnitChannelInfo = function() return end
    UnitSex = function(unit) return unit == "player" and 2 or 3 end
    UnitSexBase = function(unit) return unit == "player" and 0 or 1 end
    Enum = { UnitSex = { Male = 0, Female = 1, None = 2, Both = 3, Neutral = 4 } }
    copyPoints, reversePoints, restrictedPoint = true, false, false
    C_CurveUtil = { CreateCurve = function()
        local points = {}
        local function point(p)
            if not p then return nil end
            if restrictedPoint then return secret end
            local result = copyPoints and { x = p.x, y = p.y } or p
            result.GetXY = function(self) return self.x, self.y end
            return result
        end
        return {
            AddPoint = function(_, x, y)
                points[#points + 1] = { x = x, y = y }
                table.sort(points, function(a, b) return a.x < b.x end)
            end,
            GetPoint = function(_, index) return point(points[index]) end,
            GetPoints = function()
                local result = {}
                for i = 1, #points do
                    result[i] = point(points[reversePoints and (#points - i + 1) or i])
                end
                return result
            end,
        }
    end }
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(mode)
    SlashCmdList.APICONTRACTPROBE(mode)
    return ApiContractProbeDB.captures[#ApiContractProbeDB.captures]
end
local function containsSecret(value)
    if rawequal(value, secret) then return true end
    if type(value) == "table" then
        for key, item in pairs(value) do
            if containsSecret(key) or containsSecret(item) then return true end
        end
    end
    return false
end

test("manual wiring and raw sex comparison", function()
    reset()
    assert(ApiContractProbeDB == nil)
    assert(SLASH_APICONTRACTPROBE1 == "/apicontract")
    local record = capture("sex normal")
    assert(record.label == "normal" and record.client.values[2].value == "123")
    assert(record.sex.units.player.legacy.values[1].value == 2)
    assert(record.sex.units.player.base.values[1].value == 0)
    assert(record.sex.units.target.base.values[1].value == 1)
    assert(record.sex.units.focus.exists.values[1].value == false)
    assert(record.sex.units.focus.legacy.values[1].value == 3)
    assert(record.sex.enums.Male.value == 0)
end)
test("sex does not normalize unusual native results or nil arity", function()
    reset()
    UnitSex = function() return 77 end
    UnitSexBase = function() return nil end
    local unit = capture("sex alternate").sex.units.player
    assert(unit.legacy.values[1].value == 77)
    assert(unit.base.n == 1 and unit.base.values[1].kind == "nil")
end)
test("sex independently records absent restricted and invalid unit controls", function()
    reset()
    UnitExists = function(unit)
        if unit == "target" then return secret end
        if unit == "invalid-unit-token" then error(secret) end
        return false
    end
    UnitSex = function(unit) return "legacy:" .. unit, nil end
    UnitSexBase = function(unit)
        if unit == "invalid-unit-token" then error(secret) end
        if unit == "" then return end
        if unit == "party2" then return secret end
        return nil, "base:" .. unit
    end
    Enum.UnitSex.Male = 81
    Enum.UnitSex.Neutral = secret
    local result = capture("sex controls").sex
    local count = 0
    for _ in pairs(result.units) do count = count + 1 end
    assert(count == 9)
    for _, token in ipairs({ "player", "target", "focus", "pet", "party1", "party2",
        "nonexistent", "invalid-unit-token", "" }) do
        local unit = assert(result.units[token])
        assert(unit.legacy.n == 2 and unit.legacy.values[1].value == "legacy:" .. token)
        assert(unit.legacy.values[2].kind == "nil")
        if token ~= "invalid-unit-token" and token ~= "" and token ~= "party2" then
            assert(unit.base.n == 2 and unit.base.values[1].kind == "nil")
            assert(unit.base.values[2].value == "base:" .. token)
        end
    end
    assert(result.units.target.exists.values[1].status == "restricted")
    assert(result.units.nonexistent.exists.values[1].value == false)
    assert(result.units["invalid-unit-token"].exists.status == "call-error")
    assert(result.units["invalid-unit-token"].base.status == "call-error")
    assert(result.units[""].base.n == 0)
    assert(result.units.party2.base.values[1].status == "restricted")
    assert(result.enums.Male.value == 81 and result.enums.Neutral.status == "restricted")
    assert(not containsSecret(ApiContractProbeDB))
end)
test("curve observations retain shapes ordering and copy effects", function()
    reset()
    local record = capture("curves")
    local curves = record.curves
    assert(curves.empty.n == 1)
    assert(curves.points.values[1].entries[1].fields.x.value == 10)
    assert(curves.indices[2].index == 1)
    assert(curves.indices[2].result.values[1].fields.y.value == 7)
    assert(curves.mutation.before.values[1].fields.x.value == 10)
    assert(curves.mutation.after.values[1].fields.x.value == 10)
    assert(curves.identity.value == false)
end)
test("alternative ordering and shared point identity are recorded", function()
    reset()
    copyPoints, reversePoints = false, true
    local curves = capture("curves").curves
    assert(curves.points.values[1].entries[1].fields.x.value == 30)
    assert(curves.identity.value == true)
    assert(curves.mutation.after.values[1].fields.x.value == 91)
end)
test("secret values and opaque errors never enter saved variables", function()
    reset()
    UnitSexBase = function() return secret end
    UnitSex = function() error(secret) end
    restrictedPoint = true
    capture("all")
    local record = ApiContractProbeDB.captures[1]
    assert(record.sex.units.player.base.values[1].status == "restricted")
    assert(record.sex.units.player.legacy.status == "call-error")
    assert(record.curves.indices[2].result.values[1].status == "restricted")
    assert(not containsSecret(ApiContractProbeDB))
end)
test("missing access probes fail closed and missing APIs are explicit", function()
    reset()
    issecretvalue = nil
    local record = capture("all")
    assert(record.status == "missing-access-api")
    reset()
    UnitSexBase, C_CurveUtil.CreateCurve = nil, nil
    record = capture("all")
    assert(record.sex.units.player.base.status == "missing-api")
    assert(record.curves.status == "missing-api")
end)
test("bounded captures and invalid command", function()
    reset()
    SlashCmdList.APICONTRACTPROBE("unknown")
    assert(ApiContractProbeDB == nil)
    for _ = 1, 12 do capture("sex") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 2)
end)
test("restricted unit existence prevents queries", function()
    reset()
    UnitExists = function() return secret end
    UnitSex = function() error("must not query") end
    local record = capture("sex")
    assert(record.sex.units.player.status == "restricted-or-error")
    assert(not containsSecret(record))
end)
test("publication distinguishes missing parent/member and CVar observations", function()
    reset()
    ApiContractProbeTargets = { publication = {
        { id = "removed:Fixture.Value", owner = "enum", path = "Fixture.Value", plan = "p" },
        { id = "removed:Fixture.Absent", owner = "enum", path = "Fixture.Absent", plan = "p" },
        { id = "removed:NoParent.Value", owner = "enum", path = "NoParent.Value", plan = "p" },
        { id = "removed:testCvar", owner = "cvar", path = "testCvar", plan = "p" },
    }, events = {} }
    Fixture = { Value = 42 }
    C_CVar = { GetCVar = function() return "1" end, GetCVarDefault = function() return "0" end }
    local rows = capture("publication after-login").publication
    assert(rows[1].ordinary.value == 42 and rows[1].raw.value == 42)
    assert(rows[2].ordinary.kind == "nil" and rows[2].parent.status == "observed")
    assert(rows[3].status == "missing-parent")
    assert(rows[4].current.values[1].value == "1" and rows[4].default.values[1].value == "0")
end)
test("event recorder registers actual targets and bounds redacted tuples", function()
    reset()
    ApiContractProbeTargets = { publication = {}, events = {
        { id = "changed:UNIT_HEALTH", event = "UNIT_HEALTH", plan = "events" },
        { id = "removed:BAD_EVENT", event = "BAD_EVENT", plan = "removed" },
    } }
    local frame = {}
    CreateFrame = function()
        function frame:SetScript(_, handler) self.handler = handler end
        function frame:RegisterEvent(event) if event == "BAD_EVENT" then error(secret) end end
        function frame:UnregisterAllEvents() self.stopped = true end
        return frame
    end
    SlashCmdList.APICONTRACTPROBE("events-start cast-control")
    assert(ApiContractProbeDB.registrations[1].status == "registered")
    assert(ApiContractProbeDB.registrations[2].status == "registration-error")
    local calls = 0
    local payload = setmetatable({ GetXY = function() calls = calls + 1 end }, {
        __index = function() calls = calls + 1; return nil end,
    })
    frame.handler(frame, "UNIT_HEALTH", "player", nil, secret, payload)
    assert(calls == 0, "passive capture must not invoke payload code")
    local event = ApiContractProbeDB.events[1]
    assert(event.name == "UNIT_HEALTH" and event.payload.n == 4)
    assert(event.payload.values[2].kind == "nil" and event.payload.values[3].status == "restricted")
    for _ = 1, 300 do frame.handler(frame, "UNIT_HEALTH", "target") end
    assert(#ApiContractProbeDB.events == 256 and ApiContractProbeDB.droppedEvents == 45)
    SlashCmdList.APICONTRACTPROBE("events-stop")
    assert(frame.stopped and ApiContractProbeDB.eventStatus == "stopped")
    assert(not containsSecret(ApiContractProbeDB))
end)
test("names preserve differing values realm forms and exact nil arity", function()
    reset()
    UnitExists = function() error("names must not gate on existence") end
    UnitName = function(unit)
        if unit == "player" then return "DisplayPlayer", nil end
        if unit == "party1" then return "DisplayAlly", "" end
        if unit == "party2" then return "RemoteAlly", "OtherRealm" end
        if unit == "party3" then return nil, "RealmWithoutName" end
        if unit == "party4" then return "Fourth", nil, nil end
        if unit == "target" then return "DisplayTarget" end
        if unit == "nonexistent" then return end
        if unit == "invalid-unit-token" then return nil end
        if unit == "" then return nil, nil end
        error("unexpected fixture token")
    end
    local record = capture("names same-realm party1; cross-realm party2")
    assert(record and record.names, "names command must capture observations")
    assert(record.label == "same-realm party1; cross-realm party2")
    assert(record.client.values[2].value == "123")
    local units = record.names.units
    assert(units.player.name.n == 2 and units.player.name.values[1].value == "DisplayPlayer")
    assert(units.player.name.values[2].kind == "nil")
    assert(units.player.unmodified.values[1].value == "Original-player")
    assert(units.party1.name.values[2].kind == "string" and units.party1.name.values[2].value == "")
    assert(units.party2.name.values[2].value == "OtherRealm")
    assert(units.party3.name.values[1].kind == "nil" and units.party3.name.values[2].value == "RealmWithoutName")
    assert(units.party4.name.n == 3 and units.party4.name.values[3].kind == "nil")
    assert(units.target.name.n == 1 and units.target.name.values[1].value == "DisplayTarget")
    assert(units.nonexistent.name.n == 0 and next(units.nonexistent.name.values) == nil)
    assert(units["invalid-unit-token"].name.n == 1 and units["invalid-unit-token"].name.values[1].kind == "nil")
    assert(units[""].name.n == 2 and units[""].name.values[2].kind == "nil")
    for _, token in ipairs({
        "player", "party1", "party2", "party3", "party4", "target",
        "nonexistent", "invalid-unit-token", "",
    }) do
        assert(units[token].unmodified.n == 2)
        assert(units[token].unmodified.values[1].value == "Original-" .. token)
        assert(units[token].unmodified.values[2].value == "FixtureRealm")
    end
end)
test("names redact secret inaccessible and error returns", function()
    reset()
    UnitName = function(unit)
        if unit == "player" then return secret, "OrdinaryRealm" end
        if unit == "party1" then return "VisibleAlly", "InaccessibleRealm" end
        error(secret)
    end
    UnitNameUnmodified = function() error("private error details") end
    canaccessvalue = function(value) return value ~= secret and value ~= "InaccessibleRealm" end
    local record = capture("names restricted")
    assert(record and record.names, "names command must capture redacted observations")
    local units = record.names.units
    assert(units.player.name.n == 2 and units.player.name.values[1].status == "restricted")
    assert(units.player.name.values[2].value == "OrdinaryRealm")
    assert(units.party1.name.values[1].value == "VisibleAlly")
    assert(units.party1.name.values[2].status == "restricted" and units.party1.name.values[2].value == nil)
    assert(units.target.name.status == "call-error" and units.target.name.values == nil)
    assert(units.player.unmodified.status == "call-error" and units.player.unmodified.values == nil)
    assert(not containsSecret(ApiContractProbeDB))
end)
test("all includes names and names fail closed for unavailable access or APIs", function()
    reset()
    assert(capture("all baseline").names.units.player.name.values[1].value == "Name-player")
    reset()
    UnitName, UnitNameUnmodified = nil, nil
    local units = capture("names missing").names.units
    assert(units.player.name.status == "missing-api" and units.player.unmodified.status == "missing-api")
    reset()
    canaccessvalue = nil
    local record = capture("names no-access")
    assert(record.status == "missing-access-api" and record.names == nil)
end)
local numberInputs = {
    -2.5, -1.5, -0.5, 0, 0.5, 1.5, 2.5,
    -0.500001, -0.499999, 0.499999, 0.500001,
    -1, 1, -100, 100, -0.1, 0.1, -0.9, 0.9,
    -1234.5678, 1234.5678, -1e6, 1e6, -1e12, 1e12,
}
test("numbers preserve finite corpus and differing locale bytes without rounding", function()
    for _, locale in ipairs({ "enUS", "frFR" }) do
        reset()
        GetLocale = function() return locale end
        local floorText = locale == "enUS" and "not rounded: 1,234.5" or "1\194\160234,5"
        local roundText = locale == "enUS" and "alternate -zero" or "moins\226\136\146zéro"
        local seenFloor, seenRound = {}, {}
        C_StringUtil.FloorToNearestString = function(value)
            seenFloor[#seenFloor + 1] = value
            return floorText, nil, "tail"
        end
        C_StringUtil.RoundToNearestString = function(value)
            seenRound[#seenRound + 1] = value
            return nil, roundText, nil
        end
        local record = capture("numbers " .. locale .. " manual")
        assert(record and record.numbers, "numbers command must record observations")
        assert(record.label == locale .. " manual" and record.client.values[2].value == "123")
        assert(record.numbers.locale.n == 1 and record.numbers.locale.values[1].value == locale)
        assert(#record.numbers.samples == #numberInputs)
        assert(#seenFloor == #numberInputs and #seenRound == #numberInputs)
        for index, input in ipairs(numberInputs) do
            local row = record.numbers.samples[index]
            assert(row.input == input and seenFloor[index] == input and seenRound[index] == input)
            assert(row.floor.n == 3 and row.floor.values[1].value == floorText)
            assert(row.floor.values[2].kind == "nil" and row.floor.values[3].value == "tail")
            assert(row.round.n == 3 and row.round.values[1].kind == "nil")
            assert(row.round.values[2].value == roundText and row.round.values[3].kind == "nil")
        end
    end
end)
test("numbers include all and preserve zero returns restricted values and opaque errors", function()
    reset()
    assert(capture("all").numbers.samples[1].floor.values[1].value == "floor fixture")
    reset()
    GetLocale = function() return secret end
    C_StringUtil.FloorToNearestString = function(value)
        if value == -2.5 then return end
        if value == -1.5 then return secret end
        return "denied"
    end
    C_StringUtil.RoundToNearestString = function() error(secret) end
    canaccessvalue = function(value) return value ~= secret and value ~= "denied" end
    local numbers = capture("numbers redacted").numbers
    assert(numbers.locale.values[1].status == "restricted")
    assert(numbers.samples[1].floor.n == 0 and next(numbers.samples[1].floor.values) == nil)
    assert(numbers.samples[2].floor.values[1].status == "restricted")
    assert(numbers.samples[3].floor.values[1].status == "restricted")
    for _, row in ipairs(numbers.samples) do
        assert(row.round.status == "call-error" and row.round.values == nil)
    end
    assert(not containsSecret(ApiContractProbeDB))
end)
test("numbers fail closed for missing APIs and failed access checks", function()
    reset()
    C_StringUtil, GetLocale = nil, nil
    local numbers = capture("numbers missing").numbers
    assert(numbers.locale.status == "missing-api")
    for _, row in ipairs(numbers.samples) do
        assert(row.floor.status == "missing-api" and row.round.status == "missing-api")
    end
    for _, accessName in ipairs({ "issecretvalue", "canaccessvalue" }) do
        reset()
        local calls = 0
        C_StringUtil.FloorToNearestString = function() calls = calls + 1 end
        C_StringUtil.RoundToNearestString = C_StringUtil.FloorToNearestString
        _G[accessName] = function() error(secret) end
        numbers = capture("numbers failed-access").numbers
        assert(calls == 0 and not containsSecret(numbers))
        assert(numbers.samples[1].floor.status == "missing-api")
        reset()
        _G[accessName] = nil
        local record = capture("numbers absent-access")
        assert(record.status == "missing-access-api" and record.numbers == nil)
    end
end)
test("casts preserve fixed tokens arity identities and manual sequences", function()
    reset()
    local tokens = { "player", "target", "focus", "party1", "nonexistent", "invalid-unit-token", "" }
    local seenCast, seenChannel = {}, {}
    local identity = "cast-A"
    UnitExists = function() error("casts must not gate on existence") end
    UnitCastingInfo = function(unit)
        seenCast[#seenCast + 1] = unit
        return "Cast", nil, 123, 100, 200, false, identity, nil, 456, 901
    end
    UnitChannelInfo = function(unit)
        seenChannel[#seenChannel + 1] = unit
        return "Channel", "Shown", 124, 110, 210, false, nil, 457, true, 4, 902
    end
    local first = capture("casts active-A")
    assert(first and first.casts, "casts command must capture observations")
    assert(first.label == "active-A" and first.client.values[2].value == "123")
    assert(first.time.values[1].value == 123456)
    assert(#seenCast == 7 and #seenChannel == 7)
    for index, token in ipairs(tokens) do
        assert(seenCast[index] == token and seenChannel[index] == token)
        local row = first.casts.units[token]
        assert(row.casting.n == 10 and row.casting.values[2].kind == "nil")
        assert(row.casting.values[7].value == "cast-A" and row.casting.values[8].kind == "nil")
        assert(row.casting.values[10].value == 901 and row.casting.truncated == nil)
        assert(row.channel.n == 11 and row.channel.values[7].kind == "nil")
        assert(row.channel.values[9].value == true and row.channel.values[10].value == 4)
        assert(row.channel.values[11].value == 902 and row.channel.truncated == nil)
    end
    assert(capture("casts active-A-again").casts.units.player.casting.values[7].value == "cast-A")
    identity = "cast-B"
    assert(capture("all replacement-B").casts.units.player.casting.values[7].value == "cast-B")
    assert(first.casts.units.player.casting.values[7].value == "cast-A")
    UnitChannelInfo = function() return "Plain", nil, nil, nil, nil, false, nil, 99, false, 0, nil end
    local channel = capture("casts plain-channel").casts.units.player.channel
    assert(channel.n == 11 and channel.values[9].value == false and channel.values[10].value == 0)
    assert(channel.values[11].kind == "nil")
    UnitCastingInfo, UnitChannelInfo = function() return end, function() return end
    local idle = capture("casts completed-or-cancelled").casts.units.player
    assert(idle.casting.n == 0 and next(idle.casting.values) == nil)
    assert(idle.channel.n == 0 and next(idle.channel.values) == nil)
end)
test("casts bound positional storage without losing trailing nil arity", function()
    reset()
    UnitCastingInfo = function() return 1, nil, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, nil end
    UnitChannelInfo = function() return 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, nil end
    local row = capture("casts overflow").casts.units.player
    assert(row.casting.n == 16 and #row.casting.values == 16 and row.casting.truncated == nil)
    assert(row.casting.values[16].kind == "nil")
    assert(row.channel.n == 17 and #row.channel.values == 16 and row.channel.truncated == true)
    for _ = 1, 11 do capture("casts repeated") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 2)
end)
test("casts never inspect returned objects or expose opaque errors", function()
    reset()
    local calls = 0
    local function hostile() calls = calls + 1; error("object code executed") end
    local mt = { __index = hostile, __eq = hostile, __tostring = hostile }
    local object = setmetatable({ GetXY = hostile }, mt)
    local denied = setmetatable({}, mt)
    local userdata = newproxy(true)
    local umt = getmetatable(userdata)
    umt.__index, umt.__eq, umt.__tostring = hostile, hostile, hostile
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, denied) end
    UnitCastingInfo = function() return object, userdata, secret, denied end
    UnitChannelInfo = function() error(object) end
    local row = capture("casts hostile").casts.units.player
    assert(row.casting.n == 4 and row.casting.values[1].kind == "table")
    assert(row.casting.values[1].fields == nil and row.casting.values[1].value == nil)
    assert(row.casting.values[2].kind == "userdata" and row.casting.values[2].value == nil)
    assert(row.casting.values[3].status == "restricted" and row.casting.values[4].status == "restricted")
    assert(row.channel.status == "call-error" and row.channel.values == nil)
    assert(not containsSecret(ApiContractProbeDB) and calls == 0)
end)
test("casts fail closed for unavailable functions and access checks", function()
    reset()
    UnitCastingInfo, UnitChannelInfo = nil, nil
    local row = capture("casts absent").casts.units.player
    assert(row.casting.status == "missing-api" and row.channel.status == "missing-api")
    for _, name in ipairs({ "issecretvalue", "canaccessvalue" }) do
        reset()
        local calls = 0
        UnitCastingInfo = function() calls = calls + 1 end
        UnitChannelInfo = UnitCastingInfo
        _G[name] = function() error(secret) end
        row = capture("casts access-error").casts.units.player
        assert(calls == 0 and row.casting.status == "missing-api" and row.channel.status == "missing-api")
        reset()
        _G[name] = nil
        local record = capture("casts no-access")
        assert(record.status == "missing-access-api" and record.casts == nil)
    end
end)
test("selected action slot retains counts fields and explicit display formats", function()
    reset()
    GetActionInfo = function(slot) return "spell", slot + 100, nil end
    C_ActionBar = {
        GetActionDisplayCount = function(slot, ...)
            local n = select("#", ...)
            if n == 0 then return "" end
            local threshold, replacement = ...
            return slot .. ":" .. threshold .. ":" .. replacement
        end,
        GetActionCharges = function(slot)
            return { currentCharges = slot, maxCharges = 3, cooldownStartTime = 12,
                cooldownDuration = nil, chargeModRate = 0.5 }, nil
        end,
        GetActionChargeDuration = function() return {}, nil end,
    }
    local record = capture("actions 7 before use")
    local a = record.actions
    assert(record.label == "before use" and a.slot == 7)
    assert(a.identity.n == 3 and a.identity.values[2].value == 107)
    assert(a.display.default.values[1].value == "")
    for i, threshold in ipairs({ 0, 1, 9999 }) do
        assert(a.display.formats[i].result.values[1].value == "7:" .. threshold .. ":*")
    end
    assert(a.charges.n == 2 and a.charges.values[2].kind == "nil")
    local fields = a.charges.values[1].fields
    assert(fields.currentCharges.value == 7 and fields.maxCharges.value == 3)
    assert(fields.cooldownStartTime.value == 12 and fields.cooldownDuration.kind == "nil")
    assert(fields.chargeModRate.value == 0.5)
    assert(a.duration.n == 2 and a.duration.values[1].kind == "table")
    assert(a.duration.values[1].fields == nil and a.duration.values[2].kind == "nil")
    assert(capture("actions 0 empty").actions.slot == 0)
    assert(capture("actions -1 invalid").actions.charges.values[1].fields.currentCharges.value == -1)
end)
test("action payloads stay passive and restricted errors stay opaque", function()
    reset()
    local touched = 0
    local hostile = setmetatable({ currentCharges = secret }, {
        __index = function() touched = touched + 1; error("index invoked") end,
        __tostring = function() touched = touched + 1; error("stringified") end,
    })
    local duration = setmetatable({ GetRemainingDuration = function() touched = touched + 1 end }, getmetatable(hostile))
    GetActionInfo = function() return secret end
    C_ActionBar = { GetActionCharges = function() return hostile end,
        GetActionDisplayCount = function() error(secret) end,
        GetActionChargeDuration = function() return duration end }
    local a = capture("actions 4 hostile").actions
    assert(a.identity.values[1].status == "restricted")
    assert(a.charges.values[1].fields.currentCharges.status == "restricted")
    assert(a.charges.values[1].fields.maxCharges.kind == "nil")
    assert(a.display.default.status == "call-error" and a.duration.values[1].fields == nil)
    assert(touched == 0 and not containsSecret(a))
    C_ActionBar.GetActionCharges = function() return secret end
    C_ActionBar.GetActionChargeDuration = function() return secret end
    a = capture("actions 4 restricted").actions
    assert(a.charges.values[1].status == "restricted" and a.duration.values[1].status == "restricted")
    C_ActionBar.GetActionChargeDuration = function() return 123, "opaque" end
    a = capture("actions 4 scalar-duration").actions
    assert(a.duration.n == 2 and a.duration.values[1].kind == "number")
    assert(a.duration.values[1].value == nil and a.duration.values[2].value == nil)
    C_ActionBar = setmetatable({}, getmetatable(hostile))
    a = capture("actions 4 missing").actions
    assert(a.charges.status == "missing-api" and a.duration.status == "missing-api" and touched == 0)
end)
test("invalid action commands and all never query selected-slot APIs", function()
    reset()
    local calls = 0
    local function query() calls = calls + 1; return nil end
    GetActionInfo = query
    C_ActionBar = { GetActionDisplayCount = query, GetActionCharges = query, GetActionChargeDuration = query }
    for _, command in ipairs({ "actions", "actions nope", "actions 1.5", "actions 1e2", "actions 0x10", "actions inf", "actions 9007199254740992" }) do
        SlashCmdList.APICONTRACTPROBE(command)
        assert(ApiContractProbeDB == nil and calls == 0)
    end
    assert(capture("all").actions == nil and calls == 0)
    for i = 1, 9 do capture("actions 2 sample") end
    assert(calls == 63)
    capture("actions 2 dropped")
    assert(calls == 63 and ApiContractProbeDB.dropped == 1)
end)
test("hyperlink corpus records exact inputs and all nine flag variants", function()
    reset()
    local calls = {}
    C_StringUtil.StripHyperlinks = function(...)
        local args = { n = select("#", ...), ... }
        calls[#calls + 1] = args
        return "é漢字🙂\000|h" .. args[1], nil, args.n
    end
    local record = capture("hyperlinks native-label")
    assert(record.label == "native-label" and #record.hyperlinks == 144)
    assert(#calls == 144)
    for i, row in ipairs(record.hyperlinks) do
        local args = calls[i]
        assert(row.input == args[1] and row.argumentCount == args.n)
        assert(args.n == (row.variant == "omitted" and 1 or 6))
        for flag = 1, #row.flags do assert(row.flags[flag] == args[flag + 1]) end
        assert(row.result.n == 3 and row.result.values[2].kind == "nil")
        assert(row.result.values[1].value == "é漢字🙂\000|h" .. row.input)
        assert(row.result.values[3].value == args.n)
    end
    local expected = {
        {}, {false,false,false,false,false}, {true,false,false,false,false},
        {false,true,false,false,false}, {false,false,true,false,false},
        {false,false,false,true,false}, {false,false,false,false,true},
        {false,true,false,true,true}, {true,true,true,true,true},
    }
    for i, flags in ipairs(expected) do
        for j = 1, 5 do assert(record.hyperlinks[i].flags[j] == flags[j]) end
    end
    local found = {}
    for _, row in ipairs(record.hyperlinks) do found[row.input] = true end
    for _, input in ipairs({ "plain ASCII", "é漢字🙂", "", "a|nb", "a\nb", "||", "|h",
        "|Hitem:19019|h[Item]", "|cffff0000red", "|A:atlas:16:16", "|Ttexture:16:16" }) do
        assert(found[input])
    end
end)
test("hyperlink results remain opaque restricted or explicitly truncated", function()
    reset()
    C_StringUtil.StripHyperlinks = function() return secret end
    local rows = capture("hyperlinks restricted").hyperlinks
    assert(rows[1].result.values[1].status == "restricted" and not containsSecret(rows))
    C_StringUtil.StripHyperlinks = function() error(secret) end
    assert(capture("hyperlinks error").hyperlinks[1].result.status == "call-error")
    C_StringUtil.StripHyperlinks = function() return string.rep("x", 300) end
    local value = capture("hyperlinks long").hyperlinks[1].result.values[1]
    assert(value.value == string.rep("x", 256) and value.truncated == true)
    C_StringUtil = secret
    assert(capture("hyperlinks missing").hyperlinks[1].result.status == "missing-api")
    issecretvalue = nil
    assert(capture("hyperlinks closed").status == "missing-access-api")
end)
test("hyperlinks is manual only and capture cap prevents calls", function()
    reset()
    local calls = 0
    C_StringUtil.StripHyperlinks = function(text) calls = calls + 1; return text end
    assert(capture("all").hyperlinks == nil and calls == 0)
    for i = 1, 9 do capture("hyperlinks batch") end
    assert(calls == 9 * 144)
    capture("hyperlinks dropped")
    assert(calls == 9 * 144 and ApiContractProbeDB.dropped == 1)
end)
local function resourceFixture(scale)
    reset()
    Enum.LuaCurveType = { Linear = 37 }
    C_CurveUtil.CreateCurve = function()
        local curve = { points = {} }
        curve.SetType = function(self, kind) assert(kind == 37); self.linear = true end
        curve.AddPoint = function(self, x, y) self.points[#self.points + 1] = { x = x, y = y } end
        return curve
    end
    UnitHealth = function(unit) if unit == "nonexistent" then return end; return 50 end
    UnitHealthMax = function() return 100 end
    UnitPower = function(_, power, unmodified) assert(power == nil); return unmodified and 500 or 50 end
    UnitPowerMax = function(_, power, unmodified) assert(power == nil); return unmodified and 1000 or 100 end
    UnitPowerType = function() return 73, "FixturePower" end
    UnitHealthPercent = function(unit, predicted, curve)
        if unit == "nonexistent" then return nil end
        if curve then assert(predicted == false and curve.linear and #curve.points == 6); return scale end
        return predicted == false and 51 or 52
    end
    UnitPowerPercent = function(_, power, unmodified, curve)
        assert(power == nil)
        if curve then assert(curve.linear and #curve.points == 6); return scale + (unmodified and 1 or 0) end
        return unmodified and 61 or 60
    end
end
test("resources retain discriminating scalar results and correct argument positions", function()
    resourceFixture(10)
    local first = capture("resources partial").resources
    assert(first.curve.inputs[2].x == 0.5 and first.curve.inputs[6].y == 50)
    assert(first.units.player.health.curved.values[1].value == 10)
    assert(first.units.player.health.default.values[1].value == 52)
    assert(first.units.player.power.modified.curved.values[1].value == 10)
    assert(first.units.player.power.unmodified.curved.values[1].value == 11)
    assert(first.units.player.power.unmodified.current.values[1].value == 500)
    assert(first.units.player.power.type.values[1].value == 73)
    assert(first.units.nonexistent.health.current.n == 0)
    assert(first.units.nonexistent.health.curved.values[1].kind == "nil")
    resourceFixture(40)
    assert(capture("resources alternate").resources.units.player.health.curved.values[1].value == 40)
end)
test("resources fail closed for missing linear enum and opaque failures", function()
    resourceFixture(10)
    Enum.LuaCurveType.Linear = secret
    local result = capture("resources missing").resources
    assert(result.curve.status == "missing-linear-type")
    assert(result.units.player.health.curved.status == "curve-unavailable")
    resourceFixture(10)
    UnitHealth = function() return secret end
    UnitPowerPercent = function() error(secret) end
    result = capture("resources restricted").resources
    assert(result.units.player.health.current.values[1].status == "restricted")
    assert(result.units.player.power.modified.curved.status == "call-error")
    assert(not containsSecret(result))
    resourceFixture(10)
    C_CurveUtil = nil
    assert(capture("resources absent").resources.curve.status == "missing-api")
end)
test("resources are manual and bounded", function()
    resourceFixture(10)
    assert(capture("all").resources == nil)
    for _ = 1, 9 do assert(capture("resources").resources) end
    UnitHealth = function() error("must not run") end
    capture("resources overflow")
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)
local function callbackFixture()
    reset()
    local registered, removed = {}, {}
    RegisterEventCallback = function(event, cb)
        assert(event == "UNIT_HEALTH")
        registered.global = cb
        return true, nil
    end
    RegisterUnitEventCallback = function(event, cb, unit)
        assert(event == "UNIT_HEALTH" and unit == "player")
        registered.unit = cb
    end
    UnregisterEventCallback = function(event, cb)
        assert(event == "UNIT_HEALTH" and cb == registered.global)
        removed.global = cb; registered.global = nil
    end
    UnregisterUnitEventCallback = function(event, cb, unit)
        assert(event == "UNIT_HEALTH" and unit == "player" and cb == registered.unit)
        removed.unit = cb; registered.unit = nil
        return nil, 7
    end
    return registered, removed
end
test("callbacks retain own identity, raw payloads and restart sessions", function()
    local registered, removed = callbackFixture()
    capture("all")
    assert(not registered.global and not registered.unit)
    SlashCmdList.APICONTRACTPROBE("callbacks-start first")
    local db = ApiContractProbeDB
    local session = db.callbackSessions[1]
    assert(session.status == "recording" and session.label == "first")
    assert(session.global.registration.n == 2 and session.unit.registration.n == 0)
    local global, unit = registered.global, registered.unit
    local hostile = setmetatable({}, { __index = function() error("indexed") end,
        __tostring = function() error("stringified") end })
    global(nil, "target", secret, hostile, nil)
    unit(nil, "player")
    assert(#session.events == 2 and session.events[1].payload.n == 5)
    assert(session.events[1].payload.values[1].kind == "nil")
    assert(session.events[1].payload.values[3].status == "restricted")
    assert(session.events[1].payload.values[4].kind == "table")
    assert(not containsSecret(db))
    SlashCmdList.APICONTRACTPROBE("callbacks-start duplicate")
    assert(#db.callbackSessions == 1 and registered.global == global)
    assert(db.callbackStatus == "already-running")
    SlashCmdList.APICONTRACTPROBE("callbacks-stop")
    assert(session.status == "stopped" and removed.global == global and removed.unit == unit)
    assert(session.unit.removal.n == 2 and session.unit.removal.values[1].kind == "nil")
    global("late"); unit("late")
    assert(#session.events == 2)
    SlashCmdList.APICONTRACTPROBE("callbacks-start second")
    assert(#db.callbackSessions == 2 and registered.global ~= global)
    registered.global(nil, "player")
    assert(#db.callbackSessions[2].events == 1 and #session.events == 2)
    SlashCmdList.APICONTRACTPROBE("callbacks-stop")
end)
test("callback cap keeps cleanup available and sessions bounded", function()
    local registered = callbackFixture()
    SlashCmdList.APICONTRACTPROBE("callbacks-start capped")
    local session = ApiContractProbeDB.callbackSessions[1]
    registered.global(1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17)
    assert(session.events[1].payload.n == 17 and session.events[1].payload.truncated)
    assert(#session.events[1].payload.values == 16)
    for _ = 1, 130 do registered.unit(nil, "player") end
    assert(#session.events == 128 and session.dropped == 3)
    SlashCmdList.APICONTRACTPROBE("callbacks-stop")
    assert(not registered.global and not registered.unit)
    for _ = 2, 10 do
        SlashCmdList.APICONTRACTPROBE("callbacks-start next")
        SlashCmdList.APICONTRACTPROBE("callbacks-stop")
    end
    SlashCmdList.APICONTRACTPROBE("callbacks-start overflow")
    assert(#ApiContractProbeDB.callbackSessions == 10 and not registered.global)
    assert(ApiContractProbeDB.callbackStatus == "session-limit")
end)
test("callback partial errors retain cleanup identity and refuse false success", function()
    local registered = callbackFixture()
    RegisterUnitEventCallback = function(_, cb) registered.unit = cb; error(secret) end
    SlashCmdList.APICONTRACTPROBE("callbacks-start partial")
    local session = ApiContractProbeDB.callbackSessions[1]
    assert(session.status == "registration-incomplete")
    assert(session.unit.registration.status == "call-error")
    local cleanup = UnregisterEventCallback
    UnregisterEventCallback = function() error(secret) end
    SlashCmdList.APICONTRACTPROBE("callbacks-stop")
    assert(session.status == "cleanup-incomplete" and session.global.removal.status == "call-error")
    assert(not registered.unit and registered.global)
    SlashCmdList.APICONTRACTPROBE("callbacks-start blocked")
    assert(#ApiContractProbeDB.callbackSessions == 1)
    UnregisterEventCallback = cleanup
    SlashCmdList.APICONTRACTPROBE("callbacks-stop")
    assert(session.status == "stopped" and not registered.global and not containsSecret(session))
    callbackFixture()
    RegisterEventCallback = function() return false end
    SlashCmdList.APICONTRACTPROBE("callbacks-start refused")
    session = ApiContractProbeDB.callbackSessions[1]
    assert(session.status == "registration-incomplete" and session.global.status == "refused")
    assert(session.unit.status == "registered")
end)
test("callbacks missing access fails closed and cleanup refusal remains active", function()
    local registered = callbackFixture()
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("callbacks-start unsafe")
    assert(not registered.global and not registered.unit)
    assert(ApiContractProbeDB.callbackStatus == "missing-access-api")
    callbackFixture()
    SlashCmdList.APICONTRACTPROBE("callbacks-start safe")
    UnregisterUnitEventCallback = function() return false end
    SlashCmdList.APICONTRACTPROBE("callbacks-stop")
    assert(ApiContractProbeDB.callbackSessions[1].status == "cleanup-incomplete")
    assert(ApiContractProbeDB.callbackSessions[1].unit.status == "cleanup-refused")
end)
local function stateCurveFixture(alias, replaceDuplicates, defaultType)
    Enum.LuaCurveType = { Linear = 73 }
    local function create(points, curveType)
        points = points or {}
        curveType = curveType or 19
        local curve = {}
        function curve:SetType(value) curveType = value end
        function curve:GetType() return curveType end
        function curve:HasSecretValues() return false end
        function curve:GetPointCount() return #points end
        function curve:GetPoints()
            local result = {}
            for i, p in ipairs(points) do result[i] = { x = p.x, y = p.y } end
            return result
        end
        function curve:AddPoint(x, y)
            if replaceDuplicates then
                for _, p in ipairs(points) do if p.x == x then p.y = y; return end end
            end
            points[#points + 1] = { x = x, y = y }
        end
        function curve:Evaluate(x) return x + #points * 100 end
        function curve:SetToDefaults() points = {}; curveType = defaultType end
        function curve:ClearPoints() points = {} end
        function curve:Copy()
            if alias then return self end
            local copied = {}
            for i, p in ipairs(points) do copied[i] = { x = p.x, y = p.y } end
            return create(copied, curveType)
        end
        return curve
    end
    C_CurveUtil.CreateCurve = function() return create() end
end

test("curve state retains duplicate defaults evaluations and independent copies", function()
    reset(); stateCurveFixture(false, false, 91)
    local state = capture("curve-state own").curveState
    assert(state.inputs[4].x == 10 and state.inputs[4].y == 9)
    assert(state.empty.before.count.values[1].value == 0)
    assert(state.populated.setType.status == "observed")
    assert(state.populated.before.curveType.values[1].value == 73)
    assert(state.populated.before.count.values[1].value == 4)
    assert(state.populated.before.points.values[1].entries[4].fields.y.value == 9)
    assert(state.populated.before.evaluations[1].result.values[1].value == 399)
    assert(state.populated.after.curveType.values[1].value == 91)
    assert(state.populated.after.count.values[1].value == 0)
    assert(state.populated.after.evaluations[7].result.values[1].value == 31)
    assert(state.copy.before.count.values[1].value == 4)
    assert(state.copy.afterAdd.count.values[1].value == 4)
    assert(state.copy.afterClear.count.values[1].value == 4)
    assert(state.copy.originalAfterAdd.count.values[1].value == 5)
    assert(state.copy.originalAfterClear.count.values[1].value == 0)
end)
test("curve state records alternative duplicate and alias behavior without normalization", function()
    reset(); stateCurveFixture(true, true, 27)
    local state = capture("curve-state alias").curveState
    assert(state.populated.before.count.values[1].value == 3)
    assert(state.populated.after.curveType.values[1].value == 27)
    assert(state.copy.afterAdd.count.values[1].value == 4)
    assert(state.copy.afterClear.count.values[1].value == 0)
end)
test("curve state missing enum copy errors and restricted objects fail closed", function()
    reset(); stateCurveFixture(false, false, 1)
    local create = C_CurveUtil.CreateCurve
    Enum.LuaCurveType.Linear = secret
    C_CurveUtil.CreateCurve = function()
        local curve = create()
        curve.SetType = function() error("restricted enum reached setter") end
        curve.Copy = function() error(secret) end
        curve.Evaluate = function() return secret end
        return curve
    end
    local state = capture("curve-state restricted").curveState
    assert(state.populated.setType.status == "missing-linear-type")
    assert(state.populated.before.evaluations[1].result.values[1].status == "restricted")
    assert(state.copy.result.status == "call-error")
    assert(not containsSecret(ApiContractProbeDB))
    C_CurveUtil.CreateCurve = function() return secret end
    assert(capture("curve-state constructor").curveState.empty.status == "restricted-curve")
end)
test("curve state is manual and bounded without changing curves capture", function()
    reset(); stateCurveFixture(false, false, 1)
    assert(capture("all").curveState == nil)
    for _ = 1, 9 do capture("curve-state bounded") end
    C_CurveUtil.CreateCurve = function() error("overflow called constructor") end
    capture("curve-state overflow")
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
end)
local function editCurveFixture(zeroBased, reverseReplacement)
    local vectors = {}
    CreateVector2D = function(x, y)
        local vector = { x = x, y = y }
        vectors[vector] = true
        return vector
    end
    C_CurveUtil.CreateCurve = function()
        local points = {}
        return {
            AddPoint = function(_, x, y) points[#points + 1] = { x = x, y = y } end,
            GetPointCount = function() return #points end,
            GetPoints = function()
                local result = {}
                for i, p in ipairs(points) do result[i] = { x = p.x, y = p.y } end
                return result
            end,
            Evaluate = function(_, x) return #points * 100 + x end,
            RemovePoint = function(_, index)
                local position = zeroBased and index + 1 or index
                if position < 1 or position > #points then error(secret) end
                table.remove(points, position)
                return nil, "removed"
            end,
            SetPoints = function(_, replacement)
                points = {}
                for i = 1, #replacement do
                    local p = replacement[reverseReplacement and (#replacement - i + 1) or i]
                    assert(vectors[p], "SetPoints requires constructed vectors")
                    points[#points + 1] = { x = p.x, y = p.y }
                end
                return "replaced", nil
            end,
        }
    end
end

test("curve edit records independent one-based removals and constructed replacements", function()
    reset(); editCurveFixture(false, false)
    local edit = capture("curve-edit ordinary").curveEdit
    assert(#edit.removals == 6 and #edit.replacements == 2)
    for i, index in ipairs({ -1, 0, 1, 2, 3, 4 }) do
        local row = edit.removals[i]
        assert(row.index == index and row.before.count.values[1].value == 3)
        assert(row.before.evaluation.values[1].value == 315)
        local valid = index >= 1 and index <= 3
        assert(row.after.count.values[1].value == (valid and 2 or 3))
        assert(row.mutation.status == (valid and "observed" or "call-error"))
        if valid then
            assert(row.mutation.n == 2 and row.mutation.values[1].kind == "nil")
            assert(row.mutation.values[2].value == "removed")
        end
    end
    assert(edit.removals[3].after.points.values[1].entries[1].fields.x.value == 10)
    local empty, duplicate = edit.replacements[1], edit.replacements[2]
    assert(#empty.inputs == 0 and empty.before.count.values[1].value == 3)
    assert(empty.after.count.values[1].value == 0 and empty.after.evaluation.values[1].value == 15)
    assert(duplicate.inputs[4].x == 10 and duplicate.inputs[4].y == 9)
    assert(duplicate.mutation.n == 2 and duplicate.mutation.values[2].kind == "nil")
    assert(duplicate.after.count.values[1].value == 4)
    assert(duplicate.after.points.values[1].entries[1].fields.x.value == 30)
    assert(duplicate.after.points.values[1].entries[4].fields.y.value == 9)
end)
test("curve edit preserves alternate zero-based indices and replacement order", function()
    reset(); editCurveFixture(true, true)
    local edit = capture("curve-edit alternate").curveEdit
    assert(edit.removals[2].mutation.status == "observed")
    assert(edit.removals[2].after.points.values[1].entries[1].fields.x.value == 10)
    assert(edit.removals[5].mutation.status == "call-error")
    local points = edit.replacements[2].after.points.values[1].entries
    assert(points[1].fields.x.value == 10 and points[1].fields.y.value == 9)
    assert(points[4].fields.x.value == 30)
end)
test("curve edit missing restricted and throwing vector constructors fail closed", function()
    reset(); editCurveFixture(false, false)
    local create = C_CurveUtil.CreateCurve
    C_CurveUtil.CreateCurve = function()
        local curve = create()
        curve.SetPoints = function() error("unsafe replacement reached SetPoints") end
        return curve
    end
    CreateVector2D = nil
    local edit = capture("curve-edit missing").curveEdit
    assert(edit.replacements[1].mutation.status == "missing-vector-constructor")
    assert(edit.replacements[2].after.count.values[1].value == 3)
    CreateVector2D = function() return secret end
    edit = capture("curve-edit restricted").curveEdit
    assert(edit.replacements[2].mutation.status == "restricted-vector")
    CreateVector2D = function() error(secret) end
    edit = capture("curve-edit error").curveEdit
    assert(edit.replacements[2].mutation.status == "vector-construction-error")
    assert(not containsSecret(ApiContractProbeDB))
end)
test("curve edit redacts observations preserves errors and remains manual bounded", function()
    reset(); editCurveFixture(false, false)
    local create = C_CurveUtil.CreateCurve
    C_CurveUtil.CreateCurve = function()
        local curve = create()
        curve.RemovePoint = function() error(secret) end
        curve.SetPoints = function() error(secret) end
        curve.GetPoints = function() return secret end
        curve.Evaluate = function() return secret end
        return curve
    end
    assert(capture("all").curveEdit == nil)
    local edit = capture("curve-edit restricted").curveEdit
    assert(edit.removals[1].mutation.status == "call-error")
    assert(edit.removals[1].before.points.values[1].status == "restricted")
    assert(edit.removals[1].after.evaluation.values[1].status == "restricted")
    assert(edit.replacements[1].mutation.status == "call-error")
    assert(not containsSecret(ApiContractProbeDB))
    for _ = 1, 8 do capture("curve-edit bounded") end
    C_CurveUtil.CreateCurve = function() error("overflow created curve") end
    capture("curve-edit overflow")
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    reset(); issecretvalue = nil
    assert(capture("curve-edit inaccessible").status == "missing-access-api")
end)
test("cast durations retain live and frozen observations without saving objects", function()
    reset()
    local clock = 10
    local methods = { "GetTotalDuration", "GetElapsedDuration", "GetRemainingDuration", "GetElapsedPercent",
        "GetRemainingPercent", "GetStartTime", "GetEndTime", "GetClockTime", "GetModRate", "HasExpired" }
    local function object(frozen)
        local result = {}
        for _, name in ipairs(methods) do result[name] = function() return frozen or clock end end
        return setmetatable(result, { __tostring = function() error("stringified") end,
            __eq = function() error("compared") end })
    end
    UnitCastingDuration = function() return object() end
    UnitChannelDuration = function() return object(clock) end
    UnitEmpoweredChannelDuration = function(_, ...)
        local n = select("#", ...)
        return object(n == 0 and 70 or (... and 90 or 80))
    end
    local first = capture("cast-durations active").castDurations
    assert(first.units.player.casting.values[1].methods.GetClockTime.values[1].value == 10)
    assert(first.units.player.empowerDefault.values[1].methods.GetTotalDuration.values[1].value == 70)
    assert(first.units.player.empowerFalse.values[1].methods.GetTotalDuration.values[1].value == 80)
    assert(first.units.player.empowerTrue.values[1].methods.GetTotalDuration.values[1].value == 90)
    assert(first.retainedCount == 28 and first.retentionDropped == 7)
    clock = 14
    local second = capture("cast-durations later").castDurations
    assert(second.previous[1].observationRef == first.units.player.casting.values[1].observationRef)
    assert(second.previous[1].observation.methods.GetClockTime.values[1].value == 14)
    assert(second.previous[2].observation.methods.GetClockTime.values[1].value == 10)
    local function serializable(value)
        assert(type(value) ~= "function" and type(value) ~= "userdata")
        if type(value) == "table" then
            assert(getmetatable(value) == nil)
            for key, item in pairs(value) do serializable(key); serializable(item) end
        end
    end
    serializable(ApiContractProbeDB)
    local third = capture("cast-durations final").castDurations
    assert(third.previous[1].observationRef == second.units.player.casting.values[1].observationRef)
end)
test("cast duration safety arity and manual limits", function()
    reset()
    local touched = 0
    local broken = setmetatable({}, { __index = function() error(secret) end })
    local duration = { GetClockTime = function() return nil, 7, nil end,
        GetTotalDuration = function() error(secret) end, GetEndTime = secret }
    UnitCastingDuration = function(unit)
        touched = touched + 1
        if unit == "target" then return end
        if unit == "focus" then return secret end
        if unit == "party1" then return broken end
        return duration, nil
    end
    UnitChannelDuration = nil
    UnitEmpoweredChannelDuration = function() return unpack({1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17}) end
    assert(capture("all").castDurations == nil and touched == 0)
    local result = capture("cast-durations safety").castDurations
    assert(touched == 7 and result.units.target.casting.n == 0)
    assert(result.units.focus.casting.values[1].status == "restricted")
    assert(result.units.party1.casting.values[1].methods.GetClockTime.status == "field-error")
    local value = result.units.player.casting.values[1]
    assert(value.methods.GetClockTime.n == 3 and value.methods.GetClockTime.values[3].kind == "nil")
    assert(value.methods.GetTotalDuration.status == "call-error")
    assert(value.methods.GetEndTime.status == "missing-api")
    assert(result.units.player.casting.n == 2 and result.units.player.casting.values[2].kind == "nil")
    assert(result.units.player.channel.status == "missing-api")
    assert(result.units.player.empowerTrue.n == 17 and result.units.player.empowerTrue.truncated)
    assert(#result.units.player.empowerTrue.values == 16 and not containsSecret(ApiContractProbeDB))
    for _ = 1, 8 do capture("cast-durations bounded") end
    UnitCastingDuration = function() error("overflow queried") end
    capture("cast-durations overflow")
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    reset(); issecretvalue = nil
    assert(capture("cast-durations unavailable").status == "missing-access-api")
end)
print(string.format("RESULT %d passed", passed))
