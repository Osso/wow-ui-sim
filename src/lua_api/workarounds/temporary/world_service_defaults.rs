//! Named M4 policies for absent world services, never arbitrary member creation.
//!
//! Values satisfy cached return contracts, not native default evidence. Retire
//! each policy when its named backing system publishes the query through c_api.

use crate::client_profile::{ACTIVE, ClientProfile};

const SHARED: &str = r#"
C_CombatAudioAlert = C_CombatAudioAlert or {}
"#;

const MODERN: &str = r#"
local function install(namespace, name, fn)
    if rawget(namespace, name) == nil then namespace[name] = fn end
end
C_Minimap = C_Minimap or {}
C_PvP = C_PvP or {}
C_PlayerInfo = C_PlayerInfo or {}
C_QuestLog = C_QuestLog or {}
-- INFERRED-UNKNOWN: this fallback is constant false; context does not select a verified result.
install(C_Minimap, "ShouldUseHybridMinimap", function() return false end)
-- VERIFIED: absent from the incomplete recovered Retail C_QuestLog census; export status remains unknown.
-- Ordinary quest/world-quest membership is not evidence of calling membership.
install(C_QuestLog, "IsQuestCalling", function(questID) return false end)
"#;

const CLASSIC: &str = r#"
C_Map = C_Map or {}
C_PvP = C_PvP or {}
C_QuestInfoSystem = C_QuestInfoSystem or {}
-- VERIFIED: the Retail C_Map member census lists this API; Mists behavior remains unknown.
-- No map-highlight geometry is supplied, so the compatibility result has no fake DTO.
if rawget(C_Map, "GetMapHighlightInfoAtPosition") == nil then
    function C_Map.GetMapHighlightInfoAtPosition(uiMapID, x, y) end
end
-- VERIFIED: this exact name is absent from the incomplete Retail C_PvP census.
-- Mists behavior is unknown; no estimated wait is supplied.
if rawget(C_PvP, "GetWorldPvPWaitTime") == nil then
    function C_PvP.GetWorldPvPWaitTime(index) return 0 end
end
-- VERIFIED: the Retail C_QuestInfoSystem member census lists this API; Mists behavior is unknown.
-- Existing legacy reward presentation has no spell ID, so do not infer IDs from names.
if rawget(C_QuestInfoSystem, "GetQuestRewardSpells") == nil then
    function C_QuestInfoSystem.GetQuestRewardSpells(questID) return {} end
end
"#;

const FOREVER: &str = r#"
-- VERIFIED: Forever registers this API; no exact entry appears in the recovered Retail C_GameRules census.
-- Optional result remains the existing compatibility value.
if rawget(C_GameRules, "GetForeverExperiencePreset") == nil then
    function C_GameRules.GetForeverExperiencePreset() return nil end
end
-- VERIFIED: Forever registers this API; the incomplete Retail C_QuestLog census has no exact entry.
-- UNKNOWN: the Retail census does not establish export status or timer behavior.
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
