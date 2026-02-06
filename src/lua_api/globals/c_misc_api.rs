//! Miscellaneous C_* namespace APIs.
//!
//! Contains various WoW API namespaces that are too small to warrant their own modules:
//! - C_ScenarioInfo - scenario/dungeon info
//! - C_TooltipInfo - tooltip data
//! - C_PetBattles - pet battle system
//! - C_TradeSkillUI - profession/tradeskill UI
//! - C_MythicPlus - Mythic+ dungeon info
//! - C_LFGInfo - Looking for Group information
//! - C_NamePlate - Nameplate management
//! - C_PlayerInfo - Player information
//! - C_PartyInfo - Party/group information
//! - C_ChatInfo - Chat system
//! - C_EventUtils - Event utilities
//! - C_AzeriteEssence - BfA Azerite system
//! - C_PvP - PvP information
//! - C_FriendList - Friend list
//! - C_AuctionHouse - Auction house
//! - C_Bank - Personal bank
//! - C_EncounterJournal - Dungeon/raid journal
//! - C_GMTicketInfo - GM ticket system
//! - C_UnitAuras - Unit aura information
//! - C_CurrencyInfo - Currency information
//! - C_VignetteInfo - Map vignette info
//! - C_AreaPoiInfo - Area point of interest info
//! - C_PlayerChoice - Player choice system
//! - C_MajorFactions - Major Factions/Renown system
//! - C_UIWidgetManager - UI widgets
//! - C_GossipInfo - NPC gossip/dialog
//! - C_Calendar - In-game calendar
//! - C_CovenantCallings - Shadowlands covenant callings
//! - C_WeeklyRewards - Great Vault rewards
//! - C_ContributionCollector - Warfront contributions
//! - C_Scenario - Scenario system
//! - C_HousingCustomizeMode - Housing decoration
//! - C_DyeColor - Dye color information
//! - C_GameRules - Game rules
//! - C_ScriptedAnimations - Scripted animation effects
//! - C_Glue - Glue screen utilities
//! - C_UIColor - Color utilities
//! - C_ClassColor - Class colors

use mlua::{Lua, Result, Value};

/// Register all miscellaneous C_* namespace APIs.
pub fn register_c_misc_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    register_c_scenario_info(lua)?;
    register_c_tooltip_info(lua)?;
    register_c_pet_battles(lua)?;
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
    register_c_vignette_info(lua)?;
    register_c_area_poi(lua)?;
    register_c_player_choice(lua)?;
    register_c_major_factions(lua)?;
    register_c_ui_widget(lua)?;
    register_c_gossip_info(lua)?;
    register_c_calendar(lua)?;
    register_c_covenant_callings(lua)?;
    register_c_weekly_rewards(lua)?;
    register_c_contribution_collector(lua)?;
    register_c_scenario(lua)?;
    register_c_housing(lua)?;
    register_c_game_rules(lua)?;
    register_c_scripted_animations(lua)?;
    register_c_glue(lua)?;
    register_c_ui_color(lua)?;
    register_c_class_color(lua)?;
    register_c_spec_info(lua)?;
    register_c_artifact_ui(lua)?;
    register_c_super_track(lua)?;
    register_c_player_interaction_manager(lua)?;
    register_c_paper_doll_info(lua)?;
    register_c_perks_program(lua)?;
    register_tooltip_data_processor(lua)?;
    Ok(())
}

fn register_tooltip_data_processor(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "AddTooltipPostCall",
        lua.create_function(|_, (_data_type, _callback): (Option<i32>, mlua::Function)| Ok(()))?,
    )?;
    lua.globals().set("TooltipDataProcessor", t)?;
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
    // All tooltip getters return {type=N, lines={}} stub
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
    lua.globals().set("C_TooltipInfo", t)?;
    Ok(())
}

