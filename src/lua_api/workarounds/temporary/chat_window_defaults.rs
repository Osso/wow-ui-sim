//! Chat type defaults and non-Retail chat-window fallback behavior.
//!
//! Retail and WoW Forever chat-window state is backed by `SimState.chat_windows`.
//! Other profiles keep the temporary Lua compatibility state.

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const CHAT_TYPE_INDEX_DEFAULT_LUA: &str = r#"
if GetChatTypeIndex == nil then
  function GetChatTypeIndex(_chatType) return 1 end
end
"#;

const CHAT_WINDOW_DEFAULTS_LUA: &str = r#"
ChatTypeInfo = ChatTypeInfo or {}
ChatTypeInfo.SYSTEM = ChatTypeInfo.SYSTEM or { r = 1, g = 1, b = 0, id = 1 }
ChatTypeInfo.BN_WHISPER = ChatTypeInfo.BN_WHISPER or {
  r = 0, g = 1, b = 0.96470594406128, id = 19,
}

ChatFrameUtil = ChatFrameUtil or {}
if ChatFrameUtil.GetChatWindowName == nil then
  function ChatFrameUtil.GetChatWindowName(index)
    return string.format("Chat Window %d", tonumber(index) or 1)
  end
end
"#;

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const CHAT_WINDOW_STATE_LUA: &str = r#"
local __wow_chat_window_state = __wow_chat_window_state or {}
local function shown_by_default(id) return id == 1 end
local function docked_by_default(id) return id == 1 end

if GetChatWindowInfo == nil then
  function GetChatWindowInfo(id)
    local realId = id or 1
    local chat = __wow_chat_window_state[realId]
    local shown = chat and chat.shown
    local docked = chat and chat.docked
    if shown == nil then shown = shown_by_default(realId) end
    if docked == nil then docked = docked_by_default(realId) end
    return chat and chat.name or "", 12, 0, 0, 0, 0.25, shown, false, docked, false
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
    if not chat then return nil, nil end
    return chat.width, chat.height
  end
end

if SetChatWindowSavedDimensions == nil then
  function SetChatWindowSavedDimensions(id, width, height)
    local chat = __wow_chat_window_state[id] or {}
    chat.width, chat.height = width, height
    __wow_chat_window_state[id] = chat
  end
end

if GetChatWindowSavedPosition == nil then
  function GetChatWindowSavedPosition(id)
    local chat = __wow_chat_window_state[id]
    if not chat then return nil, nil, nil end
    return chat.point, chat.xOffset, chat.yOffset
  end
end

if SetChatWindowSavedPosition == nil then
  function SetChatWindowSavedPosition(id, point, xOffset, yOffset)
    local chat = __wow_chat_window_state[id] or {}
    chat.point, chat.xOffset, chat.yOffset = point, xOffset, yOffset
    __wow_chat_window_state[id] = chat
  end
end
"#;

const CHAT_FRAME_PLACEHOLDER_LUA: &str = r#"
if rawget(_G, "ChatFrame1") == nil and type(CreateFrame) == "function" then
  CreateFrame("ScrollingMessageFrame", "ChatFrame1", UIParent)
end
"#;

pub(crate) fn apply_bootstrap(
    lua: &mut rilua::Lua,
    skip_chat_frame_placeholder: bool,
) -> crate::Result<()> {
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(CHAT_TYPE_INDEX_DEFAULT_LUA)?;
    lua.exec(CHAT_WINDOW_DEFAULTS_LUA)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(CHAT_WINDOW_STATE_LUA)?;
    if !skip_chat_frame_placeholder {
        lua.exec(CHAT_FRAME_PLACEHOLDER_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_chat_type_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let index_check = if cfg!(any(
            feature = "client-retail",
            feature = "client-wowforever"
        )) {
            r#"if GetChatTypeIndex("SYSTEM") ~= 1 then return "index" end"#
        } else {
            r#"if GetChatTypeIndex("SAY") ~= 1 then return "index" end"#
        };
        let result: String = env
            .eval(&format!(
                r#"
                if ChatTypeInfo.SYSTEM.id ~= 1 then return "system" end
                if ChatTypeInfo.BN_WHISPER.id ~= 19 then return "whisper" end
                {index_check}
                return "ok"
                "#,
            ))
            .expect("chat defaults probe should run");
        assert_eq!(result, "ok");
    }
}
