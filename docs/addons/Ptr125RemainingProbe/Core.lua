local _, Probe = ...
Probe.actions = {}
local cleanup = {}
local sequence = 0

function Probe.pack(...)
    return { n = select("#", ...), ... }
end

local function observe_boolean(fn, value)
    if type(fn) ~= "function" then return "unavailable" end
    local ok, result = pcall(fn, value)
    if not ok or type(result) ~= "boolean" then return "unknown" end
    return result
end

function Probe.describe(value, includeValue)
    local kind = type(value)
    local secret = observe_boolean(rawget(_G, "issecretvalue"), value)
    local accessible = observe_boolean(rawget(_G, "canaccessvalue"), value)
    local result = { kind = kind, secret = secret, accessible = accessible }
    if not includeValue or secret ~= false or accessible == false then return result end
    if kind == "string" then
        result.value = string.sub(value, 1, 2048)
        result.truncated = #value > 2048
    elseif kind == "boolean" then
        result.value = value
    elseif kind == "number" then
        if value == value and value ~= math.huge and value ~= -math.huge then
            result.value = value
        else
            result.numberClass = "nonfinite"
        end
    end
    return result
end

function Probe.note(run, label, status, message)
    run.observations[#run.observations + 1] = {
        label = label, status = status, message = message,
    }
end

local function invoke(mode, fn, args)
    if mode == "securecallfunction" then
        return securecallfunction(fn, unpack(args, 1, args.n))
    end
    return fn(unpack(args, 1, args.n))
end

function Probe.capture(run, label, mode, fn, args, includeValues)
    local missingWrapper = mode == "securecallfunction" and type(securecallfunction) ~= "function"
    if type(fn) ~= "function" or missingWrapper then
        Probe.note(run, label, "unavailable", "Function or requested native wrapper unavailable")
        return { n = 1, false }
    end
    local raw = Probe.pack(pcall(invoke, mode, fn, args))
    local row = { label = label, mode = mode, ok = raw[1], results = {} }
    if raw[1] then
        row.returnCount = raw.n - 1
        for index = 2, raw.n do
            row.results[index - 1] = Probe.describe(raw[index], includeValues)
        end
    else
        row.error = Probe.describe(raw[2], true)
    end
    run.observations[#run.observations + 1] = row
    return raw
end

function Probe.defer(run, label, fn)
    cleanup[run] = cleanup[run] or {}
    local tasks = cleanup[run]
    tasks[#tasks + 1] = { label = label, fn = fn }
end

local function clean_run(run)
    local tasks = cleanup[run] or {}
    cleanup[run] = nil
    for index = #tasks, 1, -1 do
        local task = tasks[index]
        Probe.capture(run, "cleanup:" .. task.label, "direct", task.fn, Probe.pack(), false)
    end
end

function Probe.next_name(label)
    sequence = sequence + 1
    return "Ptr125RemainingProbe" .. label .. sequence
end

local function database()
    if Ptr125RemainingProbeDB == nil then
        Ptr125RemainingProbeDB = { schema = 1, runs = {} }
    end
    local db = Ptr125RemainingProbeDB
    if type(db) ~= "table" or db.schema ~= 1 or type(db.runs) ~= "table" then
        error("Ptr125RemainingProbe SavedVariables schema is incompatible; preserve and move it aside before retrying")
    end
    return db
end

local function metadata(run)
    local values = Probe.capture(run, "GetBuildInfo", "direct", GetBuildInfo, Probe.pack(), true)
    run.client = {}
    if values[1] then
        run.client.version = Probe.describe(values[2], true).value
        run.client.build = Probe.describe(values[3], true).value
        run.client.interface = Probe.describe(values[5], true).value
    end
    run.target = { version = "12.1.5", build = "69594", interface = 120105 }
    run.matchesPinnedBuild = run.client.version == run.target.version
        and run.client.build == run.target.build and run.client.interface == run.target.interface
    Probe.capture(run, "GetLocale", "direct", GetLocale, Probe.pack(), true)
    Probe.capture(run, "timestamp", "direct", time, Probe.pack(), true)
    Probe.capture(run, "issecure:direct", "direct", issecure, Probe.pack(), true)
    Probe.capture(run, "issecure:securecallfunction", "securecallfunction", issecure, Probe.pack(), true)
end

function Probe.run(kind)
    local action = Probe.actions[kind]
    if type(action) ~= "function" then error("Unknown probe action: " .. tostring(kind)) end
    local db = database()
    local run = { kind = kind, probeVersion = 1, observations = {} }
    metadata(run)
    local result = Probe.capture(run, "action:" .. kind, "direct", action, Probe.pack(run), false)
    run.status = result[1] and (run.status or "recorded") or "error"
    clean_run(run)
    db.runs[#db.runs + 1] = run
    print("Ptr125RemainingProbe: recorded " .. kind .. " run " .. #db.runs .. " (" .. run.status .. ")")
    if not run.matchesPinnedBuild then
        print("Ptr125RemainingProbe: build differs from pinned target; results are not pinned-build proof")
    end
    return run
end

function Probe.status()
    local db = database()
    print("Ptr125RemainingProbe: " .. #db.runs .. " recorded runs; /reload or logout to save")
end
