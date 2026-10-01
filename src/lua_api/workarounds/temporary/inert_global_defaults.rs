//! Temporary inert global defaults for unmodeled world/social state.
//!
//! These functions are startup compatibility fallbacks. The simulator does not
//! model world-entry state, battleground queues, social restrictions,
//! commentator mode, or group role composition yet, so keep the defaults
//! explicit in the workaround layer.

const INERT_GLOBAL_DEFAULTS_LUA: &str = r#"
if IsPlayerInWorld == nil then
    function IsPlayerInWorld()
        return true
    end
end
if GetCurrentRegionName == nil then
    function GetCurrentRegionName() return "US" end
end
if GetDefaultLanguage == nil then
    function GetDefaultLanguage() return "Common", 1 end
end
if GetAlternativeDefaultLanguage == nil then
    function GetAlternativeDefaultLanguage() return nil end
end
if GetNumLanguages == nil then
    function GetNumLanguages() return 1 end
end
if GetLanguageByIndex == nil then
    function GetLanguageByIndex(index)
        if index == 1 then
            return GetDefaultLanguage()
        end
        return nil
    end
end
if GetMaxBattlefieldID == nil then
    function GetMaxBattlefieldID() return 0 end
end
if IsActiveBattlefieldArena == nil then
    function IsActiveBattlefieldArena() return false end
end
if IsPVPTimerRunning == nil then
    function IsPVPTimerRunning() return false end
end
if HasArtifactEquipped == nil then
    function HasArtifactEquipped() return false end
end
if UnitExists == nil then
    function UnitExists(unit)
        return unit == "player"
    end
end
EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE = false
CLOCK_TICKER_Y_OVERRIDE = CLOCK_TICKER_Y_OVERRIDE or false

C_SocialRestrictions = C_SocialRestrictions or __wow_namespace()
if rawget(C_SocialRestrictions, "IsChatDisabled") == nil then
    function C_SocialRestrictions.IsChatDisabled() return false end
end
if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    if rawget(C_SocialRestrictions, "IsFriendsDisabled") == nil then
        function C_SocialRestrictions.IsFriendsDisabled() return false end
    end
    C_SocialUI = C_SocialUI or __wow_namespace()
    if rawget(C_SocialUI, "IsSystemEnabled") == nil then
        function C_SocialUI.IsSystemEnabled() return false end
    end
end
C_Commentator = C_Commentator or __wow_namespace()
if rawget(C_Commentator, "IsSpectating") == nil then
    function C_Commentator.IsSpectating() return false end
end
if rawget(C_Commentator, "SendAddonMessage") == nil then
    function C_Commentator.SendAddonMessage(_prefix, _message, _channel)
        return Enum and Enum.SendAddonMessageResult and Enum.SendAddonMessageResult.Success or 0
    end
end

C_GuildBank = C_GuildBank or __wow_namespace()

C_FriendList = C_FriendList or __wow_namespace()
if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    if rawget(C_FriendList, "IsLegacyFriendSystemEnabled") == nil then
        function C_FriendList.IsLegacyFriendSystemEnabled() return false end
    end
end
if rawget(C_FriendList, "GetNumFriends") == nil then
    function C_FriendList.GetNumFriends() return 0 end
end
if rawget(C_FriendList, "GetNumOnlineFriends") == nil then
    function C_FriendList.GetNumOnlineFriends() return 0 end
end
if rawget(C_FriendList, "GetNumIgnores") == nil then
    function C_FriendList.GetNumIgnores() return 0 end
end
if rawget(C_FriendList, "GetIgnoreName") == nil then
    function C_FriendList.GetIgnoreName() return nil end
end

if ACCOUNT_BINDINGS == nil then ACCOUNT_BINDINGS = 1 end
if CHARACTER_BINDINGS == nil then CHARACTER_BINDINGS = 2 end
if CHARACTERBINDINGS == nil then CHARACTERBINDINGS = CHARACTER_BINDINGS end
if IsOnTournamentRealm == nil then
    function IsOnTournamentRealm()
        return false
    end
end
if GetNumDisplayChannels == nil then
    function GetNumDisplayChannels()
        return 0
    end
end
if GetChannelDisplayInfo == nil then
    function GetChannelDisplayInfo(_index)
        return nil
    end
end

if BNGetNumFriends == nil then
    function BNGetNumFriends() return 0, 0, 0, 0 end
end
if BNGetNumFriendInvites == nil then
    function BNGetNumFriendInvites() return 0 end
end
if BNGetFriendInfo == nil then
    function BNGetFriendInfo() return nil end
end
if Ambiguate == nil then
    function Ambiguate(fullName, context)
        if context == "none" then
            return fullName
        end
        return string.match(fullName, "^(.-)%-.+$") or fullName
    end
end
if AreTalentsLocked == nil then
    function AreTalentsLocked() return false end
end

if GetGuildFactionGroup == nil then
    function GetGuildFactionGroup()
        return 1
    end
end
if GetGroupMemberCounts == nil then
    function GetGroupMemberCounts()
        return {
            TANK = 0,
            HEALER = 0,
            DAMAGER = 0,
            NOROLE = 0,
        }
    end
