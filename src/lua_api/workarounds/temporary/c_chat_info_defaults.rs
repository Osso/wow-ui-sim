//! Temporary `C_ChatInfo` no-state defaults.
//!
//! Channel lookup and message sending are Rust-backed elsewhere. These
//! emote/caution/chat-line compatibility methods stay explicit here until the
//! simulator models the related chat state.

const C_CHAT_INFO_DEFAULTS_LUA: &str = r#"
C_ChatInfo = C_ChatInfo or __wow_namespace()

local function installChatInfoDefault(name, fn)
    if rawget(C_ChatInfo, name) == nil then
        C_ChatInfo[name] = fn
    end
end

installChatInfoDefault("PerformEmote", function(_emoteToken)
    return false
end)

installChatInfoDefault("CancelEmote", function()
end)

installChatInfoDefault("IsValidChatLine", function(_chatLine)
    return false
end)

installChatInfoDefault("UncensorChatLine", function(_chatLine)
end)

installChatInfoDefault("DropCautionaryChatMessage", function(_messageID)
end)

installChatInfoDefault("SendCautionaryChatMessage", function(_messageID)
end)

installChatInfoDefault("AreOutgoingAddonChatMessagesRestricted", function()
    return false
end)

installChatInfoDefault("GetNumReservedChatWindows", function()
    return 0
end)

installChatInfoDefault("GetChannelRulesetForChannelID", function(_channelID)
    return 0
end)

installChatInfoDefault("GetChannelRuleset", function(_channelName)
    return 0
end)

installChatInfoDefault("GetChatLineText", function(_chatFrame, _chatLine)
    return nil
end)

installChatInfoDefault("IsTimerunningPlayer", function(_guid)
    return false
end)

installChatInfoDefault("GetChannelShortcutForChannelID", function(_channelID)
    return ""
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(C_CHAT_INFO_DEFAULTS_LUA)?;
    #[cfg(not(feature = "retail-12-0-5"))]
    lua.exec(
        r#"
if rawget(C_ChatInfo, "ReplaceIconAndGroupExpressions") == nil then
    C_ChatInfo.ReplaceIconAndGroupExpressions = function(text) return text end
end
"#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_chat_info_no_state_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if C_ChatInfo.PerformEmote("wave") ~= false then return "perform" end
                if C_ChatInfo.IsValidChatLine(1) ~= false then return "valid" end
                if C_ChatInfo.ReplaceIconAndGroupExpressions("hello") ~= "hello" then return "replace" end
                if C_ChatInfo.AreOutgoingAddonChatMessagesRestricted() ~= false then return "restricted" end
                if C_ChatInfo.GetNumReservedChatWindows() ~= 0 then return "reserved" end
                if C_ChatInfo.GetChannelRulesetForChannelID(1) ~= 0 then return "ruleset-id" end
                if C_ChatInfo.GetChannelRuleset("General") ~= 0 then return "ruleset" end
                if C_ChatInfo.GetChatLineText(1, 1) ~= nil then return "line-text" end
                if C_ChatInfo.IsTimerunningPlayer() ~= false then return "timerunning" end
                if C_ChatInfo.GetChannelShortcutForChannelID(1) ~= "" then return "shortcut-id" end
                C_ChatInfo.CancelEmote()
                C_ChatInfo.UncensorChatLine(1)
                C_ChatInfo.DropCautionaryChatMessage(1)
                C_ChatInfo.SendCautionaryChatMessage(1)
                return "ok"
                "#,
            )
            .expect("C_ChatInfo no-state defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_chat_info_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_ChatInfo = C_ChatInfo or __wow_namespace()

            function C_ChatInfo.PerformEmote(_emoteToken)
                return true
            end
            function C_ChatInfo.GetNumReservedChatWindows()
                return 3
            end
            function C_ChatInfo.ReplaceIconAndGroupExpressions(text)
                return "existing:" .. text
            end
            "#,
        )
        .expect("fixture should install existing C_ChatInfo providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                return tostring(C_ChatInfo.PerformEmote("wave")) .. ":" ..
                    C_ChatInfo.GetNumReservedChatWindows() .. ":" ..
                    C_ChatInfo.ReplaceIconAndGroupExpressions("hello")
                "#,
            )
            .expect("existing C_ChatInfo providers should remain callable");

        assert_eq!(result, "true:3:existing:hello");
    }
}
