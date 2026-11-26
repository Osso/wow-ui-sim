local module = QuestObjectiveTracker
local blocks = module and module.usedBlocks[module.blockTemplate]
local first = blocks and blocks[990001]
local second = blocks and blocks[990002]
local firstTitle = first and first.HeaderText:GetText() or ""
return first ~= nil and first.rightEdgeFrame ~= nil and first.rightEdgeFrame:IsShown(),
       first ~= nil and first.ItemButton ~= nil and first.ItemButton:IsShown(),
       second ~= nil and second.rightEdgeFrame == nil,
       second ~= nil and second.ItemButton == nil,
       QuestCache.objects[990001] ~= nil and QuestCache.objects[990001].questID == 990001,
       firstTitle
