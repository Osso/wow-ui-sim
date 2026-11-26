local blocks = QuestObjectiveTracker.usedBlocks[QuestObjectiveTracker.blockTemplate]
local first = blocks and blocks[80000]
local second = blocks and blocks[80001]
local firstTitle = first and first.HeaderText:GetText() or ""
return first ~= nil and first.rightEdgeFrame ~= nil and first.rightEdgeFrame:IsShown(),
       first ~= nil and first.ItemButton ~= nil and first.ItemButton:IsShown(),
       second ~= nil and second.rightEdgeFrame == nil,
       second ~= nil and second.ItemButton == nil,
       QuestCache.objects[80000] ~= nil and QuestCache.objects[80000].questID == 80000,
       firstTitle
