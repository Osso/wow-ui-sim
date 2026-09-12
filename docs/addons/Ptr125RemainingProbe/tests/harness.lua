local root = assert(arg[1], "addon directory required")
local Probe = {}
local secret = setmetatable({}, { __tostring = function() error("secret stringified") end })
_G.issecretvalue = function(value) return value == secret end
_G.canaccessvalue = function(value) return value ~= secret end
_G.issecure = function() return false end
_G.securecallfunction = function(fn, ...) return fn(...) end
_G.GetBuildInfo = function() return "12.1.5", "69594", "fixture", 120105 end
_G.GetLocale = function() return "enUS" end
_G.time = function() return 123456 end
_G.print = function() end
_G.Ptr125RemainingProbeDB = nil

assert(loadfile(root .. "/Core.lua"))("Ptr125RemainingProbe", Probe)

local run = { observations = {} }
local result = Probe.capture(run, "redaction", "direct", function()
    return secret, "public", nil, 7
end, Probe.pack(), true)
assert(result.n == 5 and result[1] == true)
local observation = run.observations[1]
assert(observation.ok and observation.returnCount == 4)
assert(observation.results[1].secret == true and observation.results[1].value == nil)
assert(observation.results[2].value == "public")
assert(observation.results[3].kind == "nil")
assert(observation.results[4].value == 7)
Probe.capture(run, "secret error", "direct", function() error(secret) end, Probe.pack(), true)
assert(run.observations[2].ok == false)
assert(run.observations[2].error.secret == true)
assert(run.observations[2].error.value == nil)

_G.issecretvalue = nil
Probe.capture(run, "unknown secrecy", "direct", function() return "do not copy" end, Probe.pack(), true)
assert(run.observations[3].results[1].secret == "unavailable")
assert(run.observations[3].results[1].value == nil)
_G.issecretvalue = function(value) return value == secret end
_G.securecallfunction = nil
Probe.capture(run, "missing secure wrapper", "securecallfunction", function() error("must not run") end, Probe.pack())
assert(run.observations[4].status == "unavailable")
_G.securecallfunction = function(fn, ...) return fn(...) end

local cleaned = false
Probe.actions.fixture = function(record)
    Probe.defer(record, "fixture", function() cleaned = true end)
    Probe.capture(record, "fixture secret", "direct", function() return secret end, Probe.pack(), true)
    error("fixture failure")
end
local recorded = Probe.run("fixture")
assert(cleaned and recorded.status == "error")
assert(recorded.matchesPinnedBuild == true)
assert(recorded.client.version == "12.1.5" and recorded.client.build == "69594")
assert(#Ptr125RemainingProbeDB.runs == 1)

local function assert_serializable(value, seen)
    assert(value ~= secret, "raw secret persisted")
    local kind = type(value)
    assert(kind ~= "function" and kind ~= "userdata" and kind ~= "thread")
    if kind == "table" then
        assert(getmetatable(value) == nil, "runtime object persisted")
        assert(not seen[value], "cycle persisted")
        seen[value] = true
        for key, child in pairs(value) do
            assert(type(key) == "string" or type(key) == "number")
            assert_serializable(child, seen)
        end
        seen[value] = nil
    end
end
assert_serializable(Ptr125RemainingProbeDB, {})
_G.GetBuildInfo = function() return "12.1.5", "99999", "fixture", 120105 end
assert(Probe.run("fixture").matchesPinnedBuild == false)
assert(#Ptr125RemainingProbeDB.runs == 2)
io.write("Core recording/redaction/cleanup/build-tag fixtures passed\n")
