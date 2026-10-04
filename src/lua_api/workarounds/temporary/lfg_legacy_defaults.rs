//! Temporary legacy LFG global defaults.
//!
//! Modern LFG list behavior is state-backed elsewhere. This module keeps small
//! legacy global probes explicit until those callers are modeled properly.

const LFG_LEGACY_DEFAULTS_LUA: &str = r#"
local function __wow_lfg_ensure_child(parent, key, frameType)
    local child = rawget(parent, key)
    if child ~= nil then
        return child
    end
    if type(CreateFrame) == "function" then
        local ok, frame = pcall(CreateFrame, frameType or "Frame", nil, parent)
        if ok then
            child = frame
        end
    end
    if child == nil then
        child = {}
    end
    rawset(parent, key, child)
    return child
end

do
    local lfgListFrame = rawget(_G, "LFGListFrame")
    if lfgListFrame == nil and type(CreateFrame) == "function" then
        lfgListFrame = CreateFrame("Frame", "LFGListFrame", UIParent)
    end
    if type(lfgListFrame) == "table" then
        local searchPanel = __wow_lfg_ensure_child(lfgListFrame, "SearchPanel", "Frame")
        __wow_lfg_ensure_child(searchPanel, "SearchBox", "EditBox")
    end
end

if GetLFGCategoryForID == nil then
    function GetLFGCategoryForID() return 0 end
end

local __wow_lfg_role_icons = {
    GUIDE = "UI-LFG-RoleIcon-Leader",
    TANK = "UI-LFG-RoleIcon-Tank",
    HEALER = "UI-LFG-RoleIcon-Healer",
    DAMAGER = "UI-LFG-RoleIcon-DPS",
    NONE = "UI-LFG-RoleIcon-DPS",
}

local __wow_lfg_role_icons_disabled = {
    GUIDE = "UI-LFG-RoleIcon-Leader-Disabled",
    TANK = "UI-LFG-RoleIcon-Tank-Disabled",
    HEALER = "UI-LFG-RoleIcon-Healer-Disabled",
    DAMAGER = "UI-LFG-RoleIcon-DPS-Disabled",
    NONE = "UI-LFG-RoleIcon-DPS-Disabled",
}

if GetIconForRole == nil then
    function GetIconForRole(role, showDisabled)
        local iconSet = showDisabled and __wow_lfg_role_icons_disabled or __wow_lfg_role_icons
        return iconSet[role] or iconSet.NONE
    end
end

local function __wow_lfg_role_name_from_enum(role)
    if role == 0 then
        return "TANK"
    end
    if role == 1 then
        return "HEALER"
    end
    if role == 2 then
        return "DAMAGER"
    end
    if Constants ~= nil
        and Constants.LFG_ROLEConstants ~= nil
        and role == Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE then
        return "GUIDE"
    end
    return "NONE"
end

if GetIconForRoleEnum == nil then
    function GetIconForRoleEnum(role, showDisabled)
        return GetIconForRole(__wow_lfg_role_name_from_enum(role), showDisabled)
    end
end

if GetLFDRoleRestrictions == nil then
    function GetLFDRoleRestrictions(_lfgID)
        return false, false, false
    end
end
if GetLFGRoleShortageRewards == nil then
    function GetLFGRoleShortageRewards(_lfgID, _shortageIndex)
        return false, false, false, false, 0, 0, 0
    end
end

if rawget(C_LFGInfo or {}, "CanPlayerUseGroupFinder") == nil then
    C_LFGInfo = C_LFGInfo or __wow_namespace()
    function C_LFGInfo.CanPlayerUseGroupFinder()
        return false, ""
    end
end
if rawget(C_LFGInfo or {}, "IsInLFGFollowerDungeon") == nil then
    C_LFGInfo = C_LFGInfo or __wow_namespace()
    function C_LFGInfo.IsInLFGFollowerDungeon()
        return false
    end
end
if rawget(C_LFGInfo or {}, "CanPlayerUseLFR") == nil then
    C_LFGInfo = C_LFGInfo or __wow_namespace()
    function C_LFGInfo.CanPlayerUseLFR()
        return true, nil
    end
end
if rawget(C_LFGInfo or {}, "GetDungeonInfo") == nil then
    C_LFGInfo = C_LFGInfo or __wow_namespace()
    function C_LFGInfo.GetDungeonInfo(_dungeonID)
        return {}
    end
