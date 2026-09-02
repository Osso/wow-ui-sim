//! Temporary chat voice button surface workaround.
//!
//! The chat frame voice/channel button expects a few runtime-created
//! globals and back-links after Blizzard chat setup has run. Seed those
//! relationships until the simulator models the full chat frame creation
//! path.

use crate::lua_api::{LoaderEnv, WowLuaEnv};

const CHAT_VOICE_BUTTON_SURFACE_WORKAROUND_LUA: &str = r#"
function __wow_apply_chat_voice_button_surface()
    local defaultChatFrame = DEFAULT_CHAT_FRAME or ChatFrame1
    local defaultEditBox = rawget(_G, "ChatFrame1EditBox")
    if type(defaultChatFrame) == "table" and type(defaultEditBox) == "table" then
        if defaultChatFrame.editBox == nil then
            defaultChatFrame.editBox = defaultEditBox
        end
        if defaultEditBox.chatFrame == nil then
            defaultEditBox.chatFrame = defaultChatFrame
        end
        if DEFAULT_CHAT_FRAME == nil then
            DEFAULT_CHAT_FRAME = defaultChatFrame
        end
    end

    local channelButton = ChatFrameChannelButton
    if type(channelButton) == "table" then
        -- Blizzard's PropertyButtonMixin:OnLoad creates .Icon and
        -- ChannelFrameButtonMixin picks its atlas from the voice state
        -- (voicechat without an active channel, headset with one). Only seed
        -- an Icon when none exists; replacing Blizzard's left the orphaned
        -- headset icon showing under the seeded speaker.
        local icon = channelButton.Icon
        if (icon == nil or type(icon) ~= "table")
            and type(channelButton.CreateTexture) == "function" then
            icon = channelButton:CreateTexture(nil, "OVERLAY")
            channelButton.Icon = icon
        end

        if icon ~= nil then
            if type(icon.SetParentKey) == "function" then
                pcall(icon.SetParentKey, icon, "Icon", true)
            end
            if type(icon.GetWidth) == "function" and type(icon.GetHeight) == "function"
                and (icon:GetWidth() == 0 or icon:GetHeight() == 0)
                and type(icon.SetSize) == "function" then
                icon:SetSize(channelButton.fixedIconWidth or 15, channelButton.fixedIconHeight or 15)
            end
            if type(icon.GetNumPoints) == "function" and icon:GetNumPoints() == 0
                and type(icon.SetPoint) == "function" then
                icon:SetPoint("CENTER", channelButton, "CENTER", 0, 0)
            end
            if type(channelButton.UpdateVisibleState) == "function" then
                pcall(channelButton.UpdateVisibleState, channelButton)
            end
            local hasAtlas = type(icon.GetAtlas) == "function" and icon:GetAtlas() ~= nil
                or rawget(icon, "atlas") ~= nil
            if not hasAtlas then
                if type(icon.SetAtlas) == "function" then
                    icon:SetAtlas("chatframe-button-icon-voicechat")
                else
                    rawset(icon, "atlas", "chatframe-button-icon-voicechat")
                end
            end
            if type(icon.Show) == "function" then
                icon:Show()
            end
        end
    end

    if QuickJoinToastButton == nil and type(CreateFrame) == "function" and UIParent ~= nil then
        QuickJoinToastButton = CreateFrame("Button", "QuickJoinToastButton", UIParent)
    end
end

_G.__wow_apply_chat_voice_button_surface()

if C_AddOns and type(C_AddOns.LoadAddOn) == "function" and not rawget(_G, "__wow_chat_voice_button_surface_load_hooked") then
    rawset(_G, "__wow_chat_voice_button_surface_load_hooked", true)
    hooksecurefunc(C_AddOns, "LoadAddOn", function(addonName)
        if addonName == "Blizzard_ChatFrame"
            or addonName == "Blizzard_QuickJoin"
            or addonName == "Blizzard_Channels"
            or addonName == "Blizzard_VoiceToggleButton" then
            _G.__wow_apply_chat_voice_button_surface()
        end
    end)
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CHAT_VOICE_BUTTON_SURFACE_WORKAROUND_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(CHAT_VOICE_BUTTON_SURFACE_WORKAROUND_LUA);
}

pub(crate) fn patch_loader(env: &LoaderEnv<'_>) {
    let _ = env.exec(CHAT_VOICE_BUTTON_SURFACE_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_chat_frame_and_initializes_channel_button_icon() {
        let env = WowLuaEnv::new().expect("env should initialize");
        env.exec(
            r#"
            ChatFrame1 = {}
            ChatFrame1EditBox = {}
            DEFAULT_CHAT_FRAME = nil
            QuickJoinToastButton = nil
            UIParent = {}
            createdFrames = {}
            function CreateFrame(frameType, name, parent)
                local frame = { frameType = frameType, name = name, parent = parent }
                createdFrames[name] = frame
                _G[name] = frame
                return frame
            end

            local icon = {
                width = 0,
                height = 0,
                points = 0,
                shown = false,
                SetParentKey = function(self, key, propagate)
                    self.parentKey = key
                    self.propagate = propagate
                end,
                GetWidth = function(self)
                    return self.width
                end,
                GetHeight = function(self)
                    return self.height
                end,
                SetSize = function(self, width, height)
                    self.width = width
                    self.height = height
                end,
                GetNumPoints = function(self)
                    return self.points
                end,
                SetPoint = function(self, ...)
                    self.points = self.points + 1
                    self.point = {...}
                end,
                SetAtlas = function(self, atlas)
                    self.atlas = atlas
                end,
                Show = function(self)
                    self.shown = true
                end,
            }
            ChatFrameChannelButton = {
                fixedIconWidth = 19,
                fixedIconHeight = 21,
                CreateTexture = function(self)
                    self.createdIcon = icon
                    return icon
                end,
            }
            "#,
        )
        .expect("chat voice test surface should install");

        patch(&env);

        let (
            default_is_chat_frame,
            edit_box_linked,
            icon_parent_key,
            icon_width,
            icon_height,
            icon_points,
            icon_atlas,
            icon_shown,
            quick_join_type,
        ): (bool, bool, String, i64, i64, i64, String, bool, String) = env
            .eval(
                r#"
                local icon = ChatFrameChannelButton.Icon
                return DEFAULT_CHAT_FRAME == ChatFrame1,
                    ChatFrame1.editBox == ChatFrame1EditBox and ChatFrame1EditBox.chatFrame == ChatFrame1,
                    icon.parentKey,
                    icon.width,
                    icon.height,
                    icon.points,
                    icon.atlas,
                    icon.shown,
                    QuickJoinToastButton.frameType
                "#,
            )
            .expect("patched chat voice state should be readable");

        assert!(default_is_chat_frame);
        assert!(edit_box_linked);
        assert_eq!(icon_parent_key, "Icon");
        assert_eq!(icon_width, 19);
        assert_eq!(icon_height, 21);
        assert_eq!(icon_points, 1);
        assert_eq!(icon_atlas, "chatframe-button-icon-voicechat");
        assert!(icon_shown);
        assert_eq!(quick_join_type, "Button");
    }
}
