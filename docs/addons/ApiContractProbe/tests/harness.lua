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
    time = function() return 123456 end
    UnitExists = function(unit) return unit == "player" or unit == "target" end
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
print(string.format("RESULT %d passed", passed))