end
if rawget(C_LFGInfo or {}, "GetLFDLockStates") == nil then
    C_LFGInfo = C_LFGInfo or __wow_namespace()
    function C_LFGInfo.GetLFDLockStates(_lfgID)
        return {}
    end
end
if rawget(C_LFGInfo or {}, "GetAllEntriesForCategory") == nil then
    C_LFGInfo = C_LFGInfo or __wow_namespace()
    function C_LFGInfo.GetAllEntriesForCategory(_categoryID)
        return {}
    end
end
if GetLFGProposal == nil then
    function GetLFGProposal()
        return false, 0, 0, 0, "", "", "", false, 0, 0, 0, false, false, nil, false
    end
end
if GetLFGProposalEncounter == nil then
    function GetLFGProposalEncounter(_i)
        return "", "", false
    end
end
if GetLFGInfoServer == nil then
    function GetLFGInfoServer()
        return false, false, false, false, false, 0, 0, 0, ""
    end
end
if GetLFGRoleUpdate == nil then
    function GetLFGRoleUpdate()
        return false, 0, 0, 0, 0, false
    end
end
if GetLFGQueuedList == nil then
    function GetLFGQueuedList(_category, queuedList)
        queuedList = queuedList or {}
        for key in pairs(queuedList) do
            queuedList[key] = nil
        end
        return queuedList
    end
end
if GetLFGReadyCheckUpdate == nil then
    function GetLFGReadyCheckUpdate()
        return false, false
    end
end
if GetPartyLFGID == nil then
    function GetPartyLFGID() return 0 end
