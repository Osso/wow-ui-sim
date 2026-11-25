use super::{array, field_text, number, rooted_table, set};
use crate::c_api::perks_activities::{
    PerksActivity, PerksCondition, PerksCriteria, PerksRequirement, PerksThreshold,
};
use crate::lua_api::methods::create_string;
use rilua::Val;
use rilua::vm::state::LuaState;

pub(super) fn activity(state: &mut LuaState, row: &PerksActivity, tracked: bool) -> Val {
    rooted_table(state, |state, table| {
        number(state, table, "ID", row.id);
        field_text(state, table, "activityName", &row.name);
        field_text(state, table, "description", &row.description);
        number(
            state,
            table,
            "thresholdContributionAmount",
            row.contribution,
        );
        set(state, table, "completed", Val::Bool(row.completed));
        set(state, table, "inProgress", Val::Bool(row.in_progress));
        set(state, table, "tracked", Val::Bool(tracked));
        number(state, table, "supersedes", row.supersedes);
        number(state, table, "uiPriority", row.ui_priority);
        set(
            state,
            table,
            "areAllConditionsMet",
            Val::Bool(row.all_conditions_met),
        );
        activity_nested(state, table, row);
    })
}

fn activity_nested(state: &mut LuaState, table: Val, row: &PerksActivity) {
    if let Some(name) = &row.event_name {
        field_text(state, table, "eventName", name);
    }
    for (key, time) in [
        ("eventStartTime", row.event_start_time),
        ("eventEndTime", row.event_end_time),
    ] {
        if let Some(time) = time {
            set(state, table, key, Val::Num(time as f64));
        }
    }
    let conditions = array(state, &row.conditions, condition);
    set(state, table, "conditions", conditions);
    let requirements = array(state, &row.requirements, requirement);
    set(state, table, "requirementsList", requirements);
    let criteria = array(state, &row.criteria, criterion);
    set(state, table, "criteriaList", criteria);
    let tags = array(state, &row.tags, |state, tag| create_string(state, tag));
    set(state, table, "tagNames", tags);
}

fn condition(state: &mut LuaState, row: &PerksCondition) -> Val {
    rooted_table(state, |state, table| {
        set(state, table, "isMet", Val::Bool(row.is_met));
        field_text(state, table, "text", &row.text);
        number(state, table, "uiPriority", row.ui_priority);
    })
}

fn requirement(state: &mut LuaState, row: &PerksRequirement) -> Val {
    rooted_table(state, |state, table| {
        set(state, table, "completed", Val::Bool(row.completed));
        field_text(state, table, "requirementText", &row.text);
    })
}

fn criterion(state: &mut LuaState, row: &PerksCriteria) -> Val {
    rooted_table(state, |state, table| {
        number(state, table, "criteriaID", row.id);
        set(
            state,
            table,
            "requiredValue",
            Val::Num(row.required_value as f64),
        );
    })
}

pub(super) fn threshold(state: &mut LuaState, row: &PerksThreshold) -> Val {
    rooted_table(state, |state, table| {
        number(state, table, "thresholdOrderIndex", row.order_index);
        number(state, table, "currencyAwardAmount", row.currency_award);
        number(
            state,
            table,
            "requiredContributionAmount",
            row.required_contribution,
        );
        set(state, table, "pendingReward", Val::Bool(row.pending_reward));
        if let Some(item) = row.item_reward {
            number(state, table, "itemReward", item);
        }
    })
}
