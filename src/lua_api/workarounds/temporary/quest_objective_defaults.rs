//! Temporary quest/objective tracker query defaults.
//!
//! The simulator does not model bonus objective task lists, spell-targetable
//! quest state, or auto quest popup toasts yet. Keep these startup-safe empty
//! defaults in the workaround layer instead of the central runtime bootstrap.

use crate::lua_api::LoaderEnv;

const QUEST_OBJECTIVE_DEFAULTS_LUA: &str = r#"
-- Bonus / world-quest objective trackers iterate the task list at startup.
-- Return an empty table so the `for ... in ipairs(tasksTable)` loops no-op.
if GetTasksTable == nil then
  function GetTasksTable()
    return {}
  end
end

if SpellCanTargetQuest == nil then
  function SpellCanTargetQuest()
    return false
  end
end

if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
  C_QuestHub = C_QuestHub or __wow_namespace()
  if rawget(C_QuestHub, "IsAreaPOICurrentlyRelatedToHub") == nil then
    function C_QuestHub.IsAreaPOICurrentlyRelatedToHub()
      return false
    end
  end
end

-- Retail-backed ObjectiveTracker code asks ObjectAPI's QuestCache for quest
-- objects. Classic/Mists profile source does not always load ObjectAPI's
-- retail Quest.lua, so provide a thin cache backed by the modeled C_QuestLog
-- quest surface.
if QuestCache == nil then

  local Quest = {}
  Quest.__index = Quest

  function Quest:GetID()
    return self.questID
  end

  function Quest:GetQuestLogIndex()
    return C_QuestLog.GetLogIndexForQuestID(self.questID)
  end

  function Quest:IsComplete()
    return C_QuestLog.IsComplete(self.questID)
  end

  function Quest:IsDisabledForSession()
    return C_QuestLog.IsQuestDisabledForSession(self.questID)
  end

  function Quest:IsCalling()
    return self:GetQuestClassification() == Enum.QuestClassification.Calling
  end

  function Quest:GetQuestClassification()
    return C_QuestInfoSystem.GetQuestClassification(self.questID)
  end

  QuestCache = {}

  function QuestCache:Get(questID)
    local quest = setmetatable({ questID = questID }, Quest)
    quest.title = C_QuestLog.GetTitleForQuestID(questID) or ""
    quest.requiredMoney = C_QuestLog.GetRequiredMoney(questID) or 0
    quest.isAutoComplete = false
    quest.isBounty = false
    quest.isTask = false
    return quest
  end
end

QuestUtil = QuestUtil or {}
if QuestUtil.CanCreateQuestGroup == nil then
  function QuestUtil.CanCreateQuestGroup(questID)
    if C_LFGList ~= nil and type(C_LFGList.CanCreateQuestGroup) == "function" then
      return C_LFGList.CanCreateQuestGroup(questID)
    end
    return false
  end
end
if QuestUtil.QuestShowsItemByIndex == nil then
  function QuestUtil.QuestShowsItemByIndex(questLogIndex, isQuestComplete)
    if type(GetQuestLogSpecialItemInfo) ~= "function" then
      return false
    end
    local _, item, _, showItemWhenComplete = GetQuestLogSpecialItemInfo(questLogIndex)
    return item ~= nil and (not isQuestComplete or showItemWhenComplete)
  end
end
"#;

// Existing non-Retail/Forever empty popup domain remains outside QL-2.
const UNMODELED_POPUPS_LUA: &str = r#"
if GetNumAutoQuestPopUps == nil then
  function GetNumAutoQuestPopUps() return 0 end
end
if GetAutoQuestPopUp == nil then
  function GetAutoQuestPopUp(_index) return nil, nil end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !crate::c_api::quest_objectives::ENABLED {
        lua.exec(UNMODELED_POPUPS_LUA)?;
    }
    lua.exec(QUEST_OBJECTIVE_DEFAULTS_LUA)?;
    Ok(())
}

pub(crate) fn patch_loader(env: &LoaderEnv<'_>) {
    if !crate::c_api::quest_objectives::ENABLED {
        let _ = env.exec(UNMODELED_POPUPS_LUA);
    }
    let _ = env.exec(QUEST_OBJECTIVE_DEFAULTS_LUA);
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_quest_objective_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(GetTasksTable()) ~= "table" then return "tasks_type" end
                if #GetTasksTable() ~= 0 then return "tasks_count" end
                if SpellCanTargetQuest() ~= false then return "target_quest" end
                if GetNumAutoQuestPopUps() ~= 0 then return "popup_count" end
                local popupQuestID, popupType = GetAutoQuestPopUp(1)
                if popupQuestID ~= nil or popupType ~= nil then return "popup_entry" end
                local quest = QuestCache:Get(401)
                if quest:GetID() ~= 401 then return "quest_id" end
                if type(quest.title) ~= "string" then return "quest_title" end
                if quest:IsComplete() ~= false then return "quest_complete" end
                if quest:IsDisabledForSession() ~= false then return "quest_disabled" end
                if type(quest:GetQuestClassification()) ~= "number" then return "quest_classification" end
                if QuestUtil.CanCreateQuestGroup(401) ~= false then return "quest_group" end
                if QuestUtil.QuestShowsItemByIndex(1, false) ~= false then return "quest_item" end
                return "ok"
                "#,
            )
            .expect("quest objective defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_quest_objective_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function GetTasksTable() return { 7 } end
            function SpellCanTargetQuest() return true end
            function GetNumAutoQuestPopUps() return 2 end
            function GetAutoQuestPopUp(index) return 1000 + index, "COMPLETE" end
            "#,
        )
        .expect("fixture should install existing quest members");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("quest objective defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if GetTasksTable()[1] ~= 7 then return "overwrote_tasks" end
                if SpellCanTargetQuest() ~= true then return "overwrote_target_quest" end
                if GetNumAutoQuestPopUps() ~= 2 then return "overwrote_popup_count" end
                local popupQuestID, popupType = GetAutoQuestPopUp(4)
                if popupQuestID ~= 1004 or popupType ~= "COMPLETE" then return "overwrote_popup_entry" end
                return "ok"
                "#,
            )
            .expect("quest objective preservation probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn reapply_restores_quest_util_defaults_after_addon_reset() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("QuestUtil = {}")
            .expect("fixture should reset QuestUtil like FrameXMLUtil");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("quest objective defaults should reapply");
        }

        let result: String = env
            .eval(
                r#"
                if QuestUtil.CanCreateQuestGroup(401) ~= false then return "quest_group" end
                if QuestUtil.QuestShowsItemByIndex(1, false) ~= false then return "quest_item" end
                return "ok"
                "#,
            )
            .expect("quest util defaults should be callable after reapply");

        assert_eq!(result, "ok");
    }

    #[test]
    fn quest_util_group_creation_uses_lfg_list_backing_when_available() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            QuestUtil = {}
            C_LFGList.CanCreateQuestGroup = function(questID)
              return questID == 401
            end
            "#,
        )
        .expect("fixture should install LFG backing");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("quest objective defaults should reapply");
        }

        let can_create: bool = env
            .eval("return QuestUtil.CanCreateQuestGroup(401)")
            .expect("quest group probe should run");

        assert!(can_create);
    }
}
