local root = assert(arg[1], "addon directory required")
local scenarios = 0

local function observation(run, label)
    for _, row in ipairs(run.observations) do
        if row.label == label then return row end
    end
    error("missing observation: " .. label)
end

local function fixture(options)
    local Probe, objects = {}, {}
    local state = { regions = {}, hidden = false, hideCalls = 0, cancelled = 0, nextCalls = 0 }
    _G.Ptr125RemainingProbeDB = nil
    _G.issecretvalue = function(value) return value == state.secret end
    _G.canaccessvalue = function(value) return value ~= state.secret end
    _G.issecure = function() return false end
    _G.securecallfunction = function(fn, ...) return fn(...) end
    _G.GetBuildInfo = function() return "12.1.5", "69594", "fixture", 120105 end
    _G.GetLocale = function() return "enUS" end
    _G.time = function() return 123456 end
    _G.GetTime = function() return 100 end
    _G.print = function() end
    state.secret = setmetatable({}, { __tostring = function() error("secret stringified") end })
    objects[state.secret] = true
    _G.CreateRegionParams = options.globalTypes and {} or nil
    _G.TimedSignalMapEntry = options.globalTypes and newproxy(true) or nil
    if CreateRegionParams then objects[CreateRegionParams] = true end
    if TimedSignalMapEntry then objects[TimedSignalMapEntry] = true end

    local function create_region(kind, name, layer, template, sublevel)
        assert(state.hidden, "region created before owner hidden")
        local tableForm = type(name) == "table"
        if tableForm then
            if not options.acceptTables then error("fixture rejects table arguments") end
            local fields = name
            name, layer, template, sublevel = fields.name, fields.drawLayer, fields.templateName, fields.subLevel
        end
        local width, height = 0, 0
        if template == "Ptr125RemainingProbeTextureTemplate" then
            assert(kind == "Texture")
            width, height = 17, 19
        elseif template == "Ptr125RemainingProbeFontStringTemplate" then
            assert(kind == "FontString")
            width, height = 23, 29
        else
            assert(template == nil, "unknown fixture template")
        end
        local region = {}
        objects[region] = true
        function region:GetName()
            if options.getterError then error("fixture name access denied") end
            return name
        end
        function region:GetDrawLayer() return layer, sublevel or 0 end
        function region:GetSize() return width, height end
        state.regions[#state.regions + 1] = { kind = kind, tableForm = tableForm, template = template }
        return region
    end

    _G.CreateFrame = options.missingFrame and nil or function(kind, name, parent)
        assert(kind == "Frame" and parent == nil)
        assert(type(name) == "string")
        local frame = {}
        objects[frame] = true
        function frame:Hide()
            state.hideCalls = state.hideCalls + 1
            if options.hideError then error("fixture hide failure") end
            state.hidden = true
        end
        function frame:CreateTexture(...) return create_region("Texture", ...) end
        function frame:CreateFontString(...) return create_region("FontString", ...) end
        if options.optional then
            function frame:CreateLine(...) return create_region("Line", ...) end
            function frame:CreateMaskTexture(...) return create_region("MaskTexture", ...) end
        end
        return frame
    end
    if options.missingFrame then _G.CreateFrame = nil end

    local function factory(callback)
        assert(type(callback) == "function")
        local map = {}
        objects[map] = true
        function map:SignalAt(key, at)
            assert(key == 17 and at == 700)
            if options.signalError then error("fixture schedule failure") end
            self.key, self.at = key, at
        end
        function map:GetNextSignal()
            state.nextCalls = state.nextCalls + 1
            if options.returnKind == "tuple" then return self.key, self.at end
            local result
            if options.returnKind == "userdata" then
                result = newproxy(true)
                getmetatable(result).__index = function(_, key)
                    if options.fieldError then error("fixture field access denied") end
                    if key == "key" then return options.secretFields and state.secret or self.key end
                    if key == "time" then return self.at end
                end
            else
                result = setmetatable({}, { __index = function(_, key)
                    if options.fieldError then error("fixture field access denied") end
                    if key == "key" then return options.secretFields and state.secret or self.key end
                    if key == "time" then return self.at end
                end })
            end
            objects[result] = true
            return result
        end
        function map:CancelAllSignals()
            state.cancelled = state.cancelled + 1
            self.key, self.at = nil, nil
        end
        return map
    end
    _G.C_Timer = options.missingTimer and nil or { NewTimedSignalMap = factory }
    if options.missingTimer then _G.C_Timer = nil end
    assert(loadfile(root .. "/Core.lua"))("Ptr125RemainingProbe", Probe)
    assert(loadfile(root .. "/Structures.lua"))("Ptr125RemainingProbe", Probe)
    assert(loadfile(root .. "/Secrets.lua"))("Ptr125RemainingProbe", Probe)
    _G.SlashCmdList = {}
    assert(loadfile(root .. "/Main.lua"))("Ptr125RemainingProbe", Probe)
    assert(Ptr125RemainingProbeDB == nil, "loading modules must not run probes")
    SlashCmdList.PTR125REMAININGPROBE("structures")
    local run = Ptr125RemainingProbeDB.runs[1]
    assert(run and run.kind == "structures")
    local function assert_redacted(value, seen)
        assert(not objects[value], "owned object persisted")
        local kind = type(value)
        assert(kind ~= "function" and kind ~= "userdata" and kind ~= "thread")
        if kind ~= "table" then return end
        assert(getmetatable(value) == nil and not seen[value], "runtime table persisted")
        seen[value] = true
        for key, child in pairs(value) do
            assert(type(key) == "string" or type(key) == "number")
            assert_redacted(child, seen)
        end
        seen[value] = nil
    end
    assert_redacted(Ptr125RemainingProbeDB, {})
    scenarios = scenarios + 1
    return run, state
end

for _, accepted in ipairs({false, true}) do
    for _, kind in ipairs({"tuple", "table", "userdata"}) do
        local run, state = fixture({acceptTables = accepted, returnKind = kind, optional = true, globalTypes = true})
        assert(run.status == "recorded")
        assert(state.hideCalls == 2 and state.cancelled == 1)
        assert(observation(run, "globals:CreateRegionParams").results[1].kind == "table")
        assert(observation(run, "globals:TimedSignalMapEntry").results[1].kind == "userdata")
        for _, region in ipairs({"Texture", "FontString"}) do
            local prefix = "regions:" .. region
            assert(observation(run, prefix .. ":positional:plain:create").ok)
            assert(observation(run, prefix .. ":positional:template:create").ok)
            assert(observation(run, prefix .. ":table:plain:create").ok == accepted)
            assert(observation(run, prefix .. ":table:template:create").ok == accepted)
            local size = observation(run, prefix .. ":positional:template:GetSize")
            assert(size.returnCount == 2)
            assert(size.results[1].value == (region == "Texture" and 17 or 23))
            assert(size.results[2].value == (region == "Texture" and 19 or 29))
            if accepted then
                local tableSize = observation(run, prefix .. ":table:template:GetSize")
                assert(tableSize.results[1].value == size.results[1].value)
                assert(tableSize.results[2].value == size.results[2].value)
                local layer = observation(run, prefix .. ":table:template:GetDrawLayer")
                assert(layer.results[1].value == "OVERLAY" and layer.results[2].value == 3)
                assert(type(observation(run, prefix .. ":table:template:GetName").results[1].value) == "string")
            end
        end
        assert(observation(run, "regions:Line:positional:plain:create").ok)
        assert(observation(run, "regions:MaskTexture:table:plain:create").ok == accepted)
        local nextSignal = observation(run, "signals:GetNextSignal")
        if kind == "tuple" then
            assert(nextSignal.returnCount == 2)
            assert(nextSignal.results[1].value == 17 and nextSignal.results[2].value == 700)
        else
            assert(nextSignal.returnCount == 1 and nextSignal.results[1].kind == kind)
            assert(observation(run, "signals:entry:key").results[1].value == 17)
            assert(observation(run, "signals:entry:time").results[1].value == 700)
        end
    end
end

local run, state = fixture({missingFrame = true, missingTimer = true})
assert(run.status == "recorded" and state.cancelled == 0)
assert(observation(run, "regions:owner:create").status == "unavailable")
assert(observation(run, "signals:create").status == "unavailable")
assert(observation(run, "globals:CreateRegionParams").results[1].kind == "nil")

run, state = fixture({acceptTables = true, returnKind = "table", getterError = true, fieldError = true})
assert(not observation(run, "regions:Texture:positional:plain:GetName").ok)
assert(not observation(run, "signals:entry:key").ok)
assert(not observation(run, "signals:entry:time").ok)
assert(state.cancelled == 1 and state.hideCalls == 2)
assert(observation(run, "regions:Line:method").results[1].kind == "nil")

run, state = fixture({acceptTables = true, returnKind = "userdata", secretFields = true})
assert(observation(run, "signals:entry:key").results[1].value == nil)
assert(observation(run, "signals:entry:key").results[1].secret == true)
assert(state.cancelled == 1)

run, state = fixture({acceptTables = true, returnKind = "tuple", signalError = true, hideError = true})
assert(not observation(run, "regions:owner:Hide").ok)
assert(#state.regions == 0 and state.hideCalls == 2)
assert(not observation(run, "signals:SignalAt").ok and state.cancelled == 1)
assert(state.nextCalls == 0, "failed scheduling must not masquerade as a populated-map probe")
assert(observation(run, "signals:control").status == "inconclusive")
assert(not observation(run, "cleanup:structures-owner").ok)
io.write("Structures recorder fixtures passed: " .. scenarios .. " scenarios\n")
