//! Temporary glue character-select defaults.
//!
//! Character-select startup still expects a handful of glue globals before the
//! simulator has a full account/character-service state model. Keep those
//! defaults explicit here until that backing model owns them.

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const SPECIALIZATION_INFO_DEFAULT_LUA: &str = r#"
if GetSpecializationInfoForSpecID == nil then
  function GetSpecializationInfoForSpecID(_specID)
    return nil, ""
  end
end
"#;

const GLUE_CHARACTER_SELECT_DEFAULTS_LUA: &str = r#"
if GetCharacterUndeleteStatus == nil then
  function GetCharacterUndeleteStatus()
    return false, false, 0, 0
  end
end

if IsCharacterTimerunning == nil then
  function IsCharacterTimerunning(_characterIndex)
    return false
  end
end

if ShouldShowExpansionUpgradeBanner == nil then
  function ShouldShowExpansionUpgradeBanner()
    return false
  end
end

if GetCharacterListGroupsInfo == nil then
  function GetCharacterListGroupsInfo()
    return {}
  end
end

if GetCharacterListUpdate == nil then
  local function __wow_character_select_event(frame, event, ...)
    if type(frame) == "table" and type(frame.OnEvent) == "function" then
      frame:OnEvent(event, ...)
    end
  end

  function GetCharacterListUpdate()
    local includeEmptySlots = true
    local listSize = type(GetNumCharacters) == "function" and GetNumCharacters(includeEmptySlots) or 0
    if type(CharacterSelect) == "table" then
      CharacterSelect.waitingforCharacterList = false
    end
    __wow_character_select_event(CharacterSelect, "CHARACTER_LIST_UPDATE", listSize)
    __wow_character_select_event(CharacterSelectCharacterFrame, "CHARACTER_LIST_UPDATE", listSize)

    local selectedCharacter = 0
    if type(GetCharacterSelection) == "function" then
      selectedCharacter = GetCharacterSelection() or 0
    elseif listSize > 0 then
      selectedCharacter = 1
    end
    __wow_character_select_event(CharacterSelect, "UPDATE_SELECTED_CHARACTER", selectedCharacter)
    if type(CharacterSelectCharacterFrame) == "table"
      and type(CharacterSelectCharacterFrame.UpdateCharacterSelection) == "function"
    then
      CharacterSelectCharacterFrame:UpdateCharacterSelection()
    end
  end
end
"#;

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const NATIVE_GLUE_CHARACTER_SELECT_DEFAULTS_LUA: &str = r#"
local __wow_char_select_model_frame_name = nil
local __wow_char_select_map_scene_frame_name = nil
local __wow_character_screen_initialized = false

if SetWorldFrameStrata == nil then
  function SetWorldFrameStrata(frame)
    if type(frame) == "table" and type(frame.SetFrameStrata) == "function" then
      frame:SetFrameStrata("BACKGROUND")
    end
  end
end
if SetCharSelectModelFrame == nil then
  function SetCharSelectModelFrame(frameName)
    __wow_char_select_model_frame_name = frameName
  end
end
if SetCharSelectMapSceneFrame == nil then
  function SetCharSelectMapSceneFrame(frameName)
    __wow_char_select_map_scene_frame_name = frameName
  end
end
if InitializeCharacterScreenData == nil then
  function InitializeCharacterScreenData()
    __wow_character_screen_initialized = true
  end
end
if GetMaxWarbandGroupCount == nil then
  function GetMaxWarbandGroupCount() return 4 end
end
if GetActiveTimerunningSeasonID == nil then
  function GetActiveTimerunningSeasonID() return nil end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(SPECIALIZATION_INFO_DEFAULT_LUA)?;
    lua.exec(GLUE_CHARACTER_SELECT_DEFAULTS_LUA)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(NATIVE_GLUE_CHARACTER_SELECT_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_glue_character_select_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local specName, specDescription = GetSpecializationInfoForSpecID(999999)
                if specName ~= nil or specDescription ~= "" then return "spec" end

                local canUndelete, cooldownActive, cooldownRemaining, cooldownSeconds = GetCharacterUndeleteStatus()
                if canUndelete ~= false or cooldownActive ~= false then return "undelete_flags" end
                if cooldownRemaining ~= 0 or cooldownSeconds ~= 0 then return "undelete_cooldown" end

                if IsCharacterTimerunning(1) ~= false then return "timerunning" end
                if ShouldShowExpansionUpgradeBanner() ~= false then return "upgrade_banner" end

                local groups = GetCharacterListGroupsInfo()
                if type(groups) ~= "table" or next(groups) ~= nil then return "groups" end
                if type(GetCharacterListUpdate) ~= "function" then return "character_list_update" end

                return "ok"
                "#,
            )
            .expect("glue character-select defaults probe should run");

        assert_eq!(result, "ok");
    }
}
