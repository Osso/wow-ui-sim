for bag = 0, 4 do
    assert(C_Container.GetContainerNumSlots(bag) == 16, "unexpected capacity for bag " .. bag)
end
local bankSlots = GetNumBankSlots()
for bag = 5, math.max(5, 4 + bankSlots) do
    assert(C_Container.GetContainerNumSlots(bag) == 0, "unexpected capacity for bag " .. bag)
end
assert(Bagmeter_Used == 0, "BagMeter must display free slots")
assert(Bagmeter_Total == 1 and Bagmeter_Subtotal == 1 and Bagmeter_Subcount == 1,
    "BagMeter free/total display settings differ")
assert(Bagmeter_CountCraft == 1 and Bagmeter_CountAmmo == 1,
    "BagMeter must include all bag types")

local bag0 = assert(BM_Bag0Txt, "bag 0 text missing")
local total = assert(BM_BagXTxt, "aggregate text missing")
local function assertCounts(step, expectedBag, expectedTotal)
    assert(bag0:IsShown(), "bag 0 text is not shown")
    assert(total:IsShown(), "aggregate text is not shown")
    assert(bag0:GetText() == expectedBag, "unexpected bag 0 text: " .. tostring(bag0:GetText()))
    assert(total:GetText() == expectedTotal, "unexpected aggregate text: " .. tostring(total:GetText()))
    print("BAGMETER_WORKFLOW", step, bag0:GetText(), total:GetText())
end

A_Admin.ClearBags()
A_Admin.AddBagItem(0, 1, 6948, 1)
A_Admin.FireEvent("BAG_UPDATE", 0)
assertCounts("one-item", "15/16", "79/80")

A_Admin.AddBagItem(0, 2, 6948, 1)
A_Admin.FireEvent("BAG_UPDATE", 0)
assertCounts("two-items", "14/16", "78/80")

A_Admin.RemoveBagItem(0, 1)
A_Admin.FireEvent("BAG_UPDATE", 0)
assertCounts("removed-first", "15/16", "79/80")
print("BAGMETER_WORKFLOW", "DONE")
