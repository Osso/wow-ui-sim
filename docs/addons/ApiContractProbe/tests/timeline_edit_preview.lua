local root = assert(arg[1], "addon directory required")
local passed, total = 0, 0
local baseMeta = getmetatable(_G)
local function pack(...) return { n = select("#", ...), ... } end
local function opaque()
    local value = newproxy(true)
    getmetatable(value).__index = function() error("opaque lookup") end
    getmetatable(value).__tostring = function() error("opaque stringify") end
    return value
end
local function setup()
    setmetatable(_G, baseMeta)
    ApiContractProbeDB, SlashCmdList = nil, {}
    local f = { calls = {}, counts = {}, hooks = {}, denied = {}, active = false,
        count = 0, ids = {}, source = 47.25 }
    issecretvalue = function(value)
        if f.guard then f.guard(value, "secret") end
        return f.denied[value] == true
    end
    canaccessvalue = function(value)
        if f.guard then f.guard(value, "access") end
        return f.denied[value] ~= true
    end
    GetBuildInfo = function() return "fixture", "build" end
    time = function() return 19 end
    GetTime = function() return 19 end
    GetLocale = function() return "fixture" end
    Enum = { EncounterTimelineEventSource = { EditMode = f.source } }
    f.manager = {}
    local function call(name, ...)
        local args = pack(...)
        f.calls[#f.calls + 1] = { name = name, args = args }
        f.counts[name] = (f.counts[name] or 0) + 1
        if f.onCall then f.onCall(name, f.counts[name]) end
        if f.hooks[name] then return f.hooks[name](f, args, f.counts[name]) end
        if name == "active" then
            assert(args.n == 1 and rawequal(args[1], f.manager), "original receiver")
            return f.active
        end
        if name == "count" then
            assert(args.n == 1 and args[1] == f.source, "original published source")
            return f.count
        end
        assert(args.n == 0, "zero argument timeline call")
        if name == "add" then f.count, f.ids = 2, { 100.25, 102.75 }; return 8 end
        if name == "cancel" then f.count, f.ids = 0, {}; return end
        if name == "list" then return f.ids end
        error("unknown fixture API")
    end
    f.manager.IsEditModeActive = function(...) return call("active", ...) end
    EditModeManagerFrame = f.manager
    C_EncounterTimeline = {
        GetEventCountBySource = function(...) return call("count", ...) end,
        GetEventList = function(...) return call("list", ...) end,
        AddEditModeEvents = function(...) return call("add", ...) end,
        CancelEditModeEvents = function(...) return call("cancel", ...) end,
    }
    f.api = C_EncounterTimeline
    C_Timer = { NewTimer = function() error("timer forbidden") end,
        NewTicker = function() error("ticker forbidden") end }
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
    return f
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("timeline-edit-preview " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "preview mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].timelineEditPreview,
        "preview record absent")
end
local function count(f, name) return f.counts[name] or 0 end
local function assertLocked(f)
    local before = #f.calls
    ApiContractProbeDB = nil
    assert(capture().status == "blocked-cleanup-unconfirmed", "persistent session lock")
    assert(#f.calls == before, "locked capture must not call backend")
end
local function test(name, fn)
    total = total + 1
    local ok, err = pcall(fn)
    setmetatable(_G, baseMeta)
    if ok then passed = passed + 1; print("PASS " .. name)
    else print("FAIL " .. name .. ": " .. tostring(err)) end
end

test("two gates one add one cleanup and independent final reads follow exact sequence", function()
    local f = setup()
    local r = capture()
    local names = { "active", "count", "active", "count", "add", "count", "list",
        "active", "cancel", "count", "list", "active" }
    assert(#f.calls == #names)
    for i, name in ipairs(names) do assert(f.calls[i].name == name, "call ordering") end
    assert(r.baseline.active.values[1].value == false and r.baseline.count.values[1].value == 0)
    assert(r.beforeAdd.active.values[1].value == false and r.beforeAdd.count.values[1].value == 0)
    assert(r.post.count.values[1].value == 2)
    assert(r.post.list.entries[1].value == 100.25)
    assert(r.post.list.entries[2].value == 102.75)
    assert(r.cleanup.cancel.n == 0, "zero-return cleanup accepted")
    assert(r.cleanup.status == "confirmed-by-observation" and r.status == "observed")
    assert(not r.locked and f.count == 0)
    assert(capture().status == "observed" and count(f, "add") == 2)
end)

test("unknown active or count baseline skips every mutator without guessing", function()
    for _, name in ipairs({ "active", "count" }) do
        for _, kind in ipairs({ "nil", "none", "string", "wrong", "nan", "infinity", "secret", "error" }) do
            local f, hidden = setup(), opaque()
            f.denied[hidden] = true
            f.hooks[name] = function()
                if kind == "nil" then return nil end
                if kind == "none" then return end
                if kind == "string" then return "0" end
                if kind == "wrong" then return name == "active" and true or 1 end
                if kind == "nan" then return 0 / 0 end
                if kind == "infinity" then return math.huge end
                if kind == "secret" then return hidden end
                error(hidden)
            end
            local r = capture()
            assert(r.status == "gate-unavailable")
            assert(count(f, "add") == 0 and count(f, "cancel") == 0)
        end
    end
end)

test("missing restricted or invalid publication and original receiver fail closed", function()
    for _, kind in ipairs({ "missing-frame", "secret-frame", "missing-method", "secret-method",
        "missing-enum", "secret-enum", "invalid-member", "secret-member", "missing-add", "missing-cancel" }) do
        local f = setup()
        if kind == "missing-frame" then EditModeManagerFrame = nil
        elseif kind == "secret-frame" then f.denied[f.manager] = true
        elseif kind == "missing-method" then f.manager.IsEditModeActive = nil
        elseif kind == "secret-method" then f.denied[f.manager.IsEditModeActive] = true
        elseif kind == "missing-enum" then Enum.EncounterTimelineEventSource = nil
        elseif kind == "secret-enum" then f.denied[Enum.EncounterTimelineEventSource] = true
        elseif kind == "invalid-member" then Enum.EncounterTimelineEventSource.EditMode = "EditMode"
        elseif kind == "secret-member" then f.denied[f.source] = true
        elseif kind == "missing-add" then f.api.AddEditModeEvents = nil
        else f.api.CancelEditModeEvents = nil end
        capture()
        assert(count(f, "add") == 0 and count(f, "cancel") == 0, kind)
    end
end)

test("second gate observes state changed during mutator lookup and skips all mutations", function()
    for _, changed in ipairs({ "active", "count" }) do
        local f = setup()
        local add = f.api.AddEditModeEvents
        f.api.AddEditModeEvents = nil
        setmetatable(f.api, { __index = function(_, key)
            if key == "AddEditModeEvents" then
                if changed == "active" then f.active = true else f.count = 3 end
                return add
            end
        end })
        local r = capture()
        assert(r.baseline.active.values[1].value == false)
        assert(r.baseline.count.values[1].value == 0)
        assert(r.beforeAdd and r.status == "gate-unavailable")
        assert(count(f, "active") == 2 and count(f, "count") == 2)
        assert(count(f, "add") == 0 and count(f, "cancel") == 0)
    end
end)

test("original enum survives publication replacement and revocation blocks forwarding", function()
    local f = setup()
    f.onCall = function(name)
        if name == "active" then Enum.EncounterTimelineEventSource.EditMode = 999 end
    end
    assert(capture().status == "observed")
    for _, phase in ipairs({ "secret", "access" }) do
        for _, victim in ipairs({ "source", "receiver" }) do
            f = setup()
            local target = victim == "source" and f.api.GetEventCountBySource or f.manager.IsEditModeActive
            f.guard = function(value, currentPhase)
                if rawequal(value, target) and phase == currentPhase then
                    f.denied[victim == "source" and f.source or f.manager] = true
                end
            end
            capture()
            assert(count(f, "add") == 0 and count(f, "cancel") == 0)
            assert(count(f, victim == "source" and "count" or "active") == 0)
        end
    end
end)

test("add throwing after side effect still gets one cleanup and persistent uncertainty lock", function()
    local f, hidden = setup(), opaque()
    f.hooks.add = function(self)
        self.count, self.ids = 1, { 78 }
        error(hidden)
    end
    local r = capture()
    assert(count(f, "add") == 1 and count(f, "cancel") == 1 and f.count == 0)
    assert(r.add.status == "call-error" and r.locked)
    assertLocked(f)
end)

test("post count or list failure cannot suppress cleanup or independent final reads", function()
    for _, name in ipairs({ "count", "list" }) do
        local f = setup()
        f.hooks[name] = function(self, _, n)
            if n == (name == "count" and 3 or 1) then error("post observation") end
            if name == "count" then return self.count end
            return self.ids
        end
        local r = capture()
        assert(r.post[name].status == "call-error")
        assert(count(f, "cancel") == 1 and r.locked)
        assert(r.final.count.status == "observed" and r.final.list.status == "observed")
        assert(count(f, "active") == 4)
        assertLocked(f)
    end
end)

test("new active Edit Mode skips broad cancellation and locks future attempts", function()
    for _, where in ipairs({ "add", "cancel-lookup" }) do
        local f = setup()
        if where == "add" then
            f.hooks.add = function(self) self.active, self.count = true, 5; return 8 end
        else
            local cancel = f.api.CancelEditModeEvents
            f.hooks.add = function(self)
                self.count = 5
                self.api.CancelEditModeEvents = nil
                setmetatable(self.api, { __index = function(_, key)
                    if key == "CancelEditModeEvents" then self.active = true; return cancel end
                end })
                return 8
            end
        end
        local r = capture()
        assert(count(f, "add") == 1 and count(f, "cancel") == 0)
        assert(r.cleanup.cancel.status == "skipped-active-edit-mode" and r.locked)
        assert(r.final.count and r.final.list and r.final.active)
        assertLocked(f)
    end
end)

test("state checks use the actual manager after replacement before add or cleanup", function()
    for _, boundary in ipairs({ "before-add", "after-add" }) do
        local f = setup()
        local replacement = {}
        replacement.IsEditModeActive = function(self)
            assert(rawequal(self, replacement))
            f.calls[#f.calls + 1] = { name = "active", args = pack(self) }
            f.counts.active = count(f, "active") + 1
            return true
        end
        if boundary == "before-add" then
            local add = f.api.AddEditModeEvents
            f.api.AddEditModeEvents = nil
            setmetatable(f.api, { __index = function(_, key)
                if key == "AddEditModeEvents" then EditModeManagerFrame = replacement; return add end
            end })
        else
            f.hooks.add = function(self)
                self.count = 4
                EditModeManagerFrame = replacement
                return 8
            end
        end
        local r = capture()
        assert(count(f, "cancel") == 0, "must not cancel newly active manager events")
        if boundary == "before-add" then
            assert(count(f, "add") == 0 and r.status == "gate-unavailable")
        else
            assert(count(f, "add") == 1 and r.locked)
            assert(r.cleanup.cancel.status == "skipped-active-edit-mode")
            assertLocked(f)
        end
    end
end)

test("cleanup unknown state still records one cancel attempt but never confirms or retries", function()
    local f = setup()
    f.hooks.active = function(self, _, n)
        if n == 3 then error("cleanup state unavailable") end
        return self.active
    end
    local r = capture()
    assert(count(f, "cancel") == 1 and r.locked)
    assert(r.cleanup.active.status == "call-error")
    assertLocked(f)
end)

test("cancel errors restricted outputs and missing function leave cleanup unconfirmed", function()
    for _, kind in ipairs({ "error", "secret-output", "secret-function", "missing", "nonzero" }) do
        local f, hidden = setup(), opaque()
        f.denied[hidden] = true
        if kind == "secret-function" or kind == "missing" then
            f.hooks.add = function(self)
                self.count = 1
                if kind == "missing" then self.api.CancelEditModeEvents = nil
                else self.denied[self.api.CancelEditModeEvents] = true end
                return 8
            end
        else
            f.hooks.cancel = function(self)
                if kind ~= "nonzero" then self.count, self.ids = 0, {} end
                if kind == "error" then error(hidden) end
                if kind == "secret-output" then return hidden end
            end
        end
        local r = capture()
        assert(r.locked and r.cleanup.status == "unconfirmed")
        assert(count(f, "cancel") <= 1)
        assert(r.final.count and r.final.list and r.final.active)
        assertLocked(f)
    end
end)

test("each uncertain final observation locks despite successful zero-return cancel", function()
    for _, kind in ipairs({ "count-error", "count-secret", "count-nonzero", "list-error", "list-secret", "active-true", "active-nil" }) do
        local f, hidden = setup(), opaque()
        f.denied[hidden] = true
        f.hooks.count = function(self, _, n)
            if n == 4 then
                if kind == "count-error" then error(hidden) end
                if kind == "count-secret" then return hidden end
                if kind == "count-nonzero" then return 1 end
            end
            return self.count
        end
        f.hooks.list = function(self, _, n)
            if n == 2 then
                if kind == "list-error" then error(hidden) end
                if kind == "list-secret" then return hidden end
            end
            return self.ids
        end
        f.hooks.active = function(self, _, n)
            if n == 4 then
                if kind == "active-true" then return true end
                if kind == "active-nil" then return nil end
            end
            return self.active
        end
        local r = capture()
        assert(count(f, "cancel") == 1 and r.locked, kind)
        assert(r.final.count and r.final.list and r.final.active)
        assertLocked(f)
    end
end)

test("bounded lists retain nil positions and scalars without traversing opaque entries", function()
    local f = setup()
    f.hooks.list = function()
        return setmetatable({ [1] = 18.5, [3] = 19.5 }, { __index = function(_, key)
            assert(type(key) == "number" and key <= 8, "unbounded list lookup")
        end, __len = function() error("length forbidden") end })
    end
    local r = capture()
    assert(r.post.list.entries[1].value == 18.5 and r.post.list.entries[2].kind == "nil")
    assert(r.post.list.entries[3].value == 19.5 and r.post.list.entries[9] == nil)
    assert(r.status == "observed")
    f = setup()
    f.hooks.list = function() return { opaque() } end
    r = capture()
    assert(r.post.list.entries[1].kind == "userdata")
    assert(r.locked and count(f, "cancel") == 1)
end)

test("raw tuples strings labels snapshots and all exclusion obey caps", function()
    local f = setup()
    f.hooks.add = function(self) self.count = 1; return string.rep("x", 300), nil, false end
    local r = capture(string.rep("L", 200))
    assert(r.add.n == 3 and r.add.values[2].kind == "nil" and r.add.values[3].value == false)
    assert(#r.add.values[1].value == 256 and r.add.values[1].truncated)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    -- Truncated observations cannot establish an unqualified cleanup capture.
    assert(r.locked)
    f = setup()
    for _ = 1, 11 do capture() end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(count(f, "add") == 10 and count(f, "cancel") == 10 and #f.calls == 120)
    f = setup()
    SlashCmdList.APICONTRACTPROBE("all")
    assert(count(f, "add") == 0 and count(f, "cancel") == 0)
end)

test("missing access APIs cause no calls and final snapshots retain no raw objects", function()
    for _, name in ipairs({ "issecretvalue", "canaccessvalue" }) do
        local f = setup()
        _G[name] = nil
        SlashCmdList.APICONTRACTPROBE("timeline-edit-preview")
        assert(#f.calls == 0 and ApiContractProbeDB.captures[1].status == "missing-access-api")
    end
    local f = setup()
    local weak = setmetatable({}, { __mode = "v" })
    f.hooks.list = function()
        local list = { 123 }
        weak[#weak + 1] = list
        return list
    end
    capture()
    collectgarbage(); collectgarbage()
    assert(weak[1] == nil and weak[2] == nil, "raw lists retained")
end)

print(("timeline edit preview fixtures: %d/%d passed"):format(passed, total))
os.exit(passed == total and 0 or 1)
