local function __wow_ensure_enum_table(name)
  if type(Enum[name]) ~= "table" then
    rawset(Enum, name, {})
  end
  return Enum[name]
end

local function __wow_fill_enum(enumName, values)
  local enum = __wow_ensure_enum_table(enumName)
  local maxValue = -1
  for _, value in pairs(enum) do
    if type(value) == "number" and value > maxValue then
      maxValue = value
    end
  end
  for _, key in ipairs(values) do
    if rawget(enum, key) == nil then
      maxValue = maxValue + 1
      rawset(enum, key, maxValue)
    end
  end
end

__wow_fill_enum("ClubStreamType", { "Discord" })
__wow_fill_enum("CompanionConfigSlotTypes", { "Flavor" })
__wow_fill_enum("CooldownViewerCategory", { "GroupBuff", "SpecAgnosticEssential", "SpecAgnosticTracked", "EquipSlotEssential", "EquipSlotTracked" })
__wow_fill_enum("EditModeAccountSetting", { "ShowRaidWarning" })
__wow_fill_enum("EditModeMinimapSetting", { "IconScale" })
__wow_fill_enum("EditModeSystem", { "RaidWarning" })
__wow_fill_enum("EditModeUnitFrameSetting", { "BuffIconSize", "DebuffIconSize" })
__wow_fill_enum("FragmentID", { "FMapObject", "FWorldStateListenerData" })
__wow_fill_enum("FrameTutorialAccount", { "HousingPetBeds" })
__wow_fill_enum("HouseFinderSuggestionReason", { "Relinquished" })
__wow_fill_enum("NamePlateStyle", { "Classic" })

-- Blizzard_HousingData.lua:124-127 keys a table by these four, but no
-- Blizzard_APIDocumentationGenerated file defines them (HouseSettingFlags has
-- no enumeration entry at all). Values unknown: the next free bits above
-- both HouseSettingFlags definitions, in the order the file uses them.
if Enum.HouseSettingFlags and Enum.HouseSettingFlags.BlueprintExportNeighbors == nil then
  Enum.HouseSettingFlags.BlueprintExportNeighbors = 1024
  Enum.HouseSettingFlags.BlueprintExportGuild = 2048
  Enum.HouseSettingFlags.BlueprintExportFriends = 4096
  Enum.HouseSettingFlags.BlueprintExportParty = 8192
end
__wow_fill_enum("PingResult", { "FailedSilent" })
__wow_fill_enum("PingSubjectType", { "ActionReady", "ActionOnCooldown", "ActionUnavailable" })
__wow_fill_enum("SecretAspect", { "RadialProgress" })
__wow_fill_enum("TooltipDataLineType", { "ItemSpellTriggerOnUse", "ItemSpellTriggerOnEquip", "ItemSpellTriggerOnProc" })

if not Enum.ForbiddenScriptObjectAspect then
  rawset(Enum, "ForbiddenScriptObjectAspect", {
    UntrustedScriptExecution = 1,
    UntrustedLayoutScriptExecution = 2,
    EventRegistrations = 4,
    AlwaysPropagateInput = 8,
    ScriptedInput = 16,
    QueryFocus = 32,
  })
end

if not Enum.ForbiddenAspectInheritance then
  rawset(Enum, "ForbiddenAspectInheritance", {
    Parent = 1,
    Layout = 2,
  })
end

if not Enum.OnUpdateMode then
  rawset(Enum, "OnUpdateMode", {
    Disabled = "Disabled",
    RunWhenVisible = "RunWhenVisible",
    RunWhenVisibleOnce = "RunWhenVisibleOnce",
    RunOnce = "RunOnce",
    RunAlways = "RunAlways",
  })
end

if not Enum.ScriptObjectOnUpdateMode then
  rawset(Enum, "ScriptObjectOnUpdateMode", Enum.OnUpdateMode)
end

if type(DifficultyUtil) ~= "table" then
  DifficultyUtil = {}
end

local function __wow_dynamic_global_delegate(globalName)
  return function(...)
    return _G[globalName](...)
  end
end

local difficultyColorDelegates = {
  "GetCreatureDifficultyColor",
  "GetDifficultyColor",
  "GetQuestDifficultyColor",
  "GetRelativeDifficultyColor",
  "GetScalingQuestDifficultyColor",
}
for _, functionName in ipairs(difficultyColorDelegates) do
  if rawget(DifficultyUtil, functionName) == nil then
    rawset(DifficultyUtil, functionName, __wow_dynamic_global_delegate(functionName))
  end
end

if type(FlagsUtil) ~= "table" then
  FlagsUtil = {}
end

if type(FlagsUtilConstants) ~= "table" then
  FlagsUtilConstants = {}
end
if rawget(FlagsUtilConstants, "CombineShouldSet") == nil then
  FlagsUtilConstants.CombineShouldSet = true
end

if FlagsUtil.IsSet == nil then
  function FlagsUtil.IsSet(flags, flag)
    if type(flags) ~= "number" or type(flag) ~= "number" then
      return false
    end
    return bit.band(flags, flag) ~= 0
  end
end

if GetSpecializationSystem == nil then
  function GetSpecializationSystem()
    if Enum.SpecializationSystem and Enum.SpecializationSystem.Traits ~= nil then
      return Enum.SpecializationSystem.Traits
    end
    return "Traits"
  end
