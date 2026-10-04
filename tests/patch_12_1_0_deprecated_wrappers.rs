//! Retail 12.1.0 deprecated wrappers and TOC game-type gates, observed through
//! the cached Blizzard Lua under the default `loadDeprecationFallbacks=1`.
//!
//! The native simulator `getglobal`/`setglobal` remain registered; this file
//! proves the cached Shared/Deprecated_12_1_0.lua replaces them at load, not
//! that disabling fallbacks removes them.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BnetFriendInvite;

struct WrapperRow {
    source_id: &'static str,
    old_fn: &'static str,
    /// Cached file that must own the old function after load.
    defined_in: &'static str,
    seed: fn(&WowLuaEnv),
    /// Lua body returning "ok" or a failure label.
    probe: &'static str,
}

fn no_seed(_: &WowLuaEnv) {}

fn seed_bnet_invites(env: &WowLuaEnv) {
    let invite = |invite_id, battle_tag: &str, friend_level| BnetFriendInvite {
        invite_id,
        battle_tag: battle_tag.into(),
        account_name: battle_tag.split('#').next().unwrap().into(),
        friend_level,
        creation_timestamp: 1_760_000_000 + i64::from(invite_id),
    };
    env.state().borrow_mut().bnet_friend_invites =
        vec![invite(11, "Jaina#1111", 1), invite(12, "Thrall#2222", 2)];
}

const ROWS: &[WrapperRow] = &[
    WrapperRow {
        source_id: "deprecated api-getglobal-380",
        old_fn: "getglobal",
        defined_in: "Shared/Deprecated_12_1_0.lua",
        seed: no_seed,
        probe: r##"
            DeprecatedProbeValue = { 42 }
            if getglobal("DeprecatedProbeValue") ~= DeprecatedProbeValue then return "read" end
            if getglobal("DeprecatedProbeMissing") ~= nil then return "missing" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-setglobal-381",
        old_fn: "setglobal",
        defined_in: "Shared/Deprecated_12_1_0.lua",
        seed: no_seed,
        probe: r##"
            setglobal("DeprecatedProbeWritten", "value")
            if rawget(_G, "DeprecatedProbeWritten") ~= "value" then return "write" end
            if issecurevariable("DeprecatedProbeWritten") ~= false then return "taint" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-BNSendVerifiedBattleTagInvite-385",
        old_fn: "BNSendVerifiedBattleTagInvite",
        defined_in: "Deprecated_BattleNet.lua",
        seed: no_seed,
        probe: r##"
            local original = C_BattleNet.SendVerifiedBattleNetFriendInvite
            local calls, argCount = 0, nil
            C_BattleNet.SendVerifiedBattleNetFriendInvite = function(...)
                calls, argCount = calls + 1, select("#", ...)
            end
            local results = select("#", BNSendVerifiedBattleTagInvite("ignored"))
            C_BattleNet.SendVerifiedBattleNetFriendInvite = original
            if calls ~= 1 or argCount ~= 0 then return "delegate" end
            if results ~= 0 then return "returns" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-BNGetFriendInviteInfo-386",
        old_fn: "BNGetFriendInviteInfo",
        defined_in: "Deprecated_BattleNet.lua",
        seed: seed_bnet_invites,
        probe: r##"
            local n = select("#", BNGetFriendInviteInfo(1))
            local id, name, isBattleTag, unused, created = BNGetFriendInviteInfo(1)
            if n ~= 5 then return "count" end
            if id ~= 11 or name ~= "Jaina" or isBattleTag ~= true then return "first" end
            if unused ~= nil or created ~= 1760000011 then return "first-tail" end
            local id2, name2, isBattleTag2 = BNGetFriendInviteInfo(2)
            if id2 ~= 12 or name2 ~= "Thrall" or isBattleTag2 ~= false then return "realid" end
            if select("#", BNGetFriendInviteInfo(3)) ~= 0 then return "missing" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-C_DyeColor-GetDyeColorForItem-390",
        old_fn: "C_DyeColor.GetDyeColorForItem",
        defined_in: "Mainline/Deprecated_12_1_0.lua",
        seed: no_seed,
        probe: r##"
            local original = C_DyeColor.GetDyeColorsForItem
            local seen
            C_DyeColor.GetDyeColorsForItem = function(item)
                seen = item
                if item == 2001 then return { 7, 9 } end
                return {}
            end
            local first = C_DyeColor.GetDyeColorForItem(2001)
            local empty = C_DyeColor.GetDyeColorForItem(2002)
            C_DyeColor.GetDyeColorsForItem = original
            if first ~= 7 then return "first" end
            if empty ~= nil or seen ~= 2002 then return "empty" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-C_DyeColor-GetDyeColorForItemLocation-391",
        old_fn: "C_DyeColor.GetDyeColorForItemLocation",
        defined_in: "Mainline/Deprecated_12_1_0.lua",
        seed: no_seed,
        probe: r##"
            local original = C_DyeColor.GetDyeColorsForItemLocation
            local location = { bagID = 0, slotIndex = 1 }
            local seen
            C_DyeColor.GetDyeColorsForItemLocation = function(itemLocation)
                seen = itemLocation
                return { 12 }
            end
            local first = C_DyeColor.GetDyeColorForItemLocation(location)
            C_DyeColor.GetDyeColorsForItemLocation = original
            if first ~= 12 or seen ~= location then return "delegate" end
            if C_DyeColor.GetDyeColorForItemLocation(location) ~= nil then return "empty" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-RaidNotice_AddMessage-395",
        old_fn: "RaidNotice_AddMessage",
        defined_in: "Deprecated_RaidWarning.lua",
        seed: no_seed,
        probe: r##"
            RaidWarningFrame:ClearMessages()
            RaidNotice_AddMessage(RaidWarningFrame, "Pull in 5", ChatTypeInfo["RAID_WARNING"], 4)
            if RaidWarningFrame:GetActiveMessageCount() ~= 1 then return "count" end
            if RaidWarningFrame:GetLowestMessage():GetText() ~= "Pull in 5" then return "text" end
            return "ok"
        "##,
    },
    WrapperRow {
        source_id: "deprecated api-RaidNotice_Clear-396",
        old_fn: "RaidNotice_Clear",
        defined_in: "Deprecated_RaidWarning.lua",
        seed: no_seed,
        probe: r##"
            RaidWarningFrame:AddMessage("Stack", ChatTypeInfo["RAID_WARNING"], 4)
            if RaidWarningFrame:GetActiveMessageCount() == 0 then return "setup" end
            RaidNotice_Clear(RaidWarningFrame)
            if RaidWarningFrame:GetActiveMessageCount() ~= 0 then return "clear" end
            return "ok"
        "##,
    },
];

