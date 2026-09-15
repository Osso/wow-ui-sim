local root = assert(arg[1])
local secret = newproxy(true)
local passed = 0
local function setup(alias, opaque)
    ApiContractProbeDB, SlashCmdList = nil, {}
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo = function() return "fixture", "123" end
    time = function() return 1 end
    local hostile = { __index = function() error("returned lookup") end,
        __tostring = function() error("returned tostring") end }
    CreateColor = function(r,g,b,a) return setmetatable({r=r,g=g,b=b,a=a}, hostile) end
    local function create()
        local c = { points = {}, value = 37 }
        function c:AddPoint(x,y) self.points[#self.points+1] = setmetatable({x=x,y=y}, hostile) end
        function c:GetType() return self.value end
        function c:GetPointCount() return #self.points end
        function c:HasSecretValues() return false end
        function c:GetPoints() return self.points end
        function c:GetPoint(i) return self.points[i+1] end
        function c:Evaluate(x) return opaque or CreateColor(x, secret, 0.25, 0.75) end
        function c:EvaluateUnpacked(x) return x, nil, secret, 0.75 end
        function c:SetToDefaults() self.value=91; self.points={} end
        function c:ClearPoints() self.points={} end
        function c:Copy()
            if alias then return self end
            local other=create()
            for i,p in ipairs(self.points) do other.points[i]=p end
            return other
        end
        return c
    end
    C_CurveUtil = { CreateColorCurve = create }
    local toc=assert(io.open(root.."/ApiContractProbe.toc"))
    for line in toc:lines() do if line:match("%.lua$") then assert(loadfile(root.."/"..line))() end end
    toc:close()
end
local function capture()
    SlashCmdList.APICONTRACTPROBE("color-curves sample")
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures==1, "manual color mode absent")
    return ApiContractProbeDB.captures[1].colorCurves
end
local function test(name, fn) fn(); passed=passed+1; print("PASS "..name) end

test("raw point fields, zero-based indices, evaluation nils and secret channels",function()
    setup(false)
    local c=capture()
    assert(c.empty.count.values[1].value==0)
    assert(c.populated.count.values[1].value==4)
    assert(c.inputs[3].x==-16 and c.inputs[4].x==48)
    local first=c.populated.points.values[1].entries[1]
    assert(first.fields.x.value==0 and first.fields.y.fields.g.value==1)
    assert(c.populated.indices[1].result.values[1].kind=="nil")
    assert(c.populated.indices[2].result.values[1].fields.x.value==0)
    assert(c.populated.evaluations[1].x==-17)
    local e=c.populated.evaluations[1]
    assert(e.packed.values[1].fields.r.value==-17)
    assert(e.packed.values[1].fields.g.status=="restricted")
    assert(e.unpacked.n==4 and e.unpacked.values[2].kind=="nil")
    assert(e.unpacked.values[3].status=="restricted")
    assert(c.defaults.after.curveType.values[1].value==91)
    assert(c.copy.afterAdd.original.count.values[1].value==4)
    assert(c.copy.afterAdd.copy.count.values[1].value==5)
    assert(c.copy.afterClear.original.count.values[1].value==4)
    assert(c.copy.afterClear.copy.count.values[1].value==0)
end)
test("alias copy and opaque userdata remain raw observations",function()
    local opaque=newproxy(true)
    getmetatable(opaque).__index=function() error("userdata indexed") end
    setup(true,opaque)
    local c=capture()
    assert(c.populated.evaluations[1].packed.values[1].kind=="userdata")
    assert(c.populated.evaluations[1].packed.values[1].fields==nil)
    assert(c.copy.afterAdd.original.count.values[1].value==5)
    assert(c.copy.afterClear.original.count.values[1].value==0)
end)
test("missing constructors, opaque errors, manual routing and storage bound",function()
    setup(false)
    C_CurveUtil.CreateColorCurve=nil
    assert(capture().status=="missing-constructor")
    setup(false)
    CreateColor=function() error(secret) end
    assert(capture().status=="color-construction-error")
    setup(false)
    local count=0
    C_CurveUtil.CreateColorCurve=function() count=count+1; error(secret) end
    for i=1,11 do SlashCmdList.APICONTRACTPROBE("color-curves") end
    assert(#ApiContractProbeDB.captures==10 and ApiContractProbeDB.dropped==1 and count==10)
    setup(false)
    SlashCmdList.APICONTRACTPROBE("all")
    assert(ApiContractProbeDB.captures[1].colorCurves==nil)
end)
test("differing defaults, order and long tuples remain bounded observations", function()
    setup(false)
    local create = C_CurveUtil.CreateColorCurve
    C_CurveUtil.CreateColorCurve = function()
        local c = create()
        c.GetPoints = function(self)
            local result = {}
            for i = #self.points, 1, -1 do result[#result + 1] = self.points[i] end
            return result
        end
        c.SetToDefaults = function(self) self.value = -7 end
        c.EvaluateUnpacked = function() return 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17 end
        return c
    end
    local c = capture()
    assert(c.populated.points.values[1].entries[1].fields.x.value == 48)
    assert(c.defaults.after.curveType.values[1].value == -7)
    assert(c.defaults.after.count.values[1].value == 4)
    local e = c.populated.evaluations[1].unpacked
    assert(e.n == 17 and e.truncated and #e.values == 16)
    assert(#c.copy.afterAdd.copy.points.values[1].entries == 4)
    setup(false)
    CreateColor = function() return secret end
    assert(capture().status == "restricted-color")
    setup(false)
    C_CurveUtil.CreateColorCurve = function() return secret end
    assert(capture().status == "restricted-curve")
    setup(false)
    issecretvalue = nil
    C_CurveUtil.CreateColorCurve = function() error("must not construct") end
    SlashCmdList.APICONTRACTPROBE("color-curves")
    assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)
print(passed.." color curve tests passed")
