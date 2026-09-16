local root = assert(arg[1])
local passed, failed = 0, 0
local corpus = {
    "|Hitem:19019", "|Hitem:1|h[outer |Hitem:2|h[inner]|h]|h",
    "|cffff0000outer|cff00ff00inner|r outer|r", "|A:atlas:16:16|Ttexture:16:16|a|t",
    "|r|a|t", "before\0|Hitem:1|h[x]|h\0after",
    "\1\9\13\31|Hitem:1|h[x]|h\127", "\128\255|Hitem:1|h[x]|h\254",
}
local text = "|cffff0000|Hitem:19019|h[Item]|h|r |A:atlas:16:16|a |Ttexture:16:16|t"
local function setup(fn)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function() return false end
    canaccessvalue = function() return true end
    GetBuildInfo = function() return 'fixture' end
    time = function() return 1 end
    C_StringUtil = { StripHyperlinks = fn }
    local toc = assert(io.open(root .. '/ApiContractProbe.toc'))
    for line in toc:lines() do if line:match('%.lua$') then assert(loadfile(root .. '/' .. line))() end end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE('hyperlinks-residual ' .. (label or 'fixture'))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures == 1, 'manual residual mode absent')
    return assert(ApiContractProbeDB.captures[1].hyperlinksResidual)
end
local function test(name, fn)
    local ok, err = pcall(fn)
    if ok then passed = passed + 1; print('PASS ' .. name)
    else failed = failed + 1; print('FAIL ' .. name .. ': ' .. tostring(err)) end
end

test('all matrix arguments and distinct byte corpus', function()
    local calls, identities = 0, {}
    setup(function(...)
        calls = calls + 1
        local a = { ... }
        if calls <= 35 then
            assert(select('#', ...) == 6 and a[1] == text)
            local position = math.floor((calls - 1) / 7) + 2
            local variant = (calls - 1) % 7 + 1
            for i = 2, 6 do
                if i ~= position then assert(a[i] == false)
                elseif variant == 1 then assert(a[i] == nil)
                elseif variant == 2 then assert(a[i] == 0)
                elseif variant == 3 then assert(a[i] == 1)
                elseif variant == 4 then assert(a[i] == '')
                elseif variant == 5 then assert(a[i] == 'x')
                else
                    assert(type(a[i]) == (variant == 6 and 'table' or 'function'))
                    if identities[variant] then assert(rawequal(a[i], identities[variant])) end
                    identities[variant] = a[i]
                end
            end
        else assert(select('#', ...) == 1 and a[1] == corpus[calls - 35]) end
        return calls, nil, 'raw'
    end)
    local rows = capture()
    assert(calls == 43 and #rows == 43)
    for i, row in ipairs(rows) do
        assert(row.argumentCount == (i <= 35 and 6 or 1))
        assert(row.result.n == 3 and row.result.values[1].value == i and row.result.values[2].kind == 'nil')
        assert(row.arguments.n == row.argumentCount)
    end
end)

test('errors zero returns nil holes and output limits remain independent', function()
    local calls = 0
    setup(function()
        calls = calls + 1
        if calls == 1 then error({}) end
        if calls == 2 then return end
        if calls == 3 then return nil, 'x', nil end
        return unpack({string.rep('x', 300),2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17})
    end)
    local r = capture()
    assert(calls == 43 and r[1].result.status == 'call-error' and r[2].result.n == 0)
    assert(r[3].result.n == 3 and r[3].result.values[3].kind == 'nil')
    assert(r[4].result.n == 17 and r[4].result.truncated and #r[4].result.values == 16)
    assert(#r[4].result.values[1].value == 256)
end)

test('function guards and namespace guards block invocation', function()
    for _, phase in ipairs({'namespace', 'secret', 'access'}) do
        local calls = 0
        local fn = function() calls = calls + 1 end
        setup(fn)
        if phase == 'namespace' then canaccessvalue = function(v) return v ~= C_StringUtil end
        elseif phase == 'secret' then issecretvalue = function(v) return v == fn end
        else canaccessvalue = function(v) return v ~= fn end end
        local r = capture(); assert(calls == 0 and #r == 43)
    end
end)

test('every optional input kind denied after function access', function()
    for _, kind in ipairs({'nil','number','string','table','function','boolean'}) do
        local armed, calls = false, 0
        local fn
        fn = function(...) calls = calls + 1; for i=1,select('#',...) do assert(type(select(i,...)) ~= kind) end end
        setup(fn)
        canaccessvalue = function(v)
            if v == fn then armed = true; return true end
            return not (armed and type(v) == kind)
        end
        local r = capture(); assert(#r == 43)
        if kind == 'string' then assert(calls == 0) end
    end
end)

test('lookup and function guards revoke original text before invocation', function()
    for _, phase in ipairs({'lookup','secret','access'}) do
        local revoked, calls = false, 0
        local fn = function(s) assert(s ~= text); calls = calls + 1 end
        setup(fn)
        canaccessvalue = function(v) if phase == 'access' and v == fn then revoked = true end; return not (revoked and v == text) end
        issecretvalue = function(v) if phase == 'secret' and v == fn then revoked = true end; return false end
        if phase == 'lookup' then C_StringUtil = setmetatable({}, {__index=function() revoked=true; return fn end}) end
        local r = capture(); assert(calls == 8 and r[1].result.status == 'restricted-input')
    end
end)

test('owned objects are not retained or invoked by recorder', function()
    local refs = setmetatable({}, {__mode='v'})
    setup(function(...)
        for i=2,select('#',...) do local v=select(i,...); if type(v)=='table' then refs[1]=v elseif type(v)=='function' then refs[2]=v end end
    end)
    capture(); collectgarbage('collect'); collectgarbage('collect')
    assert(refs[1] == nil and refs[2] == nil)
end)

test('snapshot label caps and exclusion from all', function()
    local calls=0
    setup(function() calls=calls+1 end)
    for i=1,11 do SlashCmdList.APICONTRACTPROBE('hyperlinks-residual '..string.rep('z',200)) end
    assert(calls==430 and #ApiContractProbeDB.captures==10 and ApiContractProbeDB.dropped==1)
    assert(#ApiContractProbeDB.captures[1].label==128)
    setup(function() error('residual called from all') end)
    SlashCmdList.APICONTRACTPROBE('all fixture')
    assert(ApiContractProbeDB.captures[1].hyperlinksResidual==nil)
end)

print(string.format('%d passed, %d failed',passed,failed))
os.exit(failed == 0 and 0 or 1)
