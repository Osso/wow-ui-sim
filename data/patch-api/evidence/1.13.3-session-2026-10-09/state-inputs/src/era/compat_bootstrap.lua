-- Era / Anniversary compat bootstrap.
--
-- Loaded after `runtime_surface_bootstrap.lua` under `client-era` and
-- `client-anniversary` (both serve vanilla content; the only meaningful
-- difference is the source-repo build SHA). Every entry uses
-- `if rawget(_G, "X") == nil then ... end` so a real definition from a
-- vanilla `Blizzard_*` addon (loaded later) takes precedence.
--
-- Seed list comes from the 29 globals that surfaced in the era + anniversary
-- lua-errors baselines but were absent from the runtime surface. See
-- `docs/baselines/README.md` for the cluster analysis.

-- ─── Generic vanilla-era globals (no return state, no side effects) ────────

if rawget(_G, "FillLocalizedClassList") == nil then
  function FillLocalizedClassList(t) return t end
end

if rawget(_G, "AddLuaErrorHandler") == nil then
  function AddLuaErrorHandler(_handler) end
end

if rawget(_G, "IsInGlobalEnvironment") == nil then
  function IsInGlobalEnvironment() return true end
end

if rawget(_G, "AreHighResTexturesAvailable") == nil then
  function AreHighResTexturesAvailable() return true end
end

if rawget(_G, "IsCommunitiesUIDisabledByTrialAccount") == nil then
  function IsCommunitiesUIDisabledByTrialAccount() return false end
end

-- ─── Action-bar helpers ────────────────────────────────────────────────────

if rawget(_G, "GetActionBarPage") == nil then
  function GetActionBarPage() return 1 end
end

if rawget(_G, "GetActionBarToggles") == nil then
  function GetActionBarToggles()
    return false, false, false, false, false, false
  end
end

if rawget(_G, "GetDisplayedAllyFrames") == nil then
  function GetDisplayedAllyFrames() return nil end
end

-- ─── Combat / class data ───────────────────────────────────────────────────

if rawget(_G, "GetComboPoints") == nil then
  function GetComboPoints() return 0 end
end

if rawget(_G, "SpellGetVisibilityInfo") == nil then
  function SpellGetVisibilityInfo()
    return false, false, false
  end
end

-- ─── PvP / honor stats ─────────────────────────────────────────────────────

if rawget(_G, "GetPVPYesterdayStats") == nil then
  function GetPVPYesterdayStats()
    return 0, 0, 0
  end
end

if rawget(_G, "GetTabardCreationCost") == nil then
  function GetTabardCreationCost() return 0 end
end

-- ─── Quest tracking ────────────────────────────────────────────────────────

if rawget(_G, "GetNumQuestWatches") == nil then
  function GetNumQuestWatches() return 0 end
end

if rawget(_G, "GetQuestTimers") == nil then
  function GetQuestTimers() return nil end
end

-- ─── Raid frame profiles ───────────────────────────────────────────────────

if rawget(_G, "GetRaidProfileOption") == nil then
  function GetRaidProfileOption(_profile, _option) return nil end
end

if rawget(_G, "HasLoadedCUFProfiles") == nil then
  function HasLoadedCUFProfiles() return false end
end

-- ─── Guild ─────────────────────────────────────────────────────────────────

if rawget(_G, "GuildControlGetRank") == nil then
  function GuildControlGetRank(_index) return nil end
end

-- ─── Keyring / pet UI (vanilla-only features) ──────────────────────────────

if rawget(_G, "HasKey") == nil then
  function HasKey() return false end
end

if rawget(_G, "IsKeyRingEnabled") == nil then
  function IsKeyRingEnabled() return false end
end

if rawget(_G, "HasPetUI") == nil then
  function HasPetUI() return false, false end
end

-- ─── Skills (vanilla-only trade-skills frame) ──────────────────────────────

if rawget(_G, "SetSelectedSkill") == nil then
  function SetSelectedSkill(_idx) end
end

-- ─── Money frame & money input helpers ─────────────────────────────────────

if rawget(_G, "MoneyFrame_OnLoad") == nil then
  function MoneyFrame_OnLoad(_frame) end
end

if rawget(_G, "SmallMoneyFrame_OnLoad") == nil then
  function SmallMoneyFrame_OnLoad(_frame) end
end

if rawget(_G, "MoneyInputFrame_SetCompact") == nil then
  function MoneyInputFrame_SetCompact(_frame) end
end

if rawget(_G, "MoneyInputFrame_SetOnValueChangedFunc") == nil then
  function MoneyInputFrame_SetOnValueChangedFunc(_frame, _fn) end
end

if rawget(_G, "MoneyInputFrame_SetPreviousFocus") == nil then
  function MoneyInputFrame_SetPreviousFocus(_frame, _prev) end
end

-- ─── UIParent OnLoad ───────────────────────────────────────────────────────
--
-- Vanilla's UIParent.lua wires `<OnLoad function="UIParent_OnLoad"/>` directly
-- (retail moved this onto the frame mixin). The simulator pre-creates UIParent
-- in Rust before the Vanilla addon runs, so the addon's UIParent_OnLoad fires
-- against an already-constructed frame; a no-op is safe.

if rawget(_G, "UIParent_OnLoad") == nil then
  function UIParent_OnLoad(_self) end
end

-- ─── Forbidden frames / SecureMixin ────────────────────────────────────────

if rawget(_G, "CreateForbiddenFrame") == nil then
  function CreateForbiddenFrame(frameType, name, parent, template)
    local frame = CreateFrame(frameType or "Frame", name, parent, template)
    if frame and type(frame.SetForbidden) == "function" then
      frame:SetForbidden(true)
    end
    return frame
  end
end

if rawget(_G, "SecureMixin") == nil then
  function SecureMixin(target, ...)
    for i = 1, select("#", ...) do
      local mixin = select(i, ...)
      if type(mixin) == "table" then
        for k, v in pairs(mixin) do
          target[k] = v
        end
      end
    end
    return target
  end
end
