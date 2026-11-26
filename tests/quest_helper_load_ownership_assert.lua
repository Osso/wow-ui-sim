local module = QuestObjectiveTracker
local blocks = module and module.usedBlocks[module.blockTemplate]
local first = blocks and blocks[990001]
local second = blocks and blocks[990002]
local firstTitle = first and first.HeaderText:GetText() or ""
local container = module and module.parentContainer
local diagnostic = table.concat({
    "events=" .. tostring(QuestHelperProbe.events),
    "dirty=" .. tostring(QuestHelperProbe.dirty),
    "containerUpdates=" .. tostring(QuestHelperProbe.containerUpdates),
    "moduleUpdates=" .. tostring(QuestHelperProbe.moduleUpdates),
    "watchBuilds=" .. tostring(QuestHelperProbe.watchBuilds),
    "nextFrame=" .. tostring(QuestHelperProbe.nextFrame),
    "frameShown=" .. tostring(ObjectiveTrackerFrame:IsShown()),
    "frameVisible=" .. tostring(ObjectiveTrackerFrame:IsVisible()),
    "frameHeight=" .. tostring(ObjectiveTrackerFrame:GetHeight()),
    "containerVisible=" .. tostring(container:IsVisible()),
    "containerHeight=" .. tostring(container:GetHeight()),
    "moduleHeight=" .. tostring(module:GetHeight()),
    "moduleDirty=" .. tostring(module.isDirty),
    "containerDirty=" .. tostring(container.dirty),
    "containerDirtyCallback=" .. type(container.dirtyCallback),
    "moduleDirtyCallback=" .. type(module.dirtyCallback),
    "moduleState=" .. tostring(module.state),
    "watches=" .. tostring(C_QuestLog.GetNumQuestWatches()),
    "watch1=" .. tostring(C_QuestLog.GetQuestIDForQuestWatchIndex(1)),
    "moduleExists=" .. tostring(module ~= nil),
    "hasContainer=" .. tostring(module ~= nil and module.parentContainer ~= nil),
    "initialized=" .. tostring(module ~= nil and module.init == true),
    "frameExists=" .. tostring(ObjectiveTrackerFrame ~= nil),
    "watched=" .. tostring(C_QuestLog.GetQuestWatchType(990001) ~= nil),
    "parentMatches=" .. tostring(module ~= nil and module.parentContainer == ObjectiveTrackerFrame),
}, ",")
return first ~= nil and first.rightEdgeFrame ~= nil and first.rightEdgeFrame:IsShown(),
       first ~= nil and first.ItemButton ~= nil and first.ItemButton:IsShown(),
       second ~= nil and second.rightEdgeFrame == nil,
       second ~= nil and second.ItemButton == nil,
       QuestCache.objects[990001] ~= nil and QuestCache.objects[990001].questID == 990001,
       firstTitle,
       diagnostic
