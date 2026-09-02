//! Temporary chat window defaults.
//!
//! Chat window persistence and initial chat color tables are not backed by a
//! full account settings model yet. Keep those startup defaults explicit here
//! until saved chat layout/state is modeled.

const CHAT_WINDOW_DEFAULTS_LUA: &str = r#"
ChatTypeInfo = ChatTypeInfo or {}
ChatTypeInfo.SYSTEM = ChatTypeInfo.SYSTEM or {
  r = 1,
  g = 1,
  b = 0,
  id = 1,
}
ChatTypeInfo.BN_WHISPER = ChatTypeInfo.BN_WHISPER or {
  r = 0,
  g = 1,
  b = 0.96470594406128,
  id = 19,
}

if GetChatTypeIndex == nil then
  function GetChatTypeIndex(_chatType)
    return 1
  end
end

ChatFrameUtil = ChatFrameUtil or {}

if rawget(_G, "ChatFrame1") == nil and type(CreateFrame) == "function" then
  CreateFrame("ScrollingMessageFrame", "ChatFrame1", UIParent)
end

if ChatFrameUtil.ProcessMessageEventFilters == nil then
  function ChatFrameUtil.ProcessMessageEventFilters(_frame, event, ...)
    return false, event, ...
  end
end

if ChatFrameUtil.GetChatWindowName == nil then
  function ChatFrameUtil.GetChatWindowName(index)
    return string.format("Chat Window %d", tonumber(index) or 1)
  end
end

if ChatFrameUtil.GetCommunitiesChannelColor == nil then
  function ChatFrameUtil.GetCommunitiesChannelColor(_clubId, streamId)
    if tonumber(streamId) == 2 then
      return 0.25, 0.75, 0.25
    end
    return 0.25, 1, 0.25
  end
end

if ChatFrameUtil.GetCommunitiesChannelLocalID == nil then
  function ChatFrameUtil.GetCommunitiesChannelLocalID(_clubId, _streamId)
    return nil
  end
end

local __wow_chat_window_state = __wow_chat_window_state or {}

local function __wow_is_chat_window_shown_by_default(id)
  return id == 1
end

local function __wow_is_chat_window_docked_by_default(id)
  return id == 1
end

if GetChatWindowInfo == nil then
  function GetChatWindowInfo(id)
    local realId = id or 1
    local chat = __wow_chat_window_state[realId]
    local shown = __wow_is_chat_window_shown_by_default(realId)
    local docked = __wow_is_chat_window_docked_by_default(realId)
    if chat and chat.shown ~= nil then
      shown = chat.shown == true
    end
    if chat and chat.docked ~= nil then
      docked = chat.docked == true
    end
    -- An unnamed window answers "" as the client's cache does;
    -- FCF_SetWindowName (FloatingChatFrame.lua:812-830) then names windows
    -- 1..3 GENERAL / COMBAT_LOG / VOICE, formats CHAT_NAME_TEMPLATE for the
    -- rest, and stores the result through SetChatWindowName.
    local name = chat and chat.name or ""
    return name, 12, 0, 0, 0, 0.25, shown, false, docked, false
  end
end

if SetChatWindowName == nil then
  function SetChatWindowName(id, name)
    local chat = __wow_chat_window_state[id] or {}
    chat.name = name
    __wow_chat_window_state[id] = chat
  end
end

if SetChatWindowDocked == nil then
  function SetChatWindowDocked(id, dockIndex)
    local chat = __wow_chat_window_state[id] or {}
    chat.docked = dockIndex ~= nil
    __wow_chat_window_state[id] = chat
  end
end

if SetChatWindowShown == nil then
  function SetChatWindowShown(id, shown)
    local chat = __wow_chat_window_state[id] or {}
    chat.shown = shown == true
    __wow_chat_window_state[id] = chat
  end
end

if GetChatWindowSavedDimensions == nil then
  function GetChatWindowSavedDimensions(id)
    local chat = __wow_chat_window_state[id]
    if not chat then
      return nil, nil
    end
    return chat.width, chat.height
  end
end

if SetChatWindowSavedDimensions == nil then
  function SetChatWindowSavedDimensions(id, width, height)
    local chat = __wow_chat_window_state[id] or {}
    chat.width = width
    chat.height = height
    __wow_chat_window_state[id] = chat
  end
