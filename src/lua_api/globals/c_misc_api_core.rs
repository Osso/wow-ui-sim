//! Core C_* namespace API stubs.
//!
//! Contains C_ namespaces for game systems:
//! - C_ScenarioInfo, C_TooltipInfo, C_TradeSkillUI
//! - C_MythicPlus, C_LFGInfo, C_NamePlate, C_PlayerInfo
//! - C_PartyInfo, C_ChatInfo, C_EventUtils, C_AzeriteEssence
//! - C_PvP, C_FriendList, C_AuctionHouse, C_Bank
//! - C_EncounterJournal, C_GMTicketInfo, C_UnitAuras, C_CurrencyInfo

use mlua::{Lua, Result, Value};

pub(super) fn register_all(lua: &Lua) -> Result<()> {
    register_c_scenario_info(lua)?;
    register_c_tooltip_info(lua)?;
    register_c_trade_skill(lua)?;
    register_c_mythic_plus(lua)?;
    register_c_lfg_info(lua)?;
    register_c_nameplate(lua)?;
    register_c_player_info(lua)?;
    register_c_party_info(lua)?;
    register_c_chat_info(lua)?;
    register_c_event_utils(lua)?;
    register_c_azerite_essence(lua)?;
    register_c_pvp(lua)?;
    register_c_friend_list(lua)?;
    register_c_auction_house(lua)?;
    register_c_bank(lua)?;
    register_c_encounter_journal(lua)?;
    register_c_gm_ticket_info(lua)?;
    register_c_unit_auras(lua)?;
    register_c_currency_info(lua)?;
    Ok(())
}

fn register_c_scenario_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetScenarioInfo", lua.create_function(|_, ()| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Nil, Value::Integer(0), Value::Integer(0),
            Value::Integer(0), Value::Boolean(false), Value::Boolean(false),
        ]))
    })?)?;
    t.set("GetScenarioStepInfo", lua.create_function(|_, _step: Option<i32>| {
        Ok((Value::Nil, Value::Nil, Value::Integer(0), Value::Integer(0)))
    })?)?;
    let criteria_stub = lua.create_function(|_, _args: mlua::MultiValue| {
        Ok((Value::Nil, Value::Nil, Value::Boolean(false), Value::Integer(0), Value::Integer(0)))
    })?;
    t.set("GetCriteriaInfo", criteria_stub.clone())?;
    t.set("GetCriteriaInfoByStep", criteria_stub)?;
    t.set("IsInScenario", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_ScenarioInfo", t)?;
    Ok(())
}

