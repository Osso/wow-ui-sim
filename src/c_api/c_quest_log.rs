//! Quest queries projected from explicitly supplied simulator state.

#[cfg(feature = "client-wowforever")]
use super::helpers::read_untainted_signed32_argument;
use super::helpers::{ensure_namespace, validate_supplied_relation_id};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, val_to_string,
};
use crate::lua_api::sim_substates::{QuestLogEntry, QuestLogState};
use crate::lua_api::workarounds::update_campaign_headers;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
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

/// R88e.2 log-entry flags, separate from quest-record or category inputs.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuestInfoEntryFlags {
    pub is_task: bool,
    pub is_bounty: bool,
}

const QUEST_LOG_HEADER_TITLE: &str = "Khaz Algar";

pub(crate) fn register_info_queries(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_QuestLog")?;
    table_set_rust_fn_static(state, namespace, "GetInfo", get_info)?;
    if ACTIVE == ClientProfile::Retail {
        table_set_rust_fn_static(state, namespace, "IsQuestTask", is_quest_task)?;
        table_set_rust_fn_static(state, namespace, "IsQuestBounty", is_quest_bounty)?;
    }
    Ok(())
}

fn info_entry_flags(quests: &QuestLogState, entry: &QuestLogEntry) -> QuestInfoEntryFlags {
    if ACTIVE == ClientProfile::Retail {
        quests
            .info_entry_flags
            .get(&entry.quest_id)
            .copied()
            .unwrap_or_default()
    } else {
        QuestInfoEntryFlags {
            is_task: entry.is_world_quest,
            is_bounty: false,
        }
    }
}

fn is_quest_task(state: &mut LuaState) -> LuaResult<u32> {
    let id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let result = {
        let sim = borrow_state(state)?;
        let quests = &sim.quest_log_entries;
        if let Some(entry) = quests.entries.iter().find(|entry| entry.quest_id == id) {
            info_entry_flags(quests, entry).is_task
        } else if quests.task_category_five.contains(&id) {
            quests.task_list_lookup_matches.contains(&id)
        } else if let Some(value) = quests.task_record_values.get(&id) {
            matches!(value, 3 | 5)
        } else {
            quests.task_list_lookup_matches.contains(&id)
        }
    };
    state.push(Val::Bool(result));
    Ok(1)
}

fn is_quest_bounty(state: &mut LuaState) -> LuaResult<u32> {
    let id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let result = borrow_state(state)?
        .quest_log_entries
        .bounty_category_matches
        .contains(&id);
    state.push(Val::Bool(result));
    Ok(1)
}

fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let log_index = i32::from_stack(state, 1)?;
    // Native absent Lua return is UNKNOWN; keep the existing zero-return policy.
    if log_index < 1 {
        return Ok(0);
    }
    let kind = {
        let sim = borrow_state(state)?;
        classify_quest_log_index(sim.quest_log_entries.entries.len(), log_index)
    };
    let Some(kind) = kind else { return Ok(0) };

    let t = create_table(state);
    match kind {
        QuestLogIndexKind::Header => {
            write_header_identity_fields(state, t, log_index);
            write_header_constant_fields(state, t);
        }
        QuestLogIndexKind::Quest(quest_entry_index) => {
            let entry = borrow_state(state)?
                .quest_log_entries
                .entries
                .get(quest_entry_index)
                .cloned();
            let Some(entry) = entry else { return Ok(0) };
            write_quest_identity_fields(state, t, &entry, log_index);
            let (campaign, flags, auto_complete) = {
                let sim = borrow_state(state)?;
                let quests = &sim.quest_log_entries;
                let campaign = quests
                    .campaign_ids
                    .get(&entry.quest_id)
                    .copied()
                    .unwrap_or(0);
                let flags = info_entry_flags(quests, &entry);
                let prior_flag = crate::c_api::quest_objectives::ENABLED
                    && quests.readiness.auto_complete.contains(&entry.quest_id);
                let auto_complete = if ACTIVE == ClientProfile::Retail {
                    quests
                        .info_record_autocomplete
                        .get(&entry.quest_id)
                        .copied()
                        .unwrap_or(prior_flag)
                } else {
                    prior_flag
                };
                (campaign, flags, auto_complete)
            };
            write_quest_classification_flags(state, t, &entry);
            write_quest_constant_stub_fields(state, t, false);
            table_set(state, t, "isTask", Val::Bool(flags.is_task));
            table_set(state, t, "isBounty", Val::Bool(flags.is_bounty));
            table_set(state, t, "campaignID", Val::Num(f64::from(campaign)));
            table_set(state, t, "isAutoComplete", Val::Bool(auto_complete));
        }
    }

    state.push(t);
    Ok(1)
}

#[derive(Clone, Copy)]
pub(crate) enum QuestLogIndexKind {
    Header,
    Quest(usize),
}

pub(crate) fn classify_quest_log_index(
    entry_count: usize,
    log_index: i32,
) -> Option<QuestLogIndexKind> {
    if entry_count == 0 || log_index < 1 {
        return None;
    }
    if log_index == 1 {
        return Some(QuestLogIndexKind::Header);
    }
    let quest_index = (log_index - 2) as usize;
    (quest_index < entry_count).then_some(QuestLogIndexKind::Quest(quest_index))
}

fn write_header_identity_fields(state: &mut LuaState, t: Val, log_index: i32) {
    table_set(state, t, "questID", Val::Num(0.0));
    let title = create_string(state, QUEST_LOG_HEADER_TITLE);
    table_set(state, t, "title", title);
    table_set(state, t, "level", Val::Num(0.0));
    table_set(state, t, "questLogIndex", Val::Num(log_index as f64));
    table_set(state, t, "difficultyLevel", Val::Num(0.0));
    table_set(state, t, "isComplete", Val::Bool(false));
    table_set(state, t, "isFailed", Val::Bool(false));
}