end
if GetLootSpecialization == nil then
    function GetLootSpecialization()
        return 0
    end
end
if HasLootSpecializations == nil then
    function HasLootSpecializations()
        return true
    end
end
if WorldLootObjectExists == nil then
    function WorldLootObjectExists(_unit)
        return false
    end
end
if CanShowSetRoleButton == nil then
    function CanShowSetRoleButton()
        return false
    end
end

if GetActiveLootRollIDs == nil then
    function GetActiveLootRollIDs()
        return {}
    end
end

if GetNumArenaOpponents == nil then
    function GetNumArenaOpponents()
        return 0
    end
end
if GetNumArenaOpponentSpecs == nil then
    function GetNumArenaOpponentSpecs()
        return 0
    end
end

if GetItemLevelColor == nil then
    function GetItemLevelColor()
        return 1, 1, 1
    end
end
if ClearCursorHoveredItem == nil then
    function ClearCursorHoveredItem()
        return nil
    end
end
if SetCursorHoveredItem == nil then
    function SetCursorHoveredItem(_itemLocation)
        return nil
    end
end
if SetCursorHoveredItemTradeItem == nil then
    function SetCursorHoveredItemTradeItem(_enabled)
        return nil
    end
end
if UnitInSubgroup == nil then
    function UnitInSubgroup(unit)
        if unit == nil or unit == "player" then
            return false
        end
        return type(UnitInParty) == "function" and UnitInParty(unit) or false
    end
end

if GetNumGuildPerks == nil then
    function GetNumGuildPerks()
        return 0
    end
end
if RequestGuildRewards == nil then
    function RequestGuildRewards()
        return nil
    end
end
if GetGuildRenameRequired == nil then
    function GetGuildRenameRequired()
        return false
    end
end
if GetAvailableBandwidth == nil then
    function GetAvailableBandwidth()
        local bandwidthIn, bandwidthOut = GetNetStats()
        return math.max(tonumber(bandwidthIn) or 0, tonumber(bandwidthOut) or 0)
    end
end
"#;