fn register_c_pet_battles(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_pet_battles = lua.create_table()?;

    c_pet_battles.set("IsInBattle", lua.create_function(|_, ()| Ok(false))?)?;
    c_pet_battles.set("IsWildBattle", lua.create_function(|_, ()| Ok(false))?)?;
    c_pet_battles.set(
        "IsPlayerNPC",
        lua.create_function(|_, _owner_index: i32| Ok(false))?,
    )?;
    c_pet_battles.set(
        "GetNumAuras",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetActivePet",
        lua.create_function(|_, _owner: i32| Ok(1i32))?,
    )?;
    c_pet_battles.set(
        "GetNumPets",
        lua.create_function(|_, _owner: i32| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetHealth",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetMaxHealth",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(100i32))?,
    )?;
    c_pet_battles.set(
        "GetSpeed",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetPower",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;

    c_pet_battles.set(
        "GetAllEffectNames",
        lua.create_function(|_, ()| Ok(mlua::MultiValue::new()))?,
    )?;
    c_pet_battles.set(
        "GetAllStates",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;

    globals.set("C_PetBattles", c_pet_battles)?;
    Ok(())
}

fn register_c_trade_skill(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_trade_skill = lua.create_table()?;

    c_trade_skill.set(
        "GetTradeSkillLine",
        lua.create_function(|_, ()| Ok((0i32, Value::Nil, 0i32, 0i32)))?,
    )?;
    c_trade_skill.set(
        "GetRecipeInfo",
        lua.create_function(|lua, _recipe_id: i32| {
            let info = lua.create_table()?;
            info.set("recipeID", 0)?;
            info.set("name", Value::Nil)?;
            info.set("craftable", false)?;
            Ok(info)
        })?,
    )?;
    c_trade_skill.set(
        "GetRecipeSchematic",
        lua.create_function(|lua, _recipe_id: i32| {
            let schematic = lua.create_table()?;
            schematic.set("recipeID", 0)?;
            Ok(schematic)
        })?,
    )?;
    c_trade_skill.set(
        "IsTradeSkillLinked",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_trade_skill.set("IsNPCCrafting", lua.create_function(|_, ()| Ok(false))?)?;
    c_trade_skill.set(
        "GetAllRecipeIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_trade_skill.set(
        "GetProfessionSkillLineID",
        lua.create_function(|_, _profession: Value| Ok(0i32))?,
    )?;

    globals.set("C_TradeSkillUI", c_trade_skill)?;
    Ok(())
}

fn register_c_mythic_plus(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    register_mythic_plus_keystone(lua, &t)?;
    register_mythic_plus_season(lua, &t)?;
    lua.globals().set("C_MythicPlus", t)?;
    Ok(())
}

fn register_mythic_plus_keystone(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetRunHistory", lua.create_function(
        |lua, _args: mlua::MultiValue| lua.create_table(),
    )?)?;
    t.set("GetOwnedKeystoneLevel", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOwnedKeystoneChallengeMapID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOwnedKeystoneMapID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetCurrentAffixes", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetRewardLevelFromKeystoneLevel", lua.create_function(|_, _level: i32| Ok(0i32))?)?;
    t.set("GetWeeklyBestForMap", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    Ok(())
}

fn register_mythic_plus_season(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetSeasonInfo", lua.create_function(|_, ()| Ok((1i32, 0i32, 0i32)))?)?;
    t.set("GetCurrentSeason", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("GetOverallDungeonScore", lua.create_function(|_, ()| Ok(0.0_f64))?)?;
    t.set("IsMythicPlusActive", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

fn register_c_lfg_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_lfg_info = lua.create_table()?;

    c_lfg_info.set(
        "GetRoleCheckDifficultyDetails",
        lua.create_function(|_, ()| Ok((false, false, false)))?,
    )?;
    c_lfg_info.set(
        "GetDungeonInfo",
        lua.create_function(|lua, _dungeon_id: i32| lua.create_table())?,
    )?;
    c_lfg_info.set(
        "GetLFDLockStates",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_lfg_info.set(
        "CanPartyLFGBackfill",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_lfg_info.set(
        "GetAllEntriesForCategory",
        lua.create_function(|lua, _category: i32| lua.create_table())?,
    )?;
    c_lfg_info.set(
        "HideNameFromUI",
        lua.create_function(|_, _dungeon_id: i32| Ok(false))?,
    )?;

    globals.set("C_LFGInfo", c_lfg_info)?;
    Ok(())
}

fn register_c_nameplate(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_nameplate = lua.create_table()?;

    c_nameplate.set(
        "GetNamePlateForUnit",
        lua.create_function(|_, _unit: String| Ok(Value::Nil))?,
    )?;
    c_nameplate.set(
        "GetNamePlates",
        lua.create_function(|lua, _include_forbidden: Option<bool>| lua.create_table())?,
    )?;
    c_nameplate.set(
        "SetNamePlateEnemySize",
        lua.create_function(|_, (_width, _height): (f32, f32)| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateFriendlySize",
        lua.create_function(|_, (_width, _height): (f32, f32)| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateSelfSize",
        lua.create_function(|_, (_width, _height): (f32, f32)| Ok(()))?,
    )?;
    c_nameplate.set(
        "GetNamePlateEnemySize",
        lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?,
    )?;
    c_nameplate.set(
        "GetNamePlateFriendlySize",
        lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?,
    )?;
    c_nameplate.set(
        "GetNamePlateSelfSize",
        lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?,
    )?;
    c_nameplate.set(
        "SetNamePlateSelfClickThrough",
        lua.create_function(|_, _click_through: bool| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateEnemyClickThrough",
        lua.create_function(|_, _click_through: bool| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateFriendlyClickThrough",
        lua.create_function(|_, _click_through: bool| Ok(()))?,
    )?;

    globals.set("C_NamePlate", c_nameplate)?;
    Ok(())
}

fn register_c_player_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_player_info = lua.create_table()?;

    c_player_info.set(
        "GetPlayerMythicPlusRatingSummary",
        lua.create_function(|lua, _unit: String| {
            let summary = lua.create_table()?;
            summary.set("currentSeasonScore", 0.0_f64)?;
            summary.set("runs", lua.create_table()?)?;
            Ok(summary)
        })?,
    )?;
    c_player_info.set(
        "GetContentDifficultyCreatureForPlayer",
        lua.create_function(|_, _unit: String| Ok(0i32))?,
    )?;
    c_player_info.set(
        "GetContentDifficultyQualityForPlayer",
        lua.create_function(|_, _unit: String| Ok(0i32))?,
    )?;
    c_player_info.set(
        "CanPlayerUseMountEquipment",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    c_player_info.set(
        "IsPlayerNPERestricted",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_player_info.set(
        "GetGlidingInfo",
        lua.create_function(|_, ()| Ok((false, false, 0.0_f64)))?,
    )?;
    c_player_info.set(
        "IsPlayerInChromieTime",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_player_info.set(
        "IsTradingPostAvailable",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_player_info.set(
        "IsTutorialsTabAvailable",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_PlayerInfo", c_player_info)?;
    Ok(())
}

fn register_c_party_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    register_party_invite_stubs(lua, &t)?;
    register_party_management_stubs(lua, &t)?;
    lua.globals().set("C_PartyInfo", t)?;
    Ok(())
}

fn register_party_invite_stubs(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetActiveCategories", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetInviteConfirmationInfo", lua.create_function(|_, _g: String| Ok(Value::Nil))?)?;
    t.set("GetInviteReferralInfo", lua.create_function(|_, _g: String| Ok(Value::Nil))?)?;
    t.set("ConfirmInviteUnit", lua.create_function(|_, _g: String| Ok(()))?)?;
    t.set("DeclineInviteUnit", lua.create_function(|_, _g: String| Ok(()))?)?;
    t.set("IsPartyFull", lua.create_function(|_, _cat: Option<i32>| Ok(false))?)?;
    t.set("CanInvite", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("InviteUnit", lua.create_function(|_, _name: String| Ok(()))?)?;
    Ok(())
}

fn register_party_management_stubs(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("AllowedToDoPartyConversion", lua.create_function(|_, _r: bool| Ok(true))?)?;
    t.set("LeaveParty", lua.create_function(|_, _cat: Option<i32>| Ok(()))?)?;
    t.set("ConvertToParty", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("ConvertToRaid", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetMinLevel", lua.create_function(|_, _cat: Option<i32>| Ok(1i32))?)?;
    t.set("GetGatheringRequestInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    Ok(())
}

fn register_c_chat_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_chat_info = lua.create_table()?;

    c_chat_info.set(
        "RegisterAddonMessagePrefix",
        lua.create_function(|_, _prefix: String| Ok(true))?,
    )?;
    c_chat_info.set(
        "IsAddonMessagePrefixRegistered",
        lua.create_function(|_, _prefix: String| Ok(false))?,
    )?;
    c_chat_info.set(
        "SendAddonMessage",
        lua.create_function(
            |_,
             (_prefix, _message, _channel, _target): (
                String,
                String,
                Option<String>,
                Option<String>,
            )| { Ok(()) },
        )?,
    )?;
    c_chat_info.set(
        "GetRegisteredAddonMessagePrefixes",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_chat_info.set(
        "SendChatMessage",
        lua.create_function(
            |_,
             (_message, _channel, _language, _target): (
                String,
                Option<String>,
                Option<Value>,
                Option<String>,
            )| { Ok(()) },
        )?,
    )?;

    globals.set("C_ChatInfo", c_chat_info)?;
    Ok(())
}

fn register_c_event_utils(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_event_utils = lua.create_table()?;

    c_event_utils.set(
        "IsEventValid",
        lua.create_function(|_, event: String| {
            if event.len() < 3 {
                return Ok(false);
            }
            let chars: Vec<char> = event.chars().collect();
            if !chars[0].is_ascii_uppercase() {
                return Ok(false);
            }
            let has_underscore = event.contains('_');
            let all_valid = chars
                .iter()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_');
            Ok(has_underscore && all_valid)
        })?,
    )?;

    globals.set("C_EventUtils", c_event_utils)?;
    Ok(())
}

fn register_c_azerite_essence(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_azerite_essence = lua.create_table()?;

    c_azerite_essence.set(
        "GetEssences",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_azerite_essence.set(
        "GetMilestones",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_azerite_essence.set(
        "GetEssenceInfo",
        lua.create_function(|lua, _essence_id: i32| {
            let info = lua.create_table()?;
            info.set("ID", 0)?;
            info.set("name", "Unknown Essence")?;
            info.set("icon", 0)?;
            info.set("valid", false)?;
            info.set("unlocked", false)?;
            info.set("rank", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_azerite_essence.set(
        "GetMilestoneEssence",
        lua.create_function(|_, _milestone_id: i32| Ok(Value::Nil))?,
    )?;
    c_azerite_essence.set(
        "GetNumUnlockedEssences",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_azerite_essence.set(
        "GetNumUnlockedSlots",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_azerite_essence.set("CanOpenUI", lua.create_function(|_, ()| Ok(false))?)?;

    globals.set("C_AzeriteEssence", c_azerite_essence)?;
    Ok(())
}

fn register_c_pvp(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_pvp = lua.create_table()?;

    c_pvp.set(
        "GetZonePVPInfo",
        lua.create_function(|_, ()| Ok((Value::Nil, false, Value::Nil)))?,
    )?;
    c_pvp.set(
        "GetScoreInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_pvp.set("IsWarModeDesired", lua.create_function(|_, ()| Ok(false))?)?;
    c_pvp.set("IsWarModeActive", lua.create_function(|_, ()| Ok(false))?)?;
    c_pvp.set("IsPVPMap", lua.create_function(|_, ()| Ok(false))?)?;
    c_pvp.set("IsRatedMap", lua.create_function(|_, ()| Ok(false))?)?;
    c_pvp.set("IsInBrawl", lua.create_function(|_, ()| Ok(false))?)?;

    globals.set("C_PvP", c_pvp)?;
    Ok(())
}

fn register_c_friend_list(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_friend_list = lua.create_table()?;

    c_friend_list.set("GetNumFriends", lua.create_function(|_, ()| Ok(0i32))?)?;
    c_friend_list.set(
        "GetNumOnlineFriends",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_friend_list.set(
        "GetFriendInfoByIndex",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_friend_list.set(
        "GetFriendInfoByName",
        lua.create_function(|_, _name: String| Ok(Value::Nil))?,
    )?;
    c_friend_list.set(
        "IsFriend",
        lua.create_function(|_, _guid: String| Ok(false))?,
    )?;

    globals.set("C_FriendList", c_friend_list)?;
    Ok(())
}

fn register_c_auction_house(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_auction_house = lua.create_table()?;

    c_auction_house.set(
        "GetNumReplicateItems",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;

    globals.set("C_AuctionHouse", c_auction_house)?;
    Ok(())
}

fn register_c_bank(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_bank = lua.create_table()?;

    c_bank.set(
        "FetchDepositedMoney",
        lua.create_function(|_, _bank_type: i32| Ok(0i64))?,
    )?;

    globals.set("C_Bank", c_bank)?;
    Ok(())
}

fn register_c_encounter_journal(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_encounter_journal = lua.create_table()?;

    c_encounter_journal.set(
        "GetEncounterInfo",
        lua.create_function(|_, _encounter_id: i32| Ok(Value::Nil))?,
    )?;
    c_encounter_journal.set(
        "GetSectionInfo",
        lua.create_function(|_, _section_id: i32| Ok(Value::Nil))?,
    )?;
    c_encounter_journal.set(
        "GetLootInfoByIndex",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_encounter_journal.set(
        "GetInstanceInfo",
        lua.create_function(|_, _instance_id: i32| Ok(Value::Nil))?,
    )?;

    globals.set("C_EncounterJournal", c_encounter_journal)?;
    Ok(())
}

fn register_c_gm_ticket_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_gm_ticket_info = lua.create_table()?;

    c_gm_ticket_info.set("HasGMTicket", lua.create_function(|_, ()| Ok(false))?)?;

    globals.set("C_GMTicketInfo", c_gm_ticket_info)?;
    Ok(())
}

fn register_c_unit_auras(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let t = lua.create_table()?;

    // Aura data queries
    t.set("GetAuraDataByIndex", lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil))?)?;
    t.set("GetAuraDataByAuraInstanceID", lua.create_function(|_, (_unit, _id): (String, i32)| Ok(Value::Nil))?)?;
    t.set("GetAuraDataBySlot", lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?)?;
    t.set("GetBuffDataByIndex", lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil))?)?;
    t.set("GetDebuffDataByIndex", lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| Ok(Value::Nil))?)?;
    t.set("GetPlayerAuraBySpellID", lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?)?;
    t.set("GetCooldownAuraBySpellID", lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?)?;
    t.set("IsAuraFilteredOutByInstanceID", lua.create_function(|_, (_unit, _id, _filter): (String, i32, String)| Ok(false))?)?;
    t.set("WantsAlteredForm", lua.create_function(|_, _unit: String| Ok(false))?)?;

    // Private aura management
    t.set("AddPrivateAuraAnchor", lua.create_function(|_, _args: mlua::MultiValue| Ok(0i32))?)?;
    t.set("RemovePrivateAuraAnchor", lua.create_function(|_, _anchor_id: i32| Ok(()))?)?;
    t.set("AddPrivateAuraAppliedSound", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    t.set("RemovePrivateAuraAppliedSound", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    t.set("SetPrivateWarningTextAnchor", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;

    globals.set("C_UnitAuras", t)?;
    Ok(())
}

fn register_c_currency_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    register_currency_lookup_stubs(lua, &t)?;
    register_currency_list_stubs(lua, &t)?;
    lua.globals().set("C_CurrencyInfo", t)?;
    Ok(())
}

fn register_currency_lookup_stubs(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetCurrencyInfo", lua.create_function(|lua, currency_id: i32| {
        let info = lua.create_table()?;
        info.set("name", format!("Currency {}", currency_id))?;
        info.set("currencyID", currency_id)?;
        info.set("quantity", 0)?;
        info.set("maxQuantity", 0)?;
        info.set("quality", 1)?;
        info.set("iconFileID", 0)?;
        info.set("discovered", false)?;
        info.set("isAccountWide", false)?;
        info.set("isAccountTransferable", false)?;
        info.set("transferPercentage", 0)?;
        Ok(Value::Table(info))
    })?)?;
    t.set("GetBasicCurrencyInfo", lua.create_function(|lua, (cid, _qty): (i32, Option<i32>)| {
        let info = lua.create_table()?;
        info.set("name", format!("Currency {}", cid))?;
        info.set("currencyID", cid)?;
        info.set("quantity", 0)?;
        info.set("iconFileID", 0)?;
        info.set("displayAmount", 0)?;
        Ok(Value::Table(info))
    })?)?;
    t.set("GetCurrencyInfoFromLink", lua.create_function(|_, _link: String| Ok(Value::Nil))?)?;
    Ok(())
}

fn register_currency_list_stubs(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetCurrencyListSize", lua.create_function(|_, ()| Ok(0))?)?;
    t.set("GetCurrencyListInfo", lua.create_function(|_, _idx: i32| Ok(Value::Nil))?)?;
    t.set("GetWarResourcesCurrencyID", lua.create_function(|_, ()| Ok(1560))?)?;
    t.set("GetAzeriteCurrencyID", lua.create_function(|_, ()| Ok(1553))?)?;
    Ok(())
}

fn register_c_vignette_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_vignette_info = lua.create_table()?;

    c_vignette_info.set(
        "GetVignettes",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_vignette_info.set(
        "GetVignetteInfo",
        lua.create_function(|_, _vignette_guid: String| Ok(Value::Nil))?,
    )?;
    c_vignette_info.set(
        "GetVignettePosition",
        lua.create_function(|_, (_vignette_guid, _ui_map_id): (String, Option<i32>)| {
            Ok(Value::Nil)
        })?,
    )?;
    c_vignette_info.set(
        "GetVignetteGUID",
        lua.create_function(|_, _object_guid: String| Ok(Value::Nil))?,
    )?;

    globals.set("C_VignetteInfo", c_vignette_info)?;
    Ok(())
}

fn register_c_area_poi(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_area_poi = lua.create_table()?;

    c_area_poi.set(
        "GetAreaPOIInfo",
        lua.create_function(|_, (_ui_map_id, _area_poi_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_area_poi.set(
        "GetAreaPOISecondsLeft",
        lua.create_function(|_, _area_poi_id: i32| Ok(0i32))?,
    )?;
    c_area_poi.set(
        "IsAreaPOITimed",
        lua.create_function(|_, _area_poi_id: i32| Ok(false))?,
    )?;
    c_area_poi.set(
        "GetAreaPOIForMap",
        lua.create_function(|lua, _ui_map_id: i32| lua.create_table())?,
    )?;

    globals.set("C_AreaPoiInfo", c_area_poi)?;
    Ok(())
}

fn register_c_player_choice(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_player_choice = lua.create_table()?;

    c_player_choice.set(
        "GetCurrentPlayerChoiceInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_player_choice.set(
        "GetNumPlayerChoices",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_player_choice.set(
        "GetPlayerChoiceInfo",
        lua.create_function(|_, _choice_id: i32| Ok(Value::Nil))?,
    )?;
    c_player_choice.set(
        "GetPlayerChoiceOptionInfo",
        lua.create_function(|_, (_choice_id, _option_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_player_choice.set(
        "SendPlayerChoiceResponse",
        lua.create_function(|_, _response_id: i32| Ok(()))?,
    )?;
    c_player_choice.set(
        "IsWaitingForPlayerChoiceResponse",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_PlayerChoice", c_player_choice)?;
    Ok(())
}

fn register_c_major_factions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_major_factions = lua.create_table()?;

    c_major_factions.set(
        "GetMajorFactionData",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_major_factions.set(
        "GetMajorFactionIDs",
        lua.create_function(|lua, _expansion_id: Option<i32>| lua.create_table())?,
    )?;
    c_major_factions.set(
        "GetRenownLevels",
        lua.create_function(|lua, _faction_id: i32| lua.create_table())?,
    )?;
    c_major_factions.set(
        "GetCurrentRenownLevel",
        lua.create_function(|_, _faction_id: i32| Ok(0i32))?,
    )?;
    c_major_factions.set(
        "HasMaximumRenown",
        lua.create_function(|_, _faction_id: i32| Ok(false))?,
    )?;
    c_major_factions.set(
        "GetRenownRewardsForLevel",
        lua.create_function(|lua, (_faction_id, _renown_level): (i32, i32)| lua.create_table())?,
    )?;

    globals.set("C_MajorFactions", c_major_factions)?;
    Ok(())
}

fn register_c_ui_widget(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_ui_widget = lua.create_table()?;

    c_ui_widget.set(
        "GetAllWidgetsBySetID",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    c_ui_widget.set(
        "GetStatusBarWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetTextWithStateWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetIconAndTextWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetCaptureBarWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetDoubleStatusBarWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetSpellDisplayVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetWidgetSetInfo",
        lua.create_function(|_, _set_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetTopCenterWidgetSetID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_ui_widget.set(
        "GetBelowMinimapWidgetSetID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_ui_widget.set(
        "GetObjectiveTrackerWidgetSetID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;

    globals.set("C_UIWidgetManager", c_ui_widget)?;
    Ok(())
}

fn register_c_gossip_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let t = lua.create_table()?;

    // Core gossip functions
    t.set("GetNumOptions", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOptions", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetText", lua.create_function(|_, ()| Ok(""))?)?;
    t.set("SelectOption", lua.create_function(|_, (_id, _text, _confirmed): (i32, Option<String>, Option<bool>)| Ok(()))?)?;
    t.set("CloseGossip", lua.create_function(|_, ()| Ok(()))?)?;

    // Quest-related gossip
    t.set("GetNumActiveQuests", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetNumAvailableQuests", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetActiveQuests", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetAvailableQuests", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("SelectActiveQuest", lua.create_function(|_, _index: i32| Ok(()))?)?;
    t.set("SelectAvailableQuest", lua.create_function(|_, _index: i32| Ok(()))?)?;

    // Friendship reputation
    register_gossip_friendship_methods(lua, &t)?;

    t.set("ForceGossip", lua.create_function(|_, ()| Ok(false))?)?;

    globals.set("C_GossipInfo", t)?;
    Ok(())
}

fn register_gossip_friendship_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set(
        "GetFriendshipReputation",
        lua.create_function(|lua, _faction_id: Option<i32>| {
            let info = lua.create_table()?;
            info.set("friendshipFactionID", 0)?;
            info.set("standing", 0)?;
            info.set("maxRep", 0)?;
            info.set("name", Value::Nil)?;
            info.set("text", Value::Nil)?;
            info.set("texture", Value::Nil)?;
            info.set("reaction", Value::Nil)?;
            info.set("reactionThreshold", 0)?;
            info.set("nextThreshold", Value::Nil)?;
            Ok(info)
        })?,
    )?;
    t.set(
        "GetFriendshipReputationRanks",
        lua.create_function(|lua, _faction_id: Option<i32>| {
            let info = lua.create_table()?;
            info.set("currentLevel", 0)?;
            info.set("maxLevel", 0)?;
            Ok(info)
        })?,
    )?;
    Ok(())
}

fn register_c_calendar(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_calendar = lua.create_table()?;

    c_calendar.set(
        "GetDate",
        lua.create_function(|_, ()| Ok((1i32, 1i32, 1i32, 2024i32)))?,
    )?;
    c_calendar.set(
        "GetMonthInfo",
        lua.create_function(|lua, _offset: Option<i32>| {
            let info = lua.create_table()?;
            info.set("month", 1)?;
            info.set("year", 2024)?;
            info.set("numDays", 31)?;
            info.set("firstWeekday", 1)?;
            Ok(info)
        })?,
    )?;
    c_calendar.set(
        "GetNumDayEvents",
        lua.create_function(|_, (_offset, _day): (i32, i32)| Ok(0i32))?,
    )?;
    c_calendar.set(
        "GetDayEvent",
        lua.create_function(|_, (_offset, _day, _index): (i32, i32, i32)| Ok(Value::Nil))?,
    )?;
    c_calendar.set("OpenCalendar", lua.create_function(|_, ()| Ok(()))?)?;
    c_calendar.set("CloseCalendar", lua.create_function(|_, ()| Ok(()))?)?;
    c_calendar.set("SetMonth", lua.create_function(|_, _offset: i32| Ok(()))?)?;
    c_calendar.set(
        "SetAbsMonth",
        lua.create_function(|_, (_month, _year): (i32, i32)| Ok(()))?,
    )?;
    c_calendar.set(
        "GetMinDate",
        lua.create_function(|_, ()| Ok((1i32, 1i32, 2004i32)))?,
    )?;
    c_calendar.set(
        "GetMaxDate",
        lua.create_function(|_, ()| Ok((12i32, 31i32, 2030i32)))?,
    )?;

    globals.set("C_Calendar", c_calendar)?;
    Ok(())
}

fn register_c_covenant_callings(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_covenant_callings = lua.create_table()?;

    c_covenant_callings.set(
        "AreCallingsUnlocked",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_covenant_callings.set(
        "RequestCallings",
        lua.create_function(|_, ()| Ok(()))?,
    )?;

    globals.set("C_CovenantCallings", c_covenant_callings)?;
    Ok(())
}

fn register_c_weekly_rewards(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_weekly_rewards = lua.create_table()?;

    c_weekly_rewards.set(
        "HasAvailableRewards",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_weekly_rewards.set("CanClaimRewards", lua.create_function(|_, ()| Ok(false))?)?;
    c_weekly_rewards.set(
        "GetActivities",
        lua.create_function(|lua, _type: Option<i32>| lua.create_table())?,
    )?;
    c_weekly_rewards.set(
        "GetNumCompletedDungeonRuns",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;

    globals.set("C_WeeklyRewards", c_weekly_rewards)?;
    Ok(())
}

fn register_c_contribution_collector(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_contribution_collector = lua.create_table()?;

    c_contribution_collector.set("GetState", lua.create_function(|_, _id: i32| Ok(0i32))?)?;
    c_contribution_collector.set(
        "GetContributionCollector",
        lua.create_function(|_, _id: i32| Ok(Value::Nil))?,
    )?;
    c_contribution_collector.set(
        "GetManagedContributionsForCreatureID",
        lua.create_function(|lua, _id: i32| lua.create_table())?,
    )?;
    c_contribution_collector.set(
        "GetContributionResult",
        lua.create_function(|_, _id: i32| Ok(Value::Nil))?,
    )?;
    c_contribution_collector.set(
        "IsAwaitingRewardQuestData",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_ContributionCollector", c_contribution_collector)?;
    Ok(())
}

fn register_c_scenario(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_scenario = lua.create_table()?;

    c_scenario.set(
        "GetInfo",
        lua.create_function(|_, ()| Ok((Value::Nil, 0i32, 0i32, 0i32, false, false)))?,
    )?;
    c_scenario.set(
        "GetStepInfo",
        lua.create_function(|_, _step: Option<i32>| {
            Ok((Value::Nil, Value::Nil, 0i32, false, false))
        })?,
    )?;
    c_scenario.set(
        "GetCriteriaInfo",
        lua.create_function(|_, _criteria_index: i32| Ok(Value::Nil))?,
    )?;
    c_scenario.set(
        "IsInScenario",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_Scenario", c_scenario)?;
    Ok(())
}

fn register_c_housing(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    register_c_housing_customize(lua, &globals)?;
    register_c_dye_color(lua, &globals)?;
    register_c_house_editor(lua, &globals)?;
    register_c_housing_decor(lua, &globals)?;

    let c_housing_basic_mode = lua.create_table()?;
    c_housing_basic_mode.set("IsDecorSelected", lua.create_function(|_, ()| Ok(false))?)?;
    c_housing_basic_mode.set("GetSelectedDecorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("C_HousingBasicMode", c_housing_basic_mode)?;

    Ok(())
}

fn register_c_housing_customize(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsHoveringDecor", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetHoveredDecorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetDecorDyeSlots", lua.create_function(|lua, _decor_id: i32| lua.create_table())?)?;
    globals.set("C_HousingCustomizeMode", t)
}

fn register_c_dye_color(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "GetDyeColorInfo",
        lua.create_function(|lua, _dye_color_id: i32| {
            let info = lua.create_table()?;
            info.set("name", "Dye")?;
            info.set("dyeColorID", 0)?;
            info.set("baseColor", 0xFFFFFFu32)?;
            info.set("highlightColor", 0xFFFFFFu32)?;
            info.set("shadowColor", 0x000000u32)?;
            Ok(info)
        })?,
    )?;
    globals.set("C_DyeColor", t)
}

fn register_c_house_editor(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsHouseEditorActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActiveHouseEditorMode", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("ActivateHouseEditorMode", lua.create_function(|_, _mode: i32| Ok(()))?)?;
    t.set("GetHouseEditorModeAvailability", lua.create_function(|_, _mode: i32| Ok(false))?)?;
    t.set("IsHouseEditorModeActive", lua.create_function(|_, _mode: i32| Ok(false))?)?;
    globals.set("C_HouseEditor", t)
}

fn register_c_housing_decor(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetHoveredDecorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsHoveringDecor", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetDecorInfo", lua.create_function(|_, _decor_id: i32| Ok(Value::Nil))?)?;
    globals.set("C_HousingDecor", t)
}

fn register_c_game_rules(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_game_rules = lua.create_table()?;

    c_game_rules.set(
        "IsGameRuleActive",
        lua.create_function(|_, _rule: Value| Ok(false))?,
    )?;
    c_game_rules.set(
        "GetActiveGameMode",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_game_rules.set(
        "GetGameRuleAsFloat",
        lua.create_function(|_, _rule: Value| Ok(0.0f32))?,
    )?;
    c_game_rules.set("IsStandard", lua.create_function(|_, ()| Ok(true))?)?;

    globals.set("C_GameRules", c_game_rules)?;
    Ok(())
}

fn register_c_scripted_animations(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_scripted_anims = lua.create_table()?;

    c_scripted_anims.set(
        "GetAllScriptedAnimationEffects",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;

    globals.set("C_ScriptedAnimations", c_scripted_anims)?;
    Ok(())
}

fn register_c_glue(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_glue = lua.create_table()?;

    c_glue.set(
        "IsOnGlueScreen",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_Glue", c_glue)?;
    Ok(())
}

fn register_c_ui_color(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_ui_color = lua.create_table()?;

    c_ui_color.set(
        "GetColors",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;

    globals.set("C_UIColor", c_ui_color)?;
    Ok(())
}

fn register_c_class_color(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_class_color = lua.create_table()?;

    c_class_color.set(
        "GetClassColor",
        lua.create_function(|lua, _class_name: String| {
            let r = 1.0f32;
            let g = 1.0f32;
            let b = 1.0f32;
            let a = 1.0f32;

            let color = lua.create_table()?;
            color.set("r", r)?;
            color.set("g", g)?;
            color.set("b", b)?;
            color.set("a", a)?;

            let get_rgb = lua.create_function(move |_, ()| Ok((r, g, b)))?;
            color.set("GetRGB", get_rgb)?;

            let get_rgba = lua.create_function(move |_, ()| Ok((r, g, b, a)))?;
            color.set("GetRGBA", get_rgba)?;

            let generate_hex = lua.create_function(move |lua, ()| {
                let hex = format!(
                    "{:02x}{:02x}{:02x}",
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8
                );
                Ok(Value::String(lua.create_string(&hex)?))
            })?;
            color.set("GenerateHexColor", generate_hex)?;

            let wrap_text = lua.create_function(move |lua, (_self, text): (Value, String)| {
                let hex = format!(
                    "{:02x}{:02x}{:02x}",
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8
                );
                let wrapped = format!("|cff{}{}|r", hex, text);
                Ok(Value::String(lua.create_string(&wrapped)?))
            })?;
            color.set("WrapTextInColorCode", wrap_text)?;

            Ok(color)
        })?,
    )?;

    globals.set("C_ClassColor", c_class_color)?;
    Ok(())
}

fn register_c_spec_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_spec_info = lua.create_table()?;

    c_spec_info.set(
        "GetSpellsDisplay",
        lua.create_function(|lua, _spec_id: i32| lua.create_table())?,
    )?;
    c_spec_info.set(
        "GetInspectSelectedSpecialization",
        lua.create_function(|_, _unit: Option<String>| Ok(0))?,
    )?;
    c_spec_info.set(
        "CanPlayerUseTalentSpecUI",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    c_spec_info.set("IsInitialized", lua.create_function(|_, ()| Ok(true))?)?;
    c_spec_info.set(
        "GetSpecialization",
        lua.create_function(|_, ()| Ok(1i32))?,
    )?;
    c_spec_info.set(
        "GetSpecializationInfo",
        lua.create_function(|lua, spec_index: i32| {
            let spec_id = match spec_index {
                1 => 71,
                2 => 72,
                3 => 73,
                _ => 71,
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(spec_id),
                Value::String(lua.create_string("Arms")?),
                Value::String(lua.create_string("A battle-hardened master of weapons.")?),
                Value::Integer(132355),
                Value::String(lua.create_string("DAMAGER")?),
                Value::Integer(1),
            ]))
        })?,
    )?;
    c_spec_info.set(
        "GetAllSelectedPvpTalentIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_spec_info.set(
        "GetPvpTalentSlotInfo",
        lua.create_function(|_, _slot: i32| Ok(Value::Nil))?,
    )?;

    globals.set("C_SpecializationInfo", c_spec_info)?;
    Ok(())
}

fn register_c_artifact_ui(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_artifact_ui = lua.create_table()?;

    c_artifact_ui.set(
        "GetArtifactItemID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_artifact_ui.set(
        "GetArtifactTier",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_artifact_ui.set("IsAtForge", lua.create_function(|_, ()| Ok(false))?)?;
    c_artifact_ui.set(
        "GetEquippedArtifactInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;

    globals.set("C_ArtifactUI", c_artifact_ui)?;
    Ok(())
}

fn register_c_super_track(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_super_track = lua.create_table()?;

    c_super_track.set(
        "GetSuperTrackedMapPin",
        lua.create_function(|_, ()| Ok((Value::Nil, Value::Nil)))?,
    )?;
    c_super_track.set(
        "SetSuperTrackedMapPin",
        lua.create_function(|_, (_map_id, _x, _y): (i32, f32, f32)| Ok(()))?,
    )?;
    c_super_track.set(
        "ClearSuperTrackedMapPin",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_super_track.set(
        "GetSuperTrackedQuestID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_super_track.set(
        "SetSuperTrackedQuestID",
        lua.create_function(|_, _quest_id: i32| Ok(()))?,
    )?;
    c_super_track.set(
        "IsSuperTrackingQuest",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_super_track.set(
        "IsSuperTrackingMapPin",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_super_track.set(
        "GetSuperTrackedVignette",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_super_track.set(
        "IsSuperTrackingAnything",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_super_track.set(
        "GetSuperTrackedContent",
        lua.create_function(|_, ()| Ok((Value::Nil, Value::Nil)))?,
    )?;

    globals.set("C_SuperTrack", c_super_track)?;
    Ok(())
}

fn register_c_player_interaction_manager(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let c_player_interaction_manager = lua.create_table()?;

    c_player_interaction_manager.set(
        "IsInteractingWithNpcOfType",
        lua.create_function(|_, _npc_type: i32| Ok(false))?,
    )?;
    c_player_interaction_manager.set(
        "ClearInteraction",
        lua.create_function(|_, _interaction_type: Option<i32>| Ok(()))?,
    )?;
    c_player_interaction_manager.set(
        "GetCurrentInteraction",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;

    globals.set("C_PlayerInteractionManager", c_player_interaction_manager)?;
    Ok(())
}

fn register_c_paper_doll_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let t = lua.create_table()?;

    t.set("GetStatsError", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set(
        "GetMinItemLevel",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    t.set(
        "OffhandHasShield",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    t.set(
        "OffhandHasWeapon",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    t.set(
        "IsRangedSlotShown",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_PaperDollInfo", t)?;
    Ok(())
}

fn register_c_perks_program(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let t = lua.create_table()?;

    t.set(
        "IsTradingPostAvailable",
        lua.create_function(|_, ()| Ok(false))?,
    )?;

    globals.set("C_PerksProgram", t)?;
    Ok(())
}