fn register_c_tooltip_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    for (name, type_id) in &[
        ("GetItemByID", 1), ("GetItemByGUID", 1), ("GetBagItem", 1),
        ("GetSpellByID", 2), ("GetUnit", 3), ("GetHyperlink", 1),
    ] {
        let tid = *type_id;
        t.set(*name, lua.create_function(move |lua, _args: mlua::MultiValue| {
            let info = lua.create_table()?;
            info.set("type", tid)?;
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?)?;
    }
    // Catch-all for any C_TooltipInfo getter not explicitly stubbed above.
    // TooltipDataHandler maps ~50 Set* methods to Get* getters on this table;
    // return a stub that produces an empty tooltip data table.
    let mt = lua.create_table()?;
    mt.set("__index", lua.create_function(|lua, (_table, _key): (Value, String)| {
        let func = lua.create_function(move |lua, _args: mlua::MultiValue| {
            let info = lua.create_table()?;
            info.set("type", 0)?;
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?;
        Ok(Value::Function(func))
    })?)?;
    t.set_metatable(Some(mt));
    lua.globals().set("C_TooltipInfo", t)?;
    Ok(())
}

fn register_c_trade_skill(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;

    t.set("GetTradeSkillLine", lua.create_function(|_, ()| Ok((0i32, Value::Nil, 0i32, 0i32)))?)?;
    t.set("GetRecipeInfo", lua.create_function(|lua, _id: i32| {
        let info = lua.create_table()?;
        info.set("recipeID", 0)?;
        info.set("name", Value::Nil)?;
        info.set("craftable", false)?;
        Ok(info)
    })?)?;
    t.set("GetRecipeSchematic", lua.create_function(|lua, _id: i32| {
        let s = lua.create_table()?;
        s.set("recipeID", 0)?;
        Ok(s)
    })?)?;
    t.set("IsTradeSkillLinked", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsNPCCrafting", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetAllRecipeIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetProfessionSkillLineID", lua.create_function(|_, _p: Value| Ok(0i32))?)?;
    t.set("GetRecipesTracked", lua.create_function(|lua, _is_recraft: bool| lua.create_table())?)?;
    t.set("GetItemReagentQualityByItemInfo", lua.create_function(|_, _item: Value| Ok(Value::Nil))?)?;
    t.set("GetItemCraftedQualityByItemInfo", lua.create_function(|_, _item: Value| Ok(Value::Nil))?)?;

    lua.globals().set("C_TradeSkillUI", t)?;
    Ok(())
}

fn register_c_mythic_plus(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetRunHistory", lua.create_function(|lua, _args: mlua::MultiValue| lua.create_table())?)?;
    t.set("GetOwnedKeystoneLevel", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOwnedKeystoneChallengeMapID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOwnedKeystoneMapID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetCurrentAffixes", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetRewardLevelFromKeystoneLevel", lua.create_function(|_, _l: i32| Ok(0i32))?)?;
    t.set("GetWeeklyBestForMap", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetSeasonInfo", lua.create_function(|_, ()| Ok((1i32, 0i32, 0i32)))?)?;
    t.set("GetCurrentSeason", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("GetOverallDungeonScore", lua.create_function(|_, ()| Ok(0.0_f64))?)?;
    t.set("IsMythicPlusActive", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_MythicPlus", t)?;
    Ok(())
}

fn register_c_lfg_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetRoleCheckDifficultyDetails", lua.create_function(|_, ()| Ok((false, false, false)))?)?;
    t.set("GetDungeonInfo", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetLFDLockStates", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("CanPartyLFGBackfill", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetAllEntriesForCategory", lua.create_function(|lua, _cat: i32| lua.create_table())?)?;
    t.set("HideNameFromUI", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("CanPlayerUseLFD", lua.create_function(|_, ()| Ok((true, Value::Nil)))?)?;
    t.set("CanPlayerUseLFR", lua.create_function(|_, ()| Ok((true, Value::Nil)))?)?;
    t.set("CanPlayerUsePremadeGroup", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("CanPlayerUseGroupFinder", lua.create_function(|_, ()| Ok(true))?)?;
    lua.globals().set("C_LFGInfo", t)?;
    Ok(())
}

fn register_c_nameplate(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNamePlateForUnit", lua.create_function(|_, _u: String| Ok(Value::Nil))?)?;
    t.set("GetNamePlates", lua.create_function(|lua, _f: Option<bool>| lua.create_table())?)?;
    t.set("SetNamePlateEnemySize", lua.create_function(|_, (_w, _h): (f32, f32)| Ok(()))?)?;
    t.set("SetNamePlateFriendlySize", lua.create_function(|_, (_w, _h): (f32, f32)| Ok(()))?)?;
    t.set("SetNamePlateSelfSize", lua.create_function(|_, (_w, _h): (f32, f32)| Ok(()))?)?;
    t.set("GetNamePlateEnemySize", lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?)?;
    t.set("GetNamePlateFriendlySize", lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?)?;
    t.set("GetNamePlateSelfSize", lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?)?;
    t.set("SetNamePlateSelfClickThrough", lua.create_function(|_, _c: bool| Ok(()))?)?;
    t.set("SetNamePlateEnemyClickThrough", lua.create_function(|_, _c: bool| Ok(()))?)?;
    t.set("SetNamePlateFriendlyClickThrough", lua.create_function(|_, _c: bool| Ok(()))?)?;
    t.set("SetTargetClampingInsets", lua.create_function(|_, (_top, _bottom): (f64, f64)| Ok(()))?)?;
    lua.globals().set("C_NamePlate", t)?;
    Ok(())
}

fn register_c_player_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    register_c_player_info_mythic(lua, &t)?;
    register_c_player_info_misc(lua, &t)?;
    lua.globals().set("C_PlayerInfo", t)?;
    Ok(())
}

fn register_c_player_info_mythic(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetPlayerMythicPlusRatingSummary", lua.create_function(|lua, _u: String| {
        let s = lua.create_table()?;
        s.set("currentSeasonScore", 0.0_f64)?;
        s.set("runs", lua.create_table()?)?;
        Ok(s)
    })?)?;
    t.set("GetContentDifficultyCreatureForPlayer", lua.create_function(|_, _u: String| Ok(0i32))?)?;
    t.set("GetContentDifficultyQualityForPlayer", lua.create_function(|_, _u: String| Ok(0i32))?)?;
    Ok(())
}

fn register_c_player_info_misc(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("CanPlayerUseMountEquipment", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("IsPlayerNPERestricted", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetGlidingInfo", lua.create_function(|_, ()| Ok((false, false, 0.0_f64)))?)?;
    t.set("IsPlayerInChromieTime", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsTradingPostAvailable", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsTutorialsTabAvailable", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("CanPlayerUseEventScheduler", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsPlayerInRPE", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetDisplayID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetNativeDisplayID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetContentDifficultyQuestForPlayer", lua.create_function(|_, _id: i32| Ok(1i32))?)?;
    t.set("IsExpansionLandingPageUnlockedForPlayer", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

fn register_c_party_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    // Invite stubs
    t.set("GetActiveCategories", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetInviteConfirmationInfo", lua.create_function(|_, _g: String| Ok(Value::Nil))?)?;
    t.set("GetInviteReferralInfo", lua.create_function(|_, _g: String| Ok(Value::Nil))?)?;
    t.set("ConfirmInviteUnit", lua.create_function(|_, _g: String| Ok(()))?)?;
    t.set("DeclineInviteUnit", lua.create_function(|_, _g: String| Ok(()))?)?;
    t.set("IsPartyFull", lua.create_function(|_, _cat: Option<i32>| Ok(false))?)?;
    t.set("CanInvite", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("InviteUnit", lua.create_function(|_, _n: String| Ok(()))?)?;
    // Management stubs
    t.set("AllowedToDoPartyConversion", lua.create_function(|_, _r: bool| Ok(true))?)?;
    t.set("LeaveParty", lua.create_function(|_, _cat: Option<i32>| Ok(()))?)?;
    t.set("ConvertToParty", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("ConvertToRaid", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetMinLevel", lua.create_function(|_, _cat: Option<i32>| Ok(1i32))?)?;
    t.set("GetGatheringRequestInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetInstanceAbandonVoteTime", lua.create_function(|_, ()| Ok((0.0f64, 0.0f64)))?)?;
    t.set("IsPartyWalkIn", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsCrossFactionParty", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_PartyInfo", t)?;
    Ok(())
}

fn register_c_chat_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("RegisterAddonMessagePrefix", lua.create_function(|_, _p: String| Ok(true))?)?;
    t.set("IsAddonMessagePrefixRegistered", lua.create_function(|_, _p: String| Ok(false))?)?;
    t.set("SendAddonMessage", lua.create_function(
        |_, (_p, _m, _c, _t): (String, String, Option<String>, Option<String>)| Ok(()),
    )?)?;
    t.set("GetRegisteredAddonMessagePrefixes", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("SendChatMessage", lua.create_function(
        |lua, (msg, chat_type, _lang, _target): (String, Option<String>, Option<Value>, Option<String>)| {
            let chat_type = chat_type.unwrap_or_else(|| "SAY".to_string());
            let (r, g, b) = match chat_type.as_str() {
                "EMOTE" => ("1.0", "0.5", "0.25"),
                "YELL" => ("1.0", "0.25", "0.25"),
                "PARTY" => ("0.67", "0.67", "1.0"),
                "GUILD" => ("0.25", "1.0", "0.25"),
                "WHISPER" => ("1.0", "0.5", "1.0"),
                _ => ("1.0", "1.0", "1.0"),
            };
            lua.load(format!(
                r#"
                if ChatFrame1 and ChatFrame1.AddMessage then
                    local name = UnitName("player") or "Player"
                    local msg = ...
                    local prefix = ""
                    local fmt = GetCVar and GetCVar("showTimestamps")
                    if fmt and fmt ~= "" and fmt ~= "none" then
                        prefix = date(fmt, time())
                    end
                    ChatFrame1:AddMessage(
                        prefix .. "|Hplayer:" .. name .. "|h[" .. name .. "]|h says: " .. msg,
                        {r}, {g}, {b})
                end
                "#
            )).call::<()>(msg)?;
            Ok(())
        },
    )?)?;
    t.set("GetNumReservedChatWindows", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("GetNumActiveChannels", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("IsChannelRegionalForChannelID", lua.create_function(|_, _id: Value| Ok(false))?)?;
    t.set("GetChannelShortcutForChannelID", lua.create_function(|_, _id: Value| Ok(Value::Nil))?)?;
    lua.globals().set("C_ChatInfo", t)?;
    Ok(())
}

fn register_c_event_utils(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEventValid", lua.create_function(|_, event: String| {
        if event.len() < 3 { return Ok(false); }
        let chars: Vec<char> = event.chars().collect();
        if !chars[0].is_ascii_uppercase() { return Ok(false); }
        let has_underscore = event.contains('_');
        let all_valid = chars.iter().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_');
        Ok(has_underscore && all_valid)
    })?)?;
    lua.globals().set("C_EventUtils", t)?;
    Ok(())
}

fn register_c_azerite_essence(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetEssences", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetMilestones", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetEssenceInfo", lua.create_function(|lua, _id: i32| {
        let info = lua.create_table()?;
        info.set("ID", 0)?;
        info.set("name", "Unknown Essence")?;
        info.set("icon", 0)?;
        info.set("valid", false)?;
        info.set("unlocked", false)?;
        info.set("rank", 0)?;
        Ok(Value::Table(info))
    })?)?;
    t.set("GetMilestoneEssence", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetNumUnlockedEssences", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetNumUnlockedSlots", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("CanOpenUI", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_AzeriteEssence", t)?;
    Ok(())
}

fn register_c_pvp(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetZonePVPInfo", lua.create_function(|_, ()| Ok((Value::Nil, false, Value::Nil)))?)?;
    t.set("GetScoreInfo", lua.create_function(|_, _i: i32| Ok(Value::Nil))?)?;
    t.set("IsWarModeDesired", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsWarModeActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsPVPMap", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsRatedMap", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsInBrawl", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsActiveBattlefield", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetOutdoorPvPWaitTime", lua.create_function(|_, _map_id: Option<i32>| Ok(Value::Nil))?)?;
    t.set("IsMatchActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsMatchComplete", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActiveMatchState", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetArenaCrowdControlInfo", lua.create_function(|_, _unit: Value| {
        Ok((Value::Nil, Value::Nil, Value::Nil))
    })?)?;
    t.set("IsMatchConsideredArena", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("RequestCrowdControlSpell", lua.create_function(|_, _unit: Value| Ok(()))?)?;
    t.set("GetPvpTalentsUnlockedLevel", lua.create_function(|_, ()| Ok(10i32))?)?;
    t.set("GetWarModeRewardBonusDefault", lua.create_function(|_, ()| Ok(10i32))?)?;
    t.set("GetWarModeRewardBonus", lua.create_function(|_, ()| Ok(10i32))?)?;
    t.set("CanToggleWarMode", lua.create_function(|_, _desired: Value| Ok(false))?)?;
    t.set("IsWarModeDesired", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("CanToggleWarModeInArea", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_PvP", t)?;
    Ok(())
}

fn register_c_friend_list(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumFriends", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetNumOnlineFriends", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetFriendInfoByIndex", lua.create_function(|_, _i: i32| Ok(Value::Nil))?)?;
    t.set("GetFriendInfoByName", lua.create_function(|_, _n: String| Ok(Value::Nil))?)?;
    t.set("IsFriend", lua.create_function(|_, _g: String| Ok(false))?)?;
    lua.globals().set("C_FriendList", t)?;
    Ok(())
}

fn register_c_auction_house(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumReplicateItems", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_AuctionHouse", t)?;
    Ok(())
}

fn register_c_bank(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("FetchDepositedMoney", lua.create_function(|_, _bt: i32| Ok(0i64))?)?;
    lua.globals().set("C_Bank", t)?;
    Ok(())
}

fn register_c_encounter_journal(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetEncounterInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetSectionInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetLootInfoByIndex", lua.create_function(|_, _i: i32| Ok(Value::Nil))?)?;
    t.set("GetInstanceInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    lua.globals().set("C_EncounterJournal", t)?;
    Ok(())
}

fn register_c_gm_ticket_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("HasGMTicket", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_GMTicketInfo", t)?;
    Ok(())
}

fn register_c_unit_auras(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetAuraDataByIndex", lua.create_function(|_, (_u, _i, _f): (String, i32, Option<String>)| Ok(Value::Nil))?)?;
    t.set("GetAuraDataByAuraInstanceID", lua.create_function(|_, (_u, _id): (String, i32)| Ok(Value::Nil))?)?;
    t.set("GetAuraDataBySlot", lua.create_function(|_, (_u, _s): (String, i32)| Ok(Value::Nil))?)?;
    t.set("GetBuffDataByIndex", lua.create_function(|_, (_u, _i, _f): (String, i32, Option<String>)| Ok(Value::Nil))?)?;
    t.set("GetDebuffDataByIndex", lua.create_function(|_, (_u, _i, _f): (String, i32, Option<String>)| Ok(Value::Nil))?)?;
    t.set("GetPlayerAuraBySpellID", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetCooldownAuraBySpellID", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("IsAuraFilteredOutByInstanceID", lua.create_function(|_, (_u, _id, _f): (String, i32, String)| Ok(false))?)?;
    t.set("WantsAlteredForm", lua.create_function(|_, _u: String| Ok(false))?)?;
    t.set("AddPrivateAuraAnchor", lua.create_function(|_, _a: mlua::MultiValue| Ok(0i32))?)?;
    t.set("RemovePrivateAuraAnchor", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("AddPrivateAuraAppliedSound", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("RemovePrivateAuraAppliedSound", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("SetPrivateWarningTextAnchor", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    lua.globals().set("C_UnitAuras", t)?;
    Ok(())
}

fn register_c_currency_info(lua: &Lua) -> Result<()> {
    use super::currency_data;
    let t = lua.create_table()?;
    t.set("GetCurrencyInfo", lua.create_function(currency_info_by_id)?)?;
    t.set("GetBasicCurrencyInfo", lua.create_function(|lua, (cid, _qty): (i32, Option<i32>)| {
        let info = lua.create_table()?;
        if let Some(c) = currency_data::get_currency_by_id(cid) {
            info.set("name", c.name)?;
            info.set("currencyID", c.currency_id)?;
            info.set("quantity", c.quantity)?;
            info.set("iconFileID", c.icon_file_id as i64)?;
            info.set("displayAmount", c.quantity)?;
        } else {
            info.set("name", format!("Currency {}", cid))?;
            info.set("currencyID", cid)?;
            info.set("quantity", 0)?;
            info.set("iconFileID", 0)?;
            info.set("displayAmount", 0)?;
        }
        Ok(Value::Table(info))
    })?)?;
    t.set("GetCurrencyInfoFromLink", lua.create_function(|_, _l: String| Ok(Value::Nil))?)?;
    t.set("GetCurrencyListSize", lua.create_function(|_, ()| Ok(currency_data::currency_list_size()))?)?;
    t.set("GetCurrencyListInfo", lua.create_function(currency_list_info)?)?;
    t.set("GetBackpackCurrencyInfo", lua.create_function(backpack_currency_info)?)?;
    t.set("ExpandCurrencyList", lua.create_function(|_, (_i, _e): (i32, bool)| Ok(()))?)?;
    t.set("GetCurrencyFilter", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("SetCurrencyFilter", lua.create_function(|_, _f: i32| Ok(()))?)?;
    t.set("SetCurrencyBackpack", lua.create_function(|_, (_i, _w): (i32, bool)| Ok(()))?)?;
    t.set("SetCurrencyUnused", lua.create_function(|_, (_i, _u): (i32, bool)| Ok(()))?)?;
    t.set("DoesCurrentFilterRequireAccountCurrencyData", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsAccountCharacterCurrencyDataReady", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("GetWarResourcesCurrencyID", lua.create_function(|_, ()| Ok(1560))?)?;
    t.set("GetAzeriteCurrencyID", lua.create_function(|_, ()| Ok(1553))?)?;
    lua.globals().set("C_CurrencyInfo", t)?;
    Ok(())
}

fn currency_info_by_id(lua: &Lua, cid: i32) -> Result<Value> {
    use super::currency_data;
    let info = lua.create_table()?;
    if let Some(c) = currency_data::get_currency_by_id(cid) {
        info.set("name", c.name)?;
        info.set("currencyID", c.currency_id)?;
        info.set("quantity", c.quantity)?;
        info.set("maxQuantity", c.max_quantity)?;
        info.set("quality", c.quality)?;
        info.set("iconFileID", c.icon_file_id as i64)?;
        info.set("discovered", c.is_discovered)?;
    } else {
        info.set("name", format!("Currency {}", cid))?;
        info.set("currencyID", cid)?;
        info.set("quantity", 0)?;
        info.set("maxQuantity", 0)?;
        info.set("quality", 1)?;
        info.set("iconFileID", 0)?;
        info.set("discovered", false)?;
    }
    info.set("isAccountWide", false)?;
    info.set("isAccountTransferable", false)?;
    info.set("transferPercentage", 0)?;
    Ok(Value::Table(info))
}

fn currency_list_info(lua: &Lua, index: i32) -> Result<Value> {
    use super::currency_data;
    let Some(c) = currency_data::get_currency_list_entry(index) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("name", c.name)?;
    info.set("currencyID", c.currency_id)?;
    info.set("quantity", c.quantity)?;
    info.set("maxQuantity", c.max_quantity)?;
    info.set("quality", c.quality)?;
    info.set("iconFileID", c.icon_file_id as i64)?;
    info.set("discovered", c.is_discovered)?;
    info.set("isHeader", c.is_header)?;
    info.set("isHeaderExpanded", c.is_header_expanded)?;
    info.set("currencyListDepth", c.depth)?;
    info.set("isTypeUnused", false)?;
    info.set("isShowInBackpack", c.is_show_in_backpack)?;
    info.set("isAccountWide", false)?;
    info.set("isAccountTransferable", false)?;
    info.set("transferPercentage", 0)?;
    Ok(Value::Table(info))
}

fn backpack_currency_info(lua: &Lua, index: i32) -> Result<Value> {
    use super::currency_data;
    let watched: Vec<_> = currency_data::backpack_currencies().collect();
    let Some(c) = watched.get((index - 1) as usize) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("name", c.name)?;
    info.set("quantity", c.quantity)?;
    info.set("iconFileID", c.icon_file_id as i64)?;
    info.set("currencyTypesID", c.currency_id)?;
    Ok(Value::Table(info))
}
