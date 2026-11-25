//! VERIFIED shapes: pinned Retail/Forever PerksActivitiesDocumentation.lua:10–221.
//! INFERRED: local tracking mutations and their documented synchronous UI event.
use super::{PerksActivities, PerksActivity};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

mod rows;

type Method = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);
const METHODS: &[Method] = &[
    ("AddTrackedPerksActivity", add),
    ("ClearPerksActivitiesPendingCompletion", clear_pending),
    ("GetAllPerksActivityTags", tags),
    ("GetPerksActivitiesInfo", activities),
    ("GetPerksActivitiesPendingCompletion", pending),
    ("GetPerksActivityChatLink", chat_link),
    ("GetPerksActivityInfo", activity),
    ("GetPerksUIThemePrefix", theme),
    ("GetTrackedPerksActivities", tracked),
    ("RemoveTrackedPerksActivity", remove),
];

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if super::active() {
        let namespace = crate::c_api::ensure_namespace(state, "C_PerksActivities")?;
        for &(name, call) in METHODS {
            table_set_rust_fn_static(state, namespace, name, call)?;
        }
    }
    Ok(())
}

fn mutate_tracking(state: &mut LuaState, added: bool) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let ids = &mut sim.perks_activities.tracked_ids;
        match (added, ids.contains(&id)) {
            (true, false) => {
                ids.push(id);
                true
            }
            (false, true) => {
                ids.retain(|&current| current != id);
                true
            }
            _ => false,
        }
    };
    if changed {
        fire_named_event_state(
            state,
            "PERKS_ACTIVITIES_TRACKED_LIST_CHANGED",
            &[Val::Num(f64::from(id)), Val::Bool(added)],
        );
    }
    Ok(0)
}

fn add(state: &mut LuaState) -> LuaResult<u32> {
    mutate_tracking(state, true)
}

fn remove(state: &mut LuaState) -> LuaResult<u32> {
    mutate_tracking(state, false)
}

fn clear_pending(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .perks_activities
        .pending_completion
        .clear();
    Ok(0)
}

fn activity(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let (row, tracked) = {
        let sim = borrow_state(state)?;
        let model = &sim.perks_activities;
        (
            model.activities.iter().find(|row| row.id == id).cloned(),
            model.tracked_ids.contains(&id),
        )
    };
    let value = row.map_or(Val::Nil, |row| rows::activity(state, &row, tracked));
    state.push(value);
    Ok(1)
}

fn activities(state: &mut LuaState) -> LuaResult<u32> {
    write_snapshot(state, |state, model| {
        rooted_table(state, |state, table| {
            number(state, table, "activePerksMonth", model.active_month);
            field_text(state, table, "displayMonthName", &model.month_name);
            set(
                state,
                table,
                "secondsRemaining",
                Val::Num(model.seconds_remaining as f64),
            );
            let items = array(state, &model.activities, |state, row: &PerksActivity| {
                rows::activity(state, row, model.tracked_ids.contains(&row.id))
            });
            set(state, table, "activities", items);
            let thresholds = array(state, &model.thresholds, rows::threshold);
            set(state, table, "thresholds", thresholds);
        })
    })
}

fn chat_link(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let link = borrow_state(state)?
        .perks_activities
        .activities
        .iter()
        .find(|row| row.id == id)
        .map(|row| row.chat_link.clone())
        .unwrap_or_default();
    let value = create_string(state, &link);
    state.push(value);
    Ok(1)
}

fn theme(state: &mut LuaState) -> LuaResult<u32> {
    let text = borrow_state(state)?.perks_activities.theme_prefix.clone();
    let value = create_string(state, &text);
    state.push(value);
    Ok(1)
}

fn tracked(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?.perks_activities.tracked_ids.clone();
    let value = id_list(state, "trackedIDs", &ids);
    state.push(value);
    Ok(1)
}

fn pending(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?
        .perks_activities
        .pending_completion
        .clone();
    let value = id_list(state, "pendingIDs", &ids);
    state.push(value);
    Ok(1)
}

fn tags(state: &mut LuaState) -> LuaResult<u32> {
    let tags = borrow_state(state)?.perks_activities.tags.clone();
    let value = rooted_table(state, |state, table| {
        let tags = array(state, &tags, |state, tag| create_string(state, tag));
        set(state, table, "tagName", tags);
    });
    state.push(value);
    Ok(1)
}

fn write_snapshot(
    state: &mut LuaState,
    build: impl FnOnce(&mut LuaState, &PerksActivities) -> Val,
) -> LuaResult<u32> {
    let model = borrow_state(state)?.perks_activities.clone();
    let value = build(state, &model);
    state.push(value);
    Ok(1)
}

fn id_list(state: &mut LuaState, name: &'static str, ids: &[i32]) -> Val {
    rooted_table(state, |state, table| {
        let entries = array(state, ids, |_, id| Val::Num(f64::from(*id)));
        set(state, table, name, entries);
    })
}

fn rooted_table(state: &mut LuaState, fill: impl FnOnce(&mut LuaState, Val)) -> Val {
    let table = create_table(state);
    state.push(table);
    fill(state, table);
    state.top -= 1;
    table
}

fn array<T>(state: &mut LuaState, items: &[T], encode: impl Fn(&mut LuaState, &T) -> Val) -> Val {
    rooted_table(state, |state, table| {
        for (idx, item) in items.iter().enumerate() {
            let value = encode(state, item);
            set_table_array(state, table, idx as i64 + 1, value);
        }
    })
}

fn set(state: &mut LuaState, table: Val, name: &'static str, value: Val) {
    table_set_static(state, table, name, value);
}

fn number(state: &mut LuaState, table: Val, name: &'static str, value: i32) {
    set(state, table, name, Val::Num(f64::from(value)));
}

fn field_text(state: &mut LuaState, table: Val, name: &'static str, value: &str) {
    let text = create_string(state, value);
    set(state, table, name, text);
}
