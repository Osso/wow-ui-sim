//! Quest queries projected from explicitly supplied simulator state.

use super::helpers::{ensure_namespace, validate_supplied_relation_id};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_api::workarounds::update_campaign_headers;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Map and categories highlighted by a quest. The default has no highlights.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QuestAdditionalHighlights {
    pub ui_map_id: i32,
    pub world_quests: bool,
    pub world_quests_elite: bool,
    pub dungeons: bool,
    pub treasures: bool,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    super::c_map_bounties::register_quest_queries(state)?;
    let namespace = ensure_namespace(state, "C_QuestLog")?;
    // Retail and Forever QuestLogDocumentation.lua:46 declare this query.
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        table_set_rust_fn_static(state, namespace, "CanAbandonQuest", can_abandon_quest)?;
    }
    // Retail/PTR/Forever QuestLogDocumentation.lua declare these queries
    // (IsUnitOnQuest :1033/:1066; UpdateCampaignHeaders :1269/:1302).
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(
            state,
            namespace,
            "UpdateCampaignHeaders",
            update_campaign_headers,
        )?;
        table_set_rust_fn_static(state, namespace, "IsUnitOnQuest", is_unit_on_quest)?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "GetQuestAdditionalHighlights",
        additional_highlights,
    )?;
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    {
        let namespace = ensure_namespace(state, "C_QuestLog")?;
        // QuestLogDocumentation.lua:701 Retail, :719 PTR; user-approved default.
        table_set_rust_fn_static(state, namespace, "IsAccountQuest", is_account_quest)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "QuestCanHaveWarModeBonus",
            quest_can_have_war_mode_bonus,
        )?;
    }
    // Retail/PTR/Forever QuestLogDocumentation.lua:380-392 declares this query.
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetQuestDifficultyLevel",
            get_quest_difficulty_level,
        )?;
    }
    #[cfg(feature = "client-wowforever")]
    register_forever_queries(state)?;
    if super::c_quest_log_watches::ENABLED {
        super::c_quest_log_watches::register(state)?;
    }
    Ok(())
}