end

if securecopy == nil then
  local function __wow_securecopy(value, seen)
    if type(value) ~= "table" then
      return value
    end
    if seen[value] ~= nil then
      return seen[value]
    end
    local copy = {}
    seen[value] = copy
    for k, v in pairs(value) do
      copy[__wow_securecopy(k, seen)] = __wow_securecopy(v, seen)
    end
    return copy
  end

  function securecopy(value)
    return __wow_securecopy(value, {})
  end
end

if settablesecurity == nil then
  function settablesecurity(_table, _key, _taint, _secure)
  end
end

if CooldownManagerLayout_GetGroupBuffVisualAlerts == nil then
  function CooldownManagerLayout_GetGroupBuffVisualAlerts()
    return C_UnitAuras.GetGroupBuffVisualAlerts()
  end
end
if CooldownManagerLayout_SetGroupBuffVisualAlerts == nil then
  function CooldownManagerLayout_SetGroupBuffVisualAlerts(alerts)
    C_UnitAuras.SetGroupBuffVisualAlerts(alerts)
  end
end
if CooldownManagerLayout_GetHiddenGroupBuffs == nil then
  function CooldownManagerLayout_GetHiddenGroupBuffs()
    return C_UnitAuras.GetHiddenGroupBuffs()
  end
end
if CooldownManagerLayout_SetHiddenGroupBuffs == nil then
  function CooldownManagerLayout_SetHiddenGroupBuffs(hiddenBuffs)
    C_UnitAuras.SetHiddenGroupBuffs(hiddenBuffs)
  end
end

if LoadAddOnWithErrorHandling == nil then
  function LoadAddOnWithErrorHandling(name)
    return UIParentLoadAddOn(name)
  end
end

C_CVar = C_CVar or {}
if rawget(C_CVar, "AreCVarsLoaded") == nil then
  function C_CVar.AreCVarsLoaded()
    return true
  end
end

C_CombatAudioAlert = C_CombatAudioAlert or {}
if rawget(C_CombatAudioAlert, "SpeakText") == nil then
  function C_CombatAudioAlert.SpeakText()
    return 0
  end
end

C_CooldownViewer = C_CooldownViewer or {}
if rawget(C_CooldownViewer, "GetGroupBuffItems") == nil then
  function C_CooldownViewer.GetGroupBuffItems()
    return {}
  end
end

C_Sound = C_Sound or {}
if rawget(C_Sound, "PlaySoundWithOptions") == nil then
  function C_Sound.PlaySoundWithOptions()
  end
end

C_Navigation = C_Navigation or {}
if rawget(C_Navigation, "GetNextWaypointForMap") == nil then
  function C_Navigation.GetNextWaypointForMap()
    return nil
  end
end

C_SocialQueue = C_SocialQueue or {}
if rawget(C_SocialQueue, "IsSystemEnabled") == nil then
  function C_SocialQueue.IsSystemEnabled()
    return false
  end
end
if rawget(C_SocialQueue, "IsSystemSupported") == nil then
  function C_SocialQueue.IsSystemSupported()
    return false
  end
end

C_SocialRestrictions = C_SocialRestrictions or {}
if rawget(C_SocialRestrictions, "IsFriendsDisabled") == nil then
  function C_SocialRestrictions.IsFriendsDisabled()
    return false
  end
end

C_SocialUI = C_SocialUI or {}
if rawget(C_SocialUI, "IsSystemEnabled") == nil then
  function C_SocialUI.IsSystemEnabled()
    return false
  end
end

C_FriendList = C_FriendList or {}
if rawget(C_FriendList, "IsLegacyFriendSystemEnabled") == nil then
  function C_FriendList.IsLegacyFriendSystemEnabled()
    return false
  end
end

C_UnitAuras = C_UnitAuras or {}
if type(C_UnitAuras._groupBuffVisualAlerts) ~= "table" then
  C_UnitAuras._groupBuffVisualAlerts = {}
end
if rawget(C_UnitAuras, "GetGroupBuffVisualAlerts") == nil then
  function C_UnitAuras.GetGroupBuffVisualAlerts()
    return C_UnitAuras._groupBuffVisualAlerts
  end
end
if rawget(C_UnitAuras, "SetGroupBuffVisualAlerts") == nil then
  function C_UnitAuras.SetGroupBuffVisualAlerts(alerts)
    C_UnitAuras._groupBuffVisualAlerts = type(alerts) == "table" and alerts or {}
  end
end
if type(C_UnitAuras._hiddenGroupBuffs) ~= "table" then
  C_UnitAuras._hiddenGroupBuffs = {}
end
if rawget(C_UnitAuras, "GetHiddenGroupBuffs") == nil then
  function C_UnitAuras.GetHiddenGroupBuffs()
    return C_UnitAuras._hiddenGroupBuffs
  end
end
if rawget(C_UnitAuras, "SetHiddenGroupBuffs") == nil then
  function C_UnitAuras.SetHiddenGroupBuffs(hiddenBuffs)
    C_UnitAuras._hiddenGroupBuffs = type(hiddenBuffs) == "table" and hiddenBuffs or {}
  end
end