// Historical/nonmainline profiles retain only their prior inert query.
#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
const UNMODELED_SPELL_CONFIRMATION_QUERY_LUA: &str = r#"
if GetSpellConfirmationPromptsInfo == nil then
    function GetSpellConfirmationPromptsInfo()
        return {}
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )))]
    lua.exec(UNMODELED_SPELL_CONFIRMATION_QUERY_LUA)?;
    lua.exec(INERT_GLOBAL_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_region_social_and_group_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local counts = GetGroupMemberCounts()
                if IsPlayerInWorld() ~= true then return "player_world" end
                if GetCurrentRegionName() ~= "US" then return "region" end
                local languageName, languageID = GetDefaultLanguage()
                if languageName ~= "Common" or languageID ~= 1 then return "language" end
                if GetAlternativeDefaultLanguage() ~= nil then return "alt_language" end
                if GetNumLanguages() ~= 1 then return "num_languages" end
                local indexedLanguageName, indexedLanguageID = GetLanguageByIndex(1)
                if indexedLanguageName ~= "Common" or indexedLanguageID ~= 1 then return "indexed_language" end
                if GetLanguageByIndex(2) ~= nil then return "missing_indexed_language" end
                if GetMaxBattlefieldID() ~= 0 then return "battlefield_id" end
                if IsActiveBattlefieldArena() ~= false then return "battlefield_arena" end
                if IsPVPTimerRunning() ~= false then return "pvp_timer" end
                if HasArtifactEquipped() ~= false then return "artifact" end
                if UnitExists("player") ~= true or UnitExists("target") ~= false then return "unit" end
                if EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE ~= false then return "toast_offset" end
                if CLOCK_TICKER_Y_OVERRIDE ~= false then return "clock_ticker" end
                if C_SocialRestrictions.IsChatDisabled() ~= false then return "social" end
                if C_Commentator.IsSpectating() ~= false then return "spectating" end
                if C_Commentator.SendAddonMessage("A", "B", "WHISPER") ~= Enum.SendAddonMessageResult.Success then return "send" end
                if type(C_GuildBank) ~= "table" then return "guild_bank" end
                if type(C_FriendList.GetNumFriends()) ~= "number" then return "friends" end
                if type(C_FriendList.GetNumOnlineFriends()) ~= "number" then return "online_friends" end
                if C_FriendList.GetNumIgnores() ~= 0 then return "ignores" end
                if C_FriendList.GetIgnoreName(1) ~= nil then return "ignore_name" end
                if ACCOUNT_BINDINGS ~= 1 or CHARACTER_BINDINGS ~= 2 or CHARACTERBINDINGS ~= CHARACTER_BINDINGS then return "bindings" end
                if IsOnTournamentRealm() ~= false then return "tournament" end
                if GetNumDisplayChannels() ~= 0 then return "display_channels" end
                if GetChannelDisplayInfo(1) ~= nil then return "channel_display_info" end
                local bnetTotal, bnetOnline, bnetFavorite, bnetMobile = BNGetNumFriends()
                if bnetTotal ~= 0 or bnetOnline ~= 0 or bnetFavorite ~= 0 or bnetMobile ~= 0 then return "bnet_counts" end
                if BNGetNumFriendInvites() ~= 0 then return "bnet_invites" end
                if BNGetFriendInfo(1) ~= nil then return "bnet_info" end
                if Ambiguate("Alyth-Realm", "short") ~= "Alyth" then return "ambiguate_short" end
                if Ambiguate("Alyth-Realm", "none") ~= "Alyth-Realm" then return "ambiguate_none" end
                if AreTalentsLocked() ~= false then return "talents_locked" end
                if GetGuildFactionGroup() ~= 1 then return "guild_faction" end
                if counts.TANK ~= 0 or counts.HEALER ~= 0 or counts.DAMAGER ~= 0 or counts.NOROLE ~= 0 then return "counts" end
                if GetLootSpecialization() ~= 0 then return "loot_spec" end
                if HasLootSpecializations() ~= true then return "has_loot_specs" end
                if WorldLootObjectExists("player") ~= false then return "world_loot_object" end
                if CanShowSetRoleButton() ~= false then return "role_button" end
                if #GetSpellConfirmationPromptsInfo() ~= 0 then return "spell_prompts" end
                if #GetActiveLootRollIDs() ~= 0 then return "loot_rolls" end
                if GetNumArenaOpponents() ~= 0 then return "arena" end
                if GetNumArenaOpponentSpecs() ~= 0 then return "arena_specs" end
                return "ok"
                "#,
            )
            .expect("inert defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function IsPlayerInWorld() return false end
            function GetCurrentRegionName() return "EU" end
            function GetAlternativeDefaultLanguage() return "Orcish", 2 end
            function GetNumLanguages() return 2 end
            function GetLanguageByIndex(index)
                if index == 1 then return "Common", 1 end
                if index == 2 then return "Orcish", 2 end
                return nil
            end
            function IsPVPTimerRunning() return true end
            function HasArtifactEquipped() return true end
            C_SocialRestrictions.IsChatDisabled = function() return true end
            C_Commentator.ExistingMember = 7
            C_FriendList.GetNumFriends = function() return 3 end
            EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE = true
            CLOCK_TICKER_Y_OVERRIDE = true
            ACCOUNT_BINDINGS = 9
            function BNGetNumFriends() return 5, 4, 3, 2 end
            function Ambiguate() return "Existing" end
            function WorldLootObjectExists() return true end
            function GetNumArenaOpponentSpecs() return 3 end
            "#,
        )
        .expect("fixture should install existing members");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("inert defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if IsPlayerInWorld() ~= false then return "overwrote_world_state" end
                if GetCurrentRegionName() ~= "EU" then return "overwrote_global" end
                local altLanguageName, altLanguageID = GetAlternativeDefaultLanguage()
                if altLanguageName ~= "Orcish" or altLanguageID ~= 2 then return "overwrote_alt_language" end
                if GetNumLanguages() ~= 2 then return "overwrote_num_languages" end
                local indexedLanguageName, indexedLanguageID = GetLanguageByIndex(2)
                if indexedLanguageName ~= "Orcish" or indexedLanguageID ~= 2 then return "overwrote_indexed_language" end
                if IsPVPTimerRunning() ~= true then return "overwrote_pvp_timer" end
                if HasArtifactEquipped() ~= true then return "overwrote_artifact" end
                if C_SocialRestrictions.IsChatDisabled() ~= true then return "overwrote_namespace_member" end
                if C_Commentator.ExistingMember ~= 7 then return "lost_member" end
                if type(C_Commentator.SendAddonMessage) ~= "function" then return "missing_default" end
                if C_FriendList.GetNumFriends() ~= 3 then return "overwrote_friend_list" end
                if type(C_FriendList.GetNumOnlineFriends) ~= "function" then return "missing_friend_default" end
                if ACCOUNT_BINDINGS ~= 9 then return "overwrote_binding" end
                local total, online, favorite, mobile = BNGetNumFriends()
                if total ~= 5 or online ~= 4 or favorite ~= 3 or mobile ~= 2 then return "overwrote_bnet_counts" end
                if Ambiguate("A-B", "short") ~= "Existing" then return "overwrote_ambiguate" end
                if type(BNGetNumFriendInvites) ~= "function" then return "missing_bnet_default" end
                if WorldLootObjectExists("player") ~= true then return "overwrote_world_loot_object" end
                if type(ClearCursorHoveredItem) ~= "function" then return "missing_cursor_hover_default" end
                if UnitInSubgroup("player") ~= false then return "bad_player_subgroup_default" end
                if GetNumGuildPerks() ~= 0 then return "bad_guild_perks_default" end
                if GetNumArenaOpponentSpecs() ~= 3 then return "overwrote_arena_specs" end
                if EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE ~= false then return "toast_offset_overwrite" end
                if CLOCK_TICKER_Y_OVERRIDE ~= true then return "overwrote_clock_ticker" end
                return "ok"
                "#,
            )
            .expect("preservation probe should run");

        assert_eq!(result, "ok");
    }
}