fn can_abandon_quest(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => crate::lua_api::methods::val_to_string(state, value)
            .and_then(|text| text.trim().parse::<f64>().ok()),
        _ => None,
    }
    .ok_or_else(|| {
        rilua::runtime_error("Usage: local canAbandon = C_QuestLog.CanAbandonQuest(questID)")
    })?;
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(rilua::runtime_error("questID outside signed 32-bit range"));
    }
    let id = number as i32;
    let can_abandon = {
        let sim = borrow_state(state)?;
        let quests = &sim.quest_log_entries;
        let ready = id > 0
            && (quests.abandon_ready_ids.contains(&id)
                || quests.entries.iter().any(|entry| entry.quest_id == id));
        let classification = quests
            .abandon_classification_flags
            .get(&id)
            .copied()
            .unwrap_or(0);
        let flags = quests.abandon_record_flags.get(&id).copied().unwrap_or(0);
        let conditional_blocked = if ready && flags & 2048 != 0 {
            *quests.abandon_conditional_blocked.get(&id).ok_or_else(|| {
                rilua::runtime_error("quest abandon conditional state is not supplied")
            })?
        } else {
            false
        };
        ready && classification & 256 == 0 && !conditional_blocked
    };
    state.push(Val::Bool(can_abandon));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn register_forever_queries(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_QuestLog")?;
    table_set_rust_fn_static(state, namespace, "IsEliteQuest", is_elite_quest)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn is_account_quest(state: &mut LuaState) -> LuaResult<u32> {
    let id = validate_supplied_relation_id(stack_val(state, 1), "questID must be a number")?;
    // User-approved default, not native evidence; supplied quest data takes precedence.
    let account = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| Some(entry.quest_id) == id)
        .and_then(|entry| entry.is_account_quest)
        .unwrap_or(false);
    state.push(Val::Bool(account));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn quest_can_have_war_mode_bonus(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    #[cfg(feature = "retail-12-1-0")]
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let quest_id = validate_supplied_relation_id(
        value,
        "C_QuestLog.QuestCanHaveWarModeBonus: questID must be a number",
    )?;
    let eligible = {
        let sim = borrow_state(state)?;
        quest_id.is_some_and(|id| sim.quest_war_mode_eligibility.contains(&id))
    };
    state.push(Val::Bool(eligible));
    Ok(1)
}

fn is_unit_on_quest(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::unit_misc::{
        existing_guid_for_unit, guid_for_unit, party_guid_for_index,
    };
    use crate::lua_api::methods::val_to_string;

    let unit = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let unit = match unit {
        Val::Str(_) => val_to_string(state, unit).unwrap_or_default(),
        Val::Num(number) => number.to_string(),
        _ => return Err(rilua::runtime_error("IsUnitOnQuest requires a unit")),
    };
    let identity = {
        let sim = borrow_state(state)?;
        existing_guid_for_unit(&sim, &unit)
    };
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let quest_id = validate_supplied_relation_id(value, "IsUnitOnQuest requires a questID")?;
    let on_quest = {
        let sim = borrow_state(state)?;
        let log = &sim.quest_log_entries;
        match (identity, quest_id.filter(|id| *id > 0)) {
            (Some(guid), Some(quest_id)) if sim.party_group_active => {
                if guid == guid_for_unit(&sim, "player") {
                    log.entries.iter().any(|entry| entry.quest_id == quest_id)
                } else if sim
                    .party_members
                    .iter()
                    .enumerate()
                    .any(|(index, _)| party_guid_for_index(index) == guid)
                {
                    log.party_quest_ids
                        .get(&guid)
                        .is_some_and(|quests| quests.contains(&quest_id))
                } else {
                    false
                }
            }
            _ => false,
        }
    };
    state.push(Val::Bool(on_quest));
    Ok(1)
}

/// UNKNOWN/EMULATED quest-difficulty-level: return the supplied rich row's level;
/// absent data is an error. Native tuning, unknown-ID, error and secrecy behavior
/// remain unverified. This is the existing model shared by declared profiles.
fn get_quest_difficulty_level(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = validate_supplied_relation_id(
        stack_val(state, 1),
        "C_QuestLog.GetQuestDifficultyLevel: questID must be a number",
    )?
    .ok_or_else(|| {
        rilua::runtime_error(
            "C_QuestLog.GetQuestDifficultyLevel: questID must be a finite i32 integer",
        )
    })?;
    let level = {
        let sim = borrow_state(state)?;
        sim.quest_log_entries
            .entries
            .iter()
            .find(|entry| entry.quest_id == quest_id)
            .map(|entry| entry.level)
    }
    .ok_or_else(|| {
        rilua::runtime_error(format!(
            "C_QuestLog.GetQuestDifficultyLevel: no supplied quest-log entry for questID {quest_id}"
        ))
    })?;
    state.push(Val::Num(f64::from(level)));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn is_elite_quest(state: &mut LuaState) -> LuaResult<u32> {
    // Forever's QuestConstantsDocumentation declares this tag as QUEST_INFO_ELITE_ID.
    const ELITE_TAG_ID: i32 = 1;
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let quest_id =
        validate_supplied_relation_id(value, "C_QuestLog.IsEliteQuest: questID must be a number")?;
    let is_elite = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .any(|entry| Some(entry.quest_id) == quest_id && entry.tag_id == Some(ELITE_TAG_ID));
    state.push(Val::Bool(is_elite));
    Ok(1)
}

/// Project the quest's extra highlights, or the empty set if none are modeled.
/// The empty set uses the numeric map ID and four booleans declared by Blizzard.
fn additional_highlights(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.trim().parse().ok()),
        _ => None,
    };
    let quest_id = number
        .filter(|number: &f64| {
            number.is_finite() && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(number)
        })
        .ok_or_else(|| rilua::runtime_error("questID must be a finite i32 number"))?
        as i32;
    let row = borrow_state(state)?
        .quest_log_entries
        .additional_highlights
        .get(&quest_id)
        .copied()
        .unwrap_or_default();
    state.push(Val::Num(f64::from(row.ui_map_id)));
    state.push(Val::Bool(row.world_quests));
    state.push(Val::Bool(row.world_quests_elite));
    state.push(Val::Bool(row.dungeons));
    state.push(Val::Bool(row.treasures));
    Ok(5)
}
