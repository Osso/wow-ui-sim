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
_G.canaccessvalue = nil
assert(Probe.describe("unknown access must not copy", true).value == nil)
_G.canaccessvalue = function(value) return value ~= secret end
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
_G.SlashCmdList = {}
Probe.actions.structures = function(record)
    Probe.note(record, "CLI fixture", "observed", "structure action selected")
end
Probe.actions.secrets = function() error(secret) end
assert(loadfile(root .. "/Main.lua"))("Ptr125RemainingProbe", Probe)
assert(#Ptr125RemainingProbeDB.runs == 2, "probe ran automatically")
SlashCmdList.PTR125REMAININGPROBE(" STRUCTURES")
assert(#Ptr125RemainingProbeDB.runs == 3)
assert(Ptr125RemainingProbeDB.runs[3].kind == "structures")
SlashCmdList.PTR125REMAININGPROBE("secrets")
assert(Ptr125RemainingProbeDB.runs[4].status == "error")
SlashCmdList.PTR125REMAININGPROBE("status")
SlashCmdList.PTR125REMAININGPROBE("unknown")
assert(#Ptr125RemainingProbeDB.runs == 4)
assert_serializable(Ptr125RemainingProbeDB, {})
_G.canaccessvalue = nil
local unknownBuild = Probe.run("fixture")
assert(unknownBuild.matchesPinnedBuild == "unknown")
assert(unknownBuild.client.version == nil and unknownBuild.client.build == nil)
assert_serializable(Ptr125RemainingProbeDB, {})
_G.canaccessvalue = function(value) return value ~= secret end
local uiErrors = 0
local insideWrapper = false
_G.securecallfunction = function(fn, ...)
    insideWrapper = true
    local returned = Probe.pack(pcall(fn, ...))
    insideWrapper = false
    if not returned[1] then
        uiErrors = uiErrors + 1
        return
    end
    return unpack(returned, 2, returned.n)
end

local wrappedRun = { observations = {} }
local successInside = false
local success = Probe.capture(wrappedRun, "wrapped tuple", "securecallfunction", function(...)
    successInside = insideWrapper
    local args = Probe.pack(...)
    assert(args.n == 3 and args[1] == "argument" and args[2] == nil and args[3] == 7)
    return nil, "public", nil, 7, nil
end, Probe.pack("argument", nil, 7), true)
assert(successInside, "successful callback ran outside secure wrapper")
assert(success.n == 6 and success[1] == true and success[2] == nil)
assert(success[3] == "public" and success[4] == nil and success[5] == 7 and success[6] == nil)
local tuple = wrappedRun.observations[1]
assert(tuple.ok and tuple.returnCount == 5 and #tuple.results == 5)
assert(tuple.results[1].kind == "nil" and tuple.results[2].value == "public")
assert(tuple.results[3].kind == "nil" and tuple.results[4].value == 7)
assert(tuple.results[5].kind == "nil")
assert(uiErrors == 0)

local rejection = "attempted to perform indexed assignment on a table that cannot be indexed with secret keys"
local rejectedInside = false
local rejected = Probe.capture(wrappedRun, "wrapped rejected key", "securecallfunction", function()
    rejectedInside = insideWrapper
    error(rejection, 0)
end, Probe.pack(), true)
local secretInside = false
local secretFailure = Probe.capture(wrappedRun, "wrapped secret error", "securecallfunction", function()
    secretInside = insideWrapper
    error(secret, 0)
end, Probe.pack(), true)
assert(rejectedInside and secretInside, "failing callback ran outside secure wrapper")
assert(uiErrors == 0, "callback errors escaped to the secure wrapper UI handler")
assert(rejected.n == 2 and rejected[1] == false and rejected[2] == rejection)
local rejectedRow = wrappedRun.observations[2]
assert(rejectedRow.ok == false and #rejectedRow.results == 0)
assert(rejectedRow.error.kind == "string" and rejectedRow.error.value == rejection)
assert(rejectedRow.error.secret == false and rejectedRow.error.accessible == true)
assert(secretFailure.n == 2 and secretFailure[1] == false and secretFailure[2] == secret)
local secretRow = wrappedRun.observations[3]
assert(secretRow.ok == false and #secretRow.results == 0)
assert(secretRow.error.secret == true and secretRow.error.accessible == false)
assert(secretRow.error.value == nil)
assert_serializable(wrappedRun, {})
io.write("Core recording/redaction/cleanup/build-tag/CLI/wrapper fixtures passed\n")
