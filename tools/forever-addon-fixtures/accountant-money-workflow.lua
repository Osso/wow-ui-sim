assert(Accountant_Classic, "Accountant Classic missing")
local current = GetMoney()
A_Admin.SetMoney(current)
A_Admin.FireEvent("PLAYER_MONEY")

local realm = assert(Accountant_ClassicSaveData[GetRealmName()], "realm data missing")
local profile = assert(realm[UnitName("player")], "character data missing")
local other = assert(profile.data.OTHER, "OTHER accounting data missing")
local modes = {"Session", "Day", "Total"}
local before = {}
for _, mode in ipairs(modes) do
    local totals = assert(other[mode], mode .. " totals missing")
    before[mode] = {In = totals.In, Out = totals.Out}
end

A_Admin.SetMoney(current + 250)
A_Admin.FireEvent("PLAYER_MONEY")
for _, mode in ipairs(modes) do
    local totals = profile.data.OTHER[mode]
    assert(totals.In == before[mode].In + 250, mode .. " income delta mismatch")
    assert(totals.Out == before[mode].Out, mode .. " expense changed during income")
end
assert(profile.options.totalcash == current + 250, "saved income balance mismatch")
print("ACCOUNTANT_WORKFLOW", "income", 250)

A_Admin.SetMoney(current)
A_Admin.FireEvent("PLAYER_MONEY")
for _, mode in ipairs(modes) do
    local totals = profile.data.OTHER[mode]
    assert(totals.In == before[mode].In + 250, mode .. " income changed during expense")
    assert(totals.Out == before[mode].Out + 250, mode .. " expense delta mismatch")
end
assert(profile.options.totalcash == current, "saved balance not restored")
print("ACCOUNTANT_WORKFLOW", "expense", 250)
print("ACCOUNTANT_WORKFLOW", "DONE")