end
if GetLFGDeserterExpiration == nil then
    function GetLFGDeserterExpiration() return 0 end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(LFG_LEGACY_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_lfg_category_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let category_id: i32 = env
            .eval("return GetLFGCategoryForID(123)")
            .expect("legacy LFG category probe should run");

        assert_eq!(category_id, 0);
    }

    #[test]
    fn installs_lfg_role_icon_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let icons: (String, String, String, String, String) = env
            .eval(
                r#"
                return GetIconForRole("TANK", false),
                       GetIconForRole("HEALER", true),
                       GetIconForRoleEnum(Enum.LFGRole.Damage, false),
                       GetIconForRoleEnum(Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE, false),
                       GetIconForRoleEnum(999, true)
                "#,
            )
            .expect("legacy LFG role icon probe should run");

        assert_eq!(
            icons,
            (
                "UI-LFG-RoleIcon-Tank".to_string(),
                "UI-LFG-RoleIcon-Healer-Disabled".to_string(),
                "UI-LFG-RoleIcon-DPS".to_string(),
                "UI-LFG-RoleIcon-Leader".to_string(),
                "UI-LFG-RoleIcon-DPS-Disabled".to_string(),
            )
        );
    }

    #[test]
    fn preserves_existing_lfg_role_icon_function() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("function GetIconForRole() return 'existing' end")
            .expect("fixture should install existing LFG icon function");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy LFG defaults should apply");
        }

        let icon: String = env
            .eval("return GetIconForRole('TANK', false)")
            .expect("legacy LFG icon preservation probe should run");

        assert_eq!(icon, "existing");
    }

    #[test]
    fn installs_role_restriction_and_shortage_reward_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local tank, healer, damage = GetLFDRoleRestrictions(1)
                if tank ~= false or healer ~= false or damage ~= false then
                    return "restrictions"
                end

                local eligible, forTank, forHealer, forDamage, itemID, money, xp = GetLFGRoleShortageRewards(1, 1)
                if eligible ~= false or forTank ~= false or forHealer ~= false or forDamage ~= false then
                    return "reward_flags"
                end
                if itemID ~= 0 or money ~= 0 or xp ~= 0 then
                    return "reward_values"
                end

                return "ok"
                "#,
            )
            .expect("legacy LFG role defaults probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_role_restriction_and_shortage_reward_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function GetLFDRoleRestrictions()
                return true, false, true
            end
            function GetLFGRoleShortageRewards()
                return true, true, false, true, 17, 23, 31
            end
            "#,
        )
        .expect("fixture should install existing LFG role functions");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy LFG defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                local tank, healer, damage = GetLFDRoleRestrictions(1)
                if tank ~= true or healer ~= false or damage ~= true then
                    return "restrictions"
                end

                local eligible, forTank, forHealer, forDamage, itemID, money, xp = GetLFGRoleShortageRewards(1, 1)
                if eligible ~= true or forTank ~= true or forHealer ~= false or forDamage ~= true then
                    return "reward_flags"
                end
                if itemID ~= 17 or money ~= 23 or xp ~= 31 then
                    return "reward_values"
                end

                return "ok"
                "#,
            )
            .expect("legacy LFG role preservation probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_legacy_lfg_fallback_shapes_when_missing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_LFGInfo.CanPlayerUseGroupFinder = nil
            C_LFGInfo.IsInLFGFollowerDungeon = nil
            C_LFGInfo.CanPlayerUseLFR = nil
            C_LFGInfo.GetDungeonInfo = nil
            C_LFGInfo.GetLFDLockStates = nil
            C_LFGInfo.GetAllEntriesForCategory = nil
            GetLFGProposal = nil
            GetLFGProposalEncounter = nil
            GetLFGInfoServer = nil
            GetLFGRoleUpdate = nil
            GetLFGQueuedList = nil
            GetLFGReadyCheckUpdate = nil
            GetPartyLFGID = nil
            GetLFGDeserterExpiration = nil
            "#,
        )
        .expect("fixture should clear legacy LFG globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy LFG defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                local canUse, reason = C_LFGInfo.CanPlayerUseGroupFinder()
                if canUse ~= false or reason ~= "" then return "group_finder" end
                if C_LFGInfo.IsInLFGFollowerDungeon() ~= false then return "follower" end
                local canUseLfr, lfrReason = C_LFGInfo.CanPlayerUseLFR()
                if canUseLfr ~= true or lfrReason ~= nil then return "lfr" end
                if #C_LFGInfo.GetDungeonInfo(1) ~= 0 then return "dungeon_info" end
                if #C_LFGInfo.GetLFDLockStates(1) ~= 0 then return "lock_states" end
                if #C_LFGInfo.GetAllEntriesForCategory(1) ~= 0 then return "category_entries" end

                local proposalExists, id, typeID, subtypeID, name, bg, role, responded, bosses, completed, members, leader, holiday, proposalCategory, silent = GetLFGProposal()
                if proposalExists ~= false or id ~= 0 or typeID ~= 0 or subtypeID ~= 0 then return "proposal_ids" end
                if name ~= "" or bg ~= "" or role ~= "" or responded ~= false then return "proposal_strings" end
                if bosses ~= 0 or completed ~= 0 or members ~= 0 or leader ~= false or holiday ~= false or proposalCategory ~= nil or silent ~= false then return "proposal_tail" end

                local bossName, texture, killed = GetLFGProposalEncounter(1)
                if bossName ~= "" or texture ~= "" or killed ~= false then return "encounter" end

                local server1, server2, server3, server4, server5, server6, server7, server8, server9 = GetLFGInfoServer()
                if server1 ~= false or server2 ~= false or server3 ~= false or server4 ~= false or server5 ~= false then return "server_flags" end
                if server6 ~= 0 or server7 ~= 0 or server8 ~= 0 or server9 ~= "" then return "server_tail" end

                local inProgress, slots, memberCount, category, lfgID, bgQueue = GetLFGRoleUpdate()
                if inProgress ~= false or slots ~= 0 or memberCount ~= 0 or category ~= 0 or lfgID ~= 0 or bgQueue ~= false then return "role_update" end

                local queued = { stale = true }
                local returned = GetLFGQueuedList(1, queued)
                if returned ~= queued or next(queued) ~= nil then return "queued" end

                local ready, accepted = GetLFGReadyCheckUpdate()
                if ready ~= false or accepted ~= false then return "ready" end
                if GetPartyLFGID() ~= 0 then return "party_id" end
                if GetLFGDeserterExpiration() ~= 0 then return "deserter" end
                return "ok"
                "#,
            )
            .expect("legacy LFG fallback shape probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_lfg_list_frame_search_surface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(LFGListFrame) ~= "table" then return "frame" end
                if type(LFGListFrame.SearchPanel) ~= "table" then return "panel" end
                if type(LFGListFrame.SearchPanel.SearchBox) ~= "table" then return "box" end
                return "ok"
                "#,
            )
            .expect("LFGListFrame search surface probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_lfg_category_function() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("function GetLFGCategoryForID() return 7 end")
            .expect("fixture should install existing LFG category function");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy LFG defaults should apply");
        }

        let category_id: i32 = env
            .eval("return GetLFGCategoryForID(123)")
            .expect("legacy LFG category preservation probe should run");

        assert_eq!(category_id, 7);
    }
}