fn write_quest_identity_fields(
    state: &mut LuaState,
    t: Val,
    entry: &QuestLogEntry,
    log_index: i32,
) {
    table_set(state, t, "questID", Val::Num(entry.quest_id as f64));
    let title = create_string(state, &entry.title);
    table_set(state, t, "title", title);
    table_set(state, t, "level", Val::Num(entry.level as f64));
    table_set(state, t, "questLogIndex", Val::Num(log_index as f64));
    table_set(state, t, "difficultyLevel", Val::Num(entry.level as f64));
    table_set(state, t, "isComplete", Val::Bool(entry.is_complete));
    table_set(state, t, "isFailed", Val::Bool(entry.is_failed));
}

fn write_quest_classification_flags(state: &mut LuaState, t: Val, entry: &QuestLogEntry) {
    table_set(state, t, "isMeta", Val::Bool(entry.is_meta));
    table_set(state, t, "isWorldQuest", Val::Bool(entry.is_world_quest));
    table_set(state, t, "isOnMap", Val::Bool(entry.map_id.is_some()));
}

/// Fields that the sim always reports with a constant value — headers,
/// bounty/story categories, POI flags, sort/scaling toggles, and the
/// campaign/suggestedGroup identifiers we don't model.
fn write_quest_constant_stub_fields(state: &mut LuaState, t: Val, is_header: bool) {
    table_set(state, t, "isHeader", Val::Bool(is_header));
    table_set(state, t, "isCollapsed", Val::Bool(false));
    table_set(state, t, "isStory", Val::Bool(false));
    table_set(state, t, "hasLocalPOI", Val::Bool(false));
    table_set(state, t, "isHidden", Val::Bool(false));
    table_set(state, t, "overridesSortOrder", Val::Bool(false));
    table_set(state, t, "startEvent", Val::Bool(false));
    table_set(state, t, "isScaling", Val::Bool(false));
    table_set(state, t, "readyForTranslation", Val::Bool(false));
    table_set(state, t, "suggestedGroup", Val::Num(0.0));
}

fn write_header_constant_fields(state: &mut LuaState, t: Val) {
    table_set(state, t, "isBounty", Val::Bool(false));
    table_set(state, t, "isAutoComplete", Val::Bool(false));
    table_set(state, t, "isMeta", Val::Bool(false));
    table_set(state, t, "isWorldQuest", Val::Bool(false));
    table_set(state, t, "isTask", Val::Bool(false));
    table_set(state, t, "isOnMap", Val::Bool(false));
    write_quest_constant_stub_fields(state, t, true);
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
    // Forever's parser is native-backed; Retail/PTR parser parity remains unknown.
    #[cfg(any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever"
    ))]
    table_set_rust_fn_static(state, namespace, "IsAccountQuest", is_account_quest)?;
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    table_set_rust_fn_static(
        state,
        namespace,
        "QuestCanHaveWarModeBonus",
        quest_can_have_war_mode_bonus,
    )?;
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

#[cfg(feature = "client-wowforever")]
fn is_account_quest(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = read_untainted_signed32_argument(
        state,
        1,
        "IsAccountQuest",
        "Usage: local isAccountQuest = C_QuestLog.IsAccountQuest(questID)",
    )?;
    let account = account_quest_for_id(state, Some(quest_id))?;
    state.push(Val::Bool(account));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn is_account_quest(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = validate_supplied_relation_id(stack_val(state, 1), "questID must be a number")?;
    let account = account_quest_for_id(state, quest_id)?;
    state.push(Val::Bool(account));
    Ok(1)
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn account_quest_for_id(state: &LuaState, quest_id: Option<i32>) -> LuaResult<bool> {
    let Some(quest_id) = quest_id.filter(|id| *id >= 1) else {
        return Ok(false);
    };
    let sim = borrow_state(state)?;
    Ok(sim
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .and_then(|entry| entry.is_account_quest)
        .unwrap_or(false))
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

#[cfg(all(test, feature = "client-retail"))]
mod r88e2_tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn quest_task_no_row_uses_verified_lookup_priority() {
        let env = WowLuaEnv::new().unwrap();
        {
            let mut sim = env.state().borrow_mut();
            let quests = &mut sim.quest_log_entries;
            quests.task_category_five.extend([101_001, 101_002]);
            quests
                .task_list_lookup_matches
                .extend([101_001, 101_005, 101_006]);
            quests.task_record_values.extend([
                (101_002, 3),
                (101_003, 3),
                (101_004, 5),
                (101_005, 2),
            ]);
            let row = &quests.entries[0];
            quests.task_category_five.insert(row.quest_id);
            quests.task_list_lookup_matches.insert(row.quest_id);
        }
        env.exec(
            r#"
            assert(C_QuestLog.IsQuestTask(101001))
            assert(not C_QuestLog.IsQuestTask(101002))
            assert(C_QuestLog.IsQuestTask(101003))
            assert(C_QuestLog.IsQuestTask(101004))
            assert(not C_QuestLog.IsQuestTask(101005))
            assert(C_QuestLog.IsQuestTask(101006))
            assert(not C_QuestLog.IsQuestTask(101007))
            local row = C_QuestLog.GetInfo(2)
            assert(not row.isTask and not C_QuestLog.IsQuestTask(row.questID))
        "#,
        )
        .unwrap();
    }
}
