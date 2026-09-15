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
    assert(record.sex.units.focus.status == "absent")
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
print(string.format("RESULT %d passed", passed))
