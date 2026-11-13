//! Named M4 policies for absent world services, never arbitrary member creation.
//!
//! Values satisfy cached return contracts, not native default evidence. Retire
//! each policy when its named backing system publishes the query through c_api.

use crate::client_profile::{ACTIVE, ClientProfile};

const SHARED: &str = r#"
C_CombatAudioAlert = C_CombatAudioAlert or {}
-- Missing combat-audio playback enablement policy; numeric settings alone do
-- not establish that the playback service is available or enabled.
if rawget(C_CombatAudioAlert, "IsEnabled") == nil then
    function C_CombatAudioAlert.IsEnabled() return false end
end
"#;

const MODERN: &str = r#"
local function install(namespace, name, fn)
    if rawget(namespace, name) == nil then namespace[name] = fn end
end
C_Minimap = C_Minimap or {}
C_PvP = C_PvP or {}
C_PlayerInfo = C_PlayerInfo or {}
C_QuestLog = C_QuestLog or {}
-- Missing hybrid-minimap world presentation capability.
install(C_Minimap, "ShouldUseHybridMinimap", function() return false end)
-- Missing covenant-calling membership metadata; ordinary quest/world-quest
-- membership is not evidence of calling membership.
install(C_QuestLog, "IsQuestCalling", function(questID) return false end)
"#;

const CLASSIC: &str = r#"
C_Map = C_Map or {}
C_PvP = C_PvP or {}
C_QuestInfoSystem = C_QuestInfoSystem or {}
-- Missing map-highlight polygons/textures. MayReturnNothing=true; no fake DTO.
if rawget(C_Map, "GetMapHighlightInfoAtPosition") == nil then
    function C_Map.GetMapHighlightInfoAtPosition(uiMapID, x, y) end
end
-- Missing world-PvP queue wait estimates; no estimated wait in simulator state.
if rawget(C_PvP, "GetWorldPvPWaitTime") == nil then
    function C_PvP.GetWorldPvPWaitTime(index) return 0 end
end
-- Missing numeric quest reward-spell catalog/context. Existing legacy reward
-- presentation has name/texture but no spell ID; do not infer IDs from names.
if rawget(C_QuestInfoSystem, "GetQuestRewardSpells") == nil then
    function C_QuestInfoSystem.GetQuestRewardSpells(questID) return {} end
end
"#;

const FOREVER: &str = r#"
-- Missing host-selected Forever experience preset. Optional result.
if rawget(C_GameRules, "GetForeverExperiencePreset") == nil then
    function C_GameRules.GetForeverExperiencePreset() return nil end
end
-- Missing quest timer lifecycle/remaining-time model. Existing objective
-- progress is not a timer snapshot.
if rawget(C_QuestLog, "GetQuestTimers") == nil then
    function C_QuestLog.GetQuestTimers() return {} end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if ACTIVE == ClientProfile::Wrath {
        return Ok(());
    }
    lua.exec(SHARED)?;
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        lua.exec(MODERN)?;
    } else {
        lua.exec(CLASSIC)?;
    }
    if ACTIVE == ClientProfile::WowForever {
        lua.exec(FOREVER)?;
    }
    Ok(())
}