end

if GetChatWindowSavedPosition == nil then
  function GetChatWindowSavedPosition(id)
    local chat = __wow_chat_window_state[id]
    if not chat then
      return nil, nil, nil
    end
    return chat.point, chat.xOffset, chat.yOffset
  end
end

if SetChatWindowSavedPosition == nil then
  function SetChatWindowSavedPosition(id, point, xOffset, yOffset)
    local chat = __wow_chat_window_state[id] or {}
    chat.point = point
    chat.xOffset = xOffset
    chat.yOffset = yOffset
    __wow_chat_window_state[id] = chat
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CHAT_WINDOW_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn chat_window_names_start_empty_and_round_trip() {
        // FCF_SetWindowName names an unnamed window itself (GENERAL for 1,
        // COMBAT_LOG for 2, VOICE for 3) and stores it with SetChatWindowName;
        // a stub name such as "Chat 3" kept that branch from ever running,
        // and without SetChatWindowName the branch raised.
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let (before, after, other): (String, String, String) = env
            .eval(
                r#"
                local before = (GetChatWindowInfo(1))
                SetChatWindowName(1, GENERAL or "General")
                return before, (GetChatWindowInfo(1)), (GetChatWindowInfo(3))
                "#,
            )
            .expect("chat window info");
        assert_eq!(before, "", "an unnamed window answers an empty name");
        assert_eq!(
            after, "General",
            "SetChatWindowName is read back by GetChatWindowInfo"
        );
        assert_eq!(other, "", "other windows are untouched");
    }

    #[test]
    fn installs_chat_window_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(ChatTypeInfo) ~= "table" then return "chat_type_table" end
                if ChatTypeInfo.SYSTEM.r ~= 1 or ChatTypeInfo.SYSTEM.g ~= 1 or ChatTypeInfo.SYSTEM.b ~= 0 then
                  return "system_color"
                end
                if ChatTypeInfo.BN_WHISPER.id ~= 19 then return "bn_whisper" end
                if GetChatTypeIndex("SAY") ~= 1 then return "chat_type_index" end
                if type(ChatFrameUtil.ProcessMessageEventFilters) ~= "function" then return "filters_function" end
                if type(ChatFrame1) ~= "table" then return "chat_frame1" end
                if ChatFrame1:GetObjectType() ~= "ScrollingMessageFrame" then return "chat_frame1_type" end
                if ChatFrameUtil.GetChatWindowName(3) ~= "Chat Window 3" then return "window_name" end

                local filtered, event, message = ChatFrameUtil.ProcessMessageEventFilters(nil, "CHAT_MSG_SAY", "hello")
                if filtered ~= false or event ~= "CHAT_MSG_SAY" or message ~= "hello" then return "filter_passthrough" end

                local altR, altG, altB = ChatFrameUtil.GetCommunitiesChannelColor(nil, 2)
                if altR ~= 0.25 or altG ~= 0.75 or altB ~= 0.25 then return "community_color" end
                if ChatFrameUtil.GetCommunitiesChannelLocalID(nil, nil) ~= nil then return "community_local_id" end

                local name, fontSize, r, g, b, alpha, shown, locked, docked = GetChatWindowInfo(2)
                if name ~= "" or fontSize ~= 12 then return "info_shape" end
                if r ~= 0 or g ~= 0 or b ~= 0 or alpha ~= 0.25 then return "info_color" end
                if shown ~= false or locked ~= false or docked ~= false then return "hidden_defaults" end

                SetChatWindowShown(2, true)
                if select(7, GetChatWindowInfo(2)) ~= true then return "shown_state" end

                SetChatWindowSavedDimensions(2, 430, 120)
                local width, height = GetChatWindowSavedDimensions(2)
                if width ~= 430 or height ~= 120 then return "dimensions" end

                SetChatWindowSavedPosition(2, "BOTTOMLEFT", 11, 22)
                local point, xOffset, yOffset = GetChatWindowSavedPosition(2)
                if point ~= "BOTTOMLEFT" or xOffset ~= 11 or yOffset ~= 22 then return "position" end

                return "ok"
                "#,
            )
            .expect("chat window defaults probe should run");

        assert_eq!(result, "ok");
    }
}
