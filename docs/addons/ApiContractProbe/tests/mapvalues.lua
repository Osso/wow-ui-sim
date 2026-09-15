local root = assert(arg[1], "addon directory required")
local passed = 0
local secret = newproxy(true)
getmetatable(secret).__tostring = function() error("secret stringified") end
local function pack(...) return { n = select("#", ...), ... } end
local function setup(mapper)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(value) return rawequal(value, secret) end
    canaccessvalue = function(value) return not rawequal(value, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 42 end
    mapvalues = mapper
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("mapvalues sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, "manual mapvalues mode absent")
    assert(ApiContractProbeDB.captures[1].label == "sample")
    return assert(ApiContractProbeDB.captures[1].mapvalues)
end
local function caseById(record, id)
    for _, case in ipairs(record.cases) do if case.id == id then return case end end
    error("missing case " .. id)
end
local function test(name, fn) fn(); passed = passed + 1; print("PASS " .. name) end
local function individual(fn, ...)
    local args, results = pack(...), {}
    for i = 1, args.n do results[i] = fn(args[i]) end
    return unpack(results, 1, args.n)
end

test("individual mapping records nil slots, exact tuples and known callback returns", function()
    setup(individual)
    local record = capture()
    assert(#record.cases == 9 and record.status == "observed")
    local zero = caseById(record, "zero-inputs")
    assert(zero.inputs.n == 0 and #zero.invocations == 0 and zero.output.n == 0)
    local one = caseById(record, "one-input")
    assert(one.inputs.n == 1 and one.inputs.values[1].value == 17)
    assert(one.invocations[1].n == 1 and one.invocations[1].values[1].value == 17)
    assert(one.output.n == 1 and one.output.values[1].value == "mapped")
    local many = caseById(record, "multiple-inputs")
    assert(many.inputs.n == 3 and #many.invocations == 3)
    assert(many.invocations[1].values[1].value == 17)
    assert(many.invocations[2].values[1].value == "two")
    assert(many.invocations[3].values[1].value == false)
    local middle = caseById(record, "interior-nil")
    assert(middle.inputs.n == 3 and middle.inputs.values[2].kind == "nil")
    assert(middle.invocations[2].n == 1 and middle.invocations[2].values[1].kind == "nil")
    local trailing = caseById(record, "trailing-nils")
    assert(trailing.inputs.n == 3 and trailing.invocations[3].values[1].kind == "nil")
    assert(caseById(record, "multiple-returns").output.values[1].value == "first")
    local nils = caseById(record, "nil-return")
    assert(nils.output.n == 2 and nils.output.values[2].kind == "nil")
    assert(caseById(record, "zero-returns").output.n == 2)
    local failure = caseById(record, "opaque-throw")
    assert(#failure.invocations == 1 and failure.output.status == "call-error")
end)

test("tuple-stride and reverse traversal observations are not normalized", function()
    setup(function(fn, ...) return fn(...) end)
    local record = capture()
    local zero = caseById(record, "zero-inputs")
    assert(#zero.invocations == 1 and zero.invocations[1].n == 0 and zero.output.n == 1)
    local many = caseById(record, "multiple-inputs")
    assert(#many.invocations == 1 and many.invocations[1].n == 3)
    assert(many.invocations[1].values[2].value == "two" and many.output.n == 1)
    local results = caseById(record, "multiple-returns").output
    assert(results.n == 3 and results.values[1].value == "first")
    assert(results.values[2].kind == "nil" and results.values[3].value == "third")
    assert(caseById(record, "nil-return").output.n == 1)
    assert(caseById(record, "zero-returns").output.n == 0)
    setup(function(fn, ...)
        local args = pack(...)
        for i = args.n, 1, -1 do fn(args[i]) end
        return "outer", nil, "tail", nil
    end)
    local reversed = caseById(capture(), "multiple-inputs")
    assert(reversed.invocations[1].values[1].value == false)
    assert(reversed.invocations[3].values[1].value == 17)
    assert(reversed.output.n == 4 and reversed.output.values[4].kind == "nil")
end)

test("restricted callback arguments and results stay scalar and inaccessible", function()
    local hostile = setmetatable({}, {
        __index = function() error("object indexed") end,
        __eq = function() error("object compared") end,
        __tostring = function() error("object stringified") end,
    })
    setup(function(fn) fn(secret, hostile, nil); return secret, hostile, nil end)
    local originalType = type
    type = function(value)
        assert(not rawequal(value, secret), "secret inspected before access")
        return originalType(value)
    end
    local ok, record = pcall(capture)
    type = originalType
    assert(ok, record)
    for _, case in ipairs(record.cases) do
        assert(case.invocations[1].n == 3)
        assert(case.invocations[1].values[1].status == "restricted")
        assert(case.invocations[1].values[2].kind == "table")
        assert(case.invocations[1].values[2].fields == nil)
        if case.id ~= "opaque-throw" then
            assert(case.output.n == 3 and case.output.values[1].status == "restricted")
            assert(case.output.values[2].kind == "table" and case.output.values[3].kind == "nil")
        end
    end
end)

test("tuple storage retains sixteen positions and exact larger arity", function()
    setup(function(fn)
        local args = {}
        for i = 1, 20 do args[i] = i end
        fn(unpack(args, 1, 20))
        return unpack(args, 1, 20)
    end)
    local record = capture()
    local first = record.cases[1]
    assert(first.invocations[1].n == 20 and first.invocations[1].truncated)
    assert(#first.invocations[1].values == 16 and first.invocations[1].values[16].value == 16)
    assert(first.output.n == 20 and first.output.truncated and #first.output.values == 16)
end)

test("global invocation budget stops further cases even when mapper catches cap errors", function()
    local calls = 0
    setup(function(fn)
        calls = calls + 1
        for i = 1, 40 do pcall(fn, i) end
        return "caught"
    end)
    local record = capture()
    assert(calls == 1 and #record.cases == 1)
    assert(record.status == "invocation-limit" and record.invocationCount == 32)
    assert(#record.cases[1].invocations == 32)
    assert(record.cases[1].output.values[1].value == "caught")
    setup(function(fn) for i = 1, 33 do fn(i) end end)
    local abort = capture()
    assert(abort.status == "invocation-limit" and abort.cases[1].output.status == "call-error")
    setup(function(fn) for i = 1, 10 do pcall(fn, i) end end)
    local shared = capture()
    assert(shared.invocationCount == 32 and #shared.cases == 4)
    assert(#shared.cases[4].invocations == 2 and shared.status == "invocation-limit")
end)

test("missing or restricted mapper and access probes fail closed; errors stay opaque", function()
    setup(nil)
    assert(capture().status == "missing-api")
    local calls = 0
    local fn = function() calls = calls + 1 end
    setup(fn)
    canaccessvalue = function(value) return not rawequal(value, fn) end
    assert(capture().status == "restricted" and calls == 0)
    setup(fn)
    issecretvalue = nil
    SlashCmdList.APICONTRACTPROBE("mapvalues")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api" and calls == 0)
    setup(function() error(secret) end)
    local record = capture()
    for _, case in ipairs(record.cases) do
        assert(case.output.status == "call-error" and case.output.values == nil)
    end
end)

test("retained callbacks cannot append observations after the case completes", function()
    local saved = {}
    setup(function(fn) saved[#saved + 1] = fn; return "deferred" end)
    local record = capture()
    assert(#saved == 9 and record.invocationCount == 0)
    for _, fn in ipairs(saved) do assert(not pcall(fn, "late")) end
    assert(record.invocationCount == 0 and record.status == "observed")
    for _, case in ipairs(record.cases) do assert(#case.invocations == 0) end
end)

test("manual routing and ten snapshot limit do not invoke mapper on excluded calls", function()
    local calls = 0
    setup(function() calls = calls + 1; return 71 end)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(calls == 0 and ApiContractProbeDB.captures[1].mapvalues == nil)
    setup(function() calls = calls + 1; return 71 end)
    for i = 1, 11 do SlashCmdList.APICONTRACTPROBE("mapvalues") end
    assert(#ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    assert(calls == 90)
end)
print(string.format("%d/%d passed", passed, passed))