fn run_row(env: &WowLuaEnv, row: &WrapperRow) -> String {
    (row.seed)(env);
    let ownership = format!(
        r##"
        if not GetCVarBool("loadDeprecationFallbacks") then return "fallbacks-disabled" end
        local fn = {old}
        if type(fn) ~= "function" then return "missing " .. type(fn) end
        local source = debug.getinfo(fn, "S").source or ""
        if not string.find(source, {file:?}, 1, true) then return "owner " .. source end
        return "ok"
        "##,
        old = row.old_fn,
        file = row.defined_in,
    );
    let owned: String = env.eval(&ownership).unwrap_or_else(|e| e.to_string());
    if owned != "ok" {
        return owned;
    }
    env.eval(row.probe).unwrap_or_else(|e| e.to_string())
}

prefork_full_ui_case! {
fn patch_12_1_0_deprecated_wrappers_delegate_to_new_apis(env: &WowLuaEnv) {
    let failures: Vec<String> = ROWS
        .iter()
        .filter_map(|row| {
            let result = run_row(env, row);
            (result != "ok").then(|| format!("{} ({}): {result}", row.source_id, row.old_fn))
        })
        .collect();
    assert!(failures.is_empty(), "deprecated wrapper rows failed:\n  {}", failures.join("\n  "));
}
}

prefork_full_ui_case! {
fn patch_12_1_0_secure_header_templates_follow_toc_game_type(env: &WowLuaEnv) {
    // prose-2026-07-07-148: SecureAuraHeader.lua/.xml carry
    // [AllowLoadGameType classic] in the cached TOC, so retail never loads them.
    // prose-2026-07-07-151: SecureGroupHeaderTemplate stays available.
    let result: String = env
        .eval(
            r##"
            if C_XMLUtil.GetTemplateInfo("SecureAuraHeaderTemplate") ~= nil then return "aura-template" end
            if SecureAuraHeader_Update ~= nil then return "aura-lua" end
            local info = C_XMLUtil.GetTemplateInfo("SecureGroupHeaderTemplate")
            if type(info) ~= "table" then return "group-template" end
            local header = CreateFrame("Frame", "Patch1210GroupHeader", UIParent, "SecureGroupHeaderTemplate")
            if type(header:GetScript("OnEvent")) ~= "function" then return "group-inherit" end
            return "ok"
            "##,
        )
        .unwrap_or_else(|e| e.to_string());
    assert_eq!(result, "ok");
}
}
